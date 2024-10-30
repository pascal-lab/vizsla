use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use itertools::Either;
use line_index::TextRange;
use nohash_hasher::IntMap;
use span::FilePosition;
use syntax::{
    SyntaxNodeExt, SyntaxTokenWithParent, TokenKind,
    ast::{self, AstNode},
    has_text_range::HasTextRange,
    match_ast, support,
    token::pair_token,
};
use vfs::FileId;

use crate::navigation_target::NavTarget;

bitflags::bitflags! {
    #[derive(Copy, Clone, Default, PartialEq, Eq, Hash, Debug)]
    pub struct ReferenceCategory: u8 {
        const WRITE = 1 << 0;
        const READ = 1 << 1;
    }
}

#[derive(Debug, Clone)]
pub struct References {
    pub def: Option<NavTarget>,
    pub refs: IntMap<FileId, Vec<(TextRange, ReferenceCategory)>>,
}

pub(crate) fn references(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> Option<Vec<References>> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);

    let token = file.syntax().token_at_offset(offset).pick_bext_token(token_precedence)?;

    handle_ctrl_flow_kw(&sema, token)
}

fn token_precedence(kind: TokenKind) -> usize {
    match kind {
        _ if pair_token(kind).is_some() => 4,
        _ => 1,
    }
}

fn handle_ctrl_flow_kw(
    sema: &Semantics<'_, RootDb>,
    SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
) -> Option<Vec<References>> {
    let kind = tok.kind();
    let file_id = sema.find_file(parent);
    let mut refs = vec![(tok.text_range().unwrap(), ReferenceCategory::empty())];

    if let Some(paired_kw_kind) = pair_token(kind) {
        let paired_kw = match paired_kw_kind {
            Either::Left(kind) => {
                match_ast! { parent in
                    ast::ModuleDeclaration as it => it.header().module_keyword(),
                    _ => support::child_token(parent, kind),
                }
            }
            Either::Right(kind) => {
                match_ast! { parent in
                    ast::ModuleHeader as it => {
                        let parent = it.syntax().parent().unwrap();
                        let decl = ast::ModuleDeclaration::cast(parent).unwrap();
                        decl.endmodule()
                    },
                    _ => support::child_token(parent, kind),
                }
            }
        };

        if let Some(paired_kw) = paired_kw {
            refs.push((paired_kw.text_range().unwrap(), ReferenceCategory::empty()));
        }
    }

    Some(vec![References { def: None, refs: IntMap::from_iter([(file_id.file_id(), refs)]) }])
}
