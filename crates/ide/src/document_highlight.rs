use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use itertools::Either;
use line_index::TextRange;
use span::FilePosition;
use syntax::{
    SyntaxNodeExt, SyntaxTokenWithParent, TokenKind,
    ast::{self, AstNode},
    has_text_range::HasTextRange,
    match_ast, support,
    token::pair_token,
};

use crate::references::ReferenceCategory;

#[derive(Debug, Clone)]
pub struct DocumentHighlight {
    pub range: TextRange,
    pub category: ReferenceCategory,
}

impl DocumentHighlight {
    pub fn new(range: TextRange) -> Self {
        Self { range, category: ReferenceCategory::empty() }
    }
}

pub(crate) fn document_highlight(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> Option<Vec<DocumentHighlight>> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);

    let token = file.syntax().token_at_offset(offset).pick_bext_token(token_precedence)?;

    handle_ctrl_flow_kw(&sema, token)
}

fn handle_ctrl_flow_kw(
    sema: &Semantics<'_, RootDb>,
    SyntaxTokenWithParent { parent, tok }: SyntaxTokenWithParent,
) -> Option<Vec<DocumentHighlight>> {
    let kind = tok.kind();
    let file_id = sema.find_file(parent);
    let mut res = vec![DocumentHighlight::new(tok.text_range().unwrap())];

    let paired_kw = match pair_token(kind)? {
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
        res.push(DocumentHighlight::new(paired_kw.text_range().unwrap()));
    }

    Some(res)
}

fn token_precedence(kind: TokenKind) -> usize {
    match kind {
        _ if pair_token(kind).is_some() => 4,
        _ => 1,
    }
}
