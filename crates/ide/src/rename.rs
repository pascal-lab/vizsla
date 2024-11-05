use base_db::source_db::SourceDb;
use hir::{container::InFile, hir_def::lower_ident, semantics::Semantics};
use ide_db::root_db::RootDb;
use line_index::{TextRange, TextSize};
use span::FilePosition;
use syntax::{
    SyntaxAncestors, SyntaxNode, SyntaxNodeExt, SyntaxTokenWithParent,
    ast::{self, AstNode},
    has_text_range::{HasTextRange, SourceRangeExt},
    match_ast,
    token::TokenKindExt,
};
use thiserror::Error;
use utils::text_edit::TextEdit;
use vfs::FileId;

use crate::{
    ScopeVisibility,
    definitions::{Definition, DefinitionClass},
    navigation_target::ToNav,
    references::{
        ReferencesConfig,
        search::{ReferenceToken, ReferencesCtx},
    },
    source_change::SourceChange,
};

pub type RenameResult<T> = Result<T, RenameError>;

#[derive(Debug, Clone)]
pub struct RenameConfig {
    pub scope_visibility: ScopeVisibility,
}

#[derive(Error, Debug)]
pub enum RenameError {
    #[error("No references found at position")]
    NoRefFound,
    #[error("No definitions found for the token")]
    NoDefFound,
}

pub(crate) fn prepare_rename(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> RenameResult<TextRange> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = pick_token(file.syntax(), offset)?;
    let text_range = token.text_range().ok_or(RenameError::NoRefFound)?;
    DefinitionClass::resolve(&sema, token).ok_or(RenameError::NoDefFound)?;
    Ok(text_range)
}

pub(crate) fn rename(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
    RenameConfig { scope_visibility }: RenameConfig,
    new_name: &str,
) -> RenameResult<SourceChange> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = pick_token(file.syntax(), offset)?;
    let def = match DefinitionClass::resolve(&sema, token).ok_or(RenameError::NoDefFound)? {
        DefinitionClass::Definition(def) => def,
        DefinitionClass::PortConnShorthand { data, .. } => data,
    };

    let old_name = lower_ident(Some(token.tok)).unwrap();
    let mut source_changes = SourceChange::default();
    let ref_edits = ReferencesCtx::new(&sema, &def, ReferencesConfig::new(scope_visibility, None))
        .search()
        .into_iter()
        .map(|(file_id, toks)| {
            edits_from_references(&sema, file_id, toks, &def, &old_name, new_name)
        });
    source_changes.extend(ref_edits);

    let def_edits = def.sources().into_iter().map(|def| {
        let mut text_edit = TextEdit::builder();
        // TODO: optimization??
        let nav = def.to_nav(db);
        text_edit.replace(nav.focus_range.unwrap(), new_name.to_owned());
        (nav.file_id, text_edit.finish())
    });

    // TODO:
    // source_changes.extend(def_edits);
    Ok(source_changes)
}

fn edits_from_references(
    sema: &Semantics<'_, RootDb>,
    file_id: FileId,
    toks: Vec<ReferenceToken<'_>>,
    def: &Definition,
    old_name: &str,
    new_name: &str,
) -> (FileId, TextEdit) {
    let mut text_edit = TextEdit::builder();
    let text = sema.db.file_text(file_id);

    for ReferenceToken { token: SyntaxTokenWithParent { parent, tok } } in toks.into_iter() {
        let range = tok.range().unwrap().to_text_range();

        let conn_data_range = |it: ast::NamedPortConnection| it.expr()?.syntax().text_range();

        match_ast! { parent,
            ast::NamedPortConnection[it] if it.name() == Some(tok) => {
                // .[port](data)
                match (it.open_paren(), it.close_paren()) {
                    (Some(_), Some(cp)) if conn_data_range(it).is_some_and(|r| &text[r] == new_name) => {
                        // .port(new),  => .new,
                        let end = cp.text_range().unwrap().end();
                        text_edit.replace(TextRange::new(range.start(), end), new_name.to_owned());
                    }
                    (None, None) => {
                        let ref_container = sema.find_container(InFile::new(file_id.into(), it.syntax()));
                        if def.container(sema.db) == ref_container {
                            // .old => .old(new)
                            text_edit.replace(range, format!("{old_name}({new_name})"));
                        } else {
                            // .old => .new(old)
                            text_edit.replace(range, format!("{new_name}({old_name})"));
                        }
                    }
                    _ => text_edit.replace(range, new_name.to_owned()),
                }
            },
            ast::IdentifierName => {
                let port_conn = SyntaxAncestors::start_from(parent).nth(3).and_then(ast::NamedPortConnection::cast);
                let data_range = port_conn.and_then(|it| conn_data_range(it));
                let port_name = port_conn.and_then(|it| lower_ident(it.name()));

                if data_range.is_some_and(|data_range| data_range == range)
                && port_name.is_some_and(|port_name| port_name == new_name) {
                    // .new(data) => .new
                    let port_conn = port_conn.unwrap();
                    let start = port_conn.name().unwrap().text_range().unwrap().start();
                    let end = if let Some(cp) = port_conn.close_paren() {
                        cp.text_range().unwrap().end()
                    }else {
                        range.end()
                    };
                    text_edit.replace(TextRange::new(start, end), new_name.to_owned());

                } else {
                    text_edit.replace(range, new_name.to_owned());
                }
            },
            _ => text_edit.replace(range, new_name.to_owned()),
        }
    }

    (file_id, text_edit.finish())
}

fn pick_token(node: SyntaxNode, offset: TextSize) -> RenameResult<SyntaxTokenWithParent> {
    node.token_at_offset(offset)
        .pick_bext_token(|kind| kind.name_like().into())
        .ok_or(RenameError::NoRefFound)
}
