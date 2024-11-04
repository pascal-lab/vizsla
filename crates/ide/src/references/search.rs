use std::cell::LazyCell;

use base_db::{intern::Lookup, salsa::Database, source_db::SourceDb};
use hir::{
    container::{ContainerId, InFile},
    db::HirDb,
    semantics::Semantics,
    source_map::IsSrc,
};
use ide_db::root_db::RootDb;
use itertools::Itertools;
use line_index::{TextRange, TextSize};
use memchr::memmem::Finder;
use nohash_hasher::IntMap;
use rustc_hash::FxHashMap;
use syntax::{
    SyntaxNode, SyntaxNodeExt, SyntaxTokenWithParent, TokenKind, ast::AstNode,
    has_text_range::HasTextRange,
};
use triomphe::Arc;
use utils::get::Get;
use vfs::FileId;

use super::{ReferenceCategory, ReferencesConfig};
use crate::{
    ScopeVisibility,
    definitions::{Definition, DefinitionClass, PortConnShorthand},
};

/// A search scope is a set of files and ranges within those files that should
/// be searched. None means the whole file.
#[derive(Default, Debug, Clone)]
pub struct SearchScope(FxHashMap<FileId, Option<TextRange>>);

impl SearchScope {
    pub(crate) fn single_file(file_id: FileId) -> Self {
        let mut res = FxHashMap::default();
        res.insert(file_id, None);
        SearchScope(res)
    }

    fn new(
        db: &RootDb,
        def: &Definition,
        ReferencesConfig { scope_visibility, search_scope }: ReferencesConfig,
    ) -> Self {
        match scope_visibility {
            ScopeVisibility::Public => search_scope.unwrap_or_else(|| Self::all(db)),
            ScopeVisibility::Private => {
                let cont_for_defs = def.iter().map(|src| src.container(db)).unique().collect_vec();
                let mut scope = Self::from_conts(db, cont_for_defs);

                if let Some(search_scope) = search_scope {
                    scope = scope.intersect(search_scope);
                }

                scope
            }
        }
    }

    fn all(db: &RootDb) -> Self {
        let res = db.files().iter().map(|&file_id| (file_id, None)).collect();
        SearchScope(res)
    }

    fn from_conts(db: &RootDb, cont: Vec<ContainerId>) -> Self {
        let mut res = FxHashMap::default();

        let mut union_or_insert = |file_id, new_range: TextRange| {
            use std::collections::hash_map::Entry::*;
            match res.entry(file_id) {
                Occupied(mut e) => {
                    if let Some(old_range) = e.get_mut() {
                        *old_range = new_range.cover(*old_range);
                    }
                }
                Vacant(e) => {
                    e.insert(Some(new_range));
                }
            }
        };

        for cont_id in cont {
            match cont_id {
                ContainerId::HirFileId(_) => return Self::all(db),
                ContainerId::ModuleId(InFile { value: local_module_id, cont_id: file_id }) => {
                    let (_, file_src_map) = db.hir_file_with_source_map(file_id);
                    let range = file_src_map.get(local_module_id).range();
                    union_or_insert(file_id.file_id(), range);
                }
                ContainerId::BlockId(block_id) => {
                    let range = block_id.lookup(db).src.value.range();
                    union_or_insert(block_id.file_id(db), range);
                }
            }
        }

        SearchScope(res)
    }

    fn intersect(mut self, mut other: SearchScope) -> SearchScope {
        if self.0.len() > other.0.len() {
            std::mem::swap(&mut self, &mut other)
        }

        self.0.retain(|file_id, range| {
            if let Some(other_range) = other.0.get(file_id) {
                match (&range, &other_range) {
                    (Some(r), Some(other)) => *range = r.intersect(*other),
                    (None, Some(other)) => *range = Some(*other),
                    (Some(_), None) | (None, None) => {}
                };
                true
            } else {
                false
            }
        });

        self
    }
}

pub(crate) struct ReferencesCtx<'a, 'b> {
    sema: &'a Semantics<'a, RootDb>,
    def: &'b Definition,
    scope: SearchScope,
}

impl<'a, 'b> ReferencesCtx<'a, 'b> {
    pub(crate) fn new(
        sema: &'a Semantics<'a, RootDb>,
        def: &'b Definition,
        cfg: ReferencesConfig,
    ) -> Self {
        let scope = SearchScope::new(sema.db, &def, cfg);
        Self { sema, def, scope }
    }

    pub(crate) fn search(&self) -> IntMap<FileId, Vec<(TextRange, ReferenceCategory)>> {
        let sema = &self.sema;
        let mut res: IntMap<_, Vec<_>> = IntMap::default();

        let Some(name) = self.def.iter().next().and_then(|def| def.name(sema.db)) else {
            return res;
        };

        let mut add_ref = |file_id, range, category| {
            res.entry(file_id).or_default().push((range, category));
        };

        let finder = &Finder::new(&name);
        for (text, file_id, range) in self.scope_files() {
            self.sema.db.unwind_if_cancelled();

            let file = LazyCell::new(|| sema.parse(file_id));
            Self::match_text(&text, finder, range)
                .filter_map(|offset| Self::filter_token(file.syntax(), offset))
                .filter(|tp| {
                    DefinitionClass::resolve(sema, *tp).is_some_and(|def| self.filter_def(def))
                })
                .for_each(|tp| {
                    add_ref(file_id, tp.text_range().unwrap(), ReferenceCategory::empty())
                });
        }

        res
    }

    fn scope_files(&self) -> impl Iterator<Item = (Arc<str>, FileId, TextRange)> + '_ {
        let db = self.sema.db;

        self.scope.0.iter().map(|(file_id, range)| {
            let text = db.file_text(*file_id);
            let range = range.unwrap_or_else(|| TextRange::up_to(TextSize::of(&*text)));
            (text, *file_id, range)
        })
    }

    fn match_text<'c>(
        text: &'c str,
        finder: &'c Finder,
        search_range: TextRange,
    ) -> impl Iterator<Item = TextSize> + 'c {
        finder.find_iter(text.as_bytes()).filter_map(move |idx| {
            let offset = TextSize::from(idx as u32);
            if !search_range.contains_inclusive(offset) {
                return None;
            }

            // If this is not a word boundary, that means this is only part of an ident.
            if text[..idx].chars().next_back().is_some_and(|ch| ch.is_alphabetic() || ch == '_')
                || text[idx + finder.needle().len()..]
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
            {
                return None;
            }

            Some(offset)
        })
    }

    fn filter_token(node: SyntaxNode, offset: TextSize) -> Option<SyntaxTokenWithParent> {
        node.token_at_offset(offset).pick_bext_token(|kind| match kind {
            TokenKind::IDENTIFIER | TokenKind::SYSTEM_IDENTIFIER => 1,
            _ => 0,
        })
    }

    fn filter_def(&self, found: DefinitionClass) -> bool {
        match found {
            DefinitionClass::Definition(def) => def == *self.def,
            DefinitionClass::PortConnShorthand(PortConnShorthand { data, .. }) => data == *self.def,
        }
    }
}
