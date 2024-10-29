use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use itertools::Itertools;
use span::{FilePosition, RangeInfo};
use syntax::{ast::AstNode, has_text_range::HasTextRange, SyntaxNodeExt, TokenKind};

use crate::{
    definitions::Definition,
    navigation_target::{NavTarget, ToNav},
};

pub(crate) fn goto_definition(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> Option<RangeInfo<Vec<NavTarget>>> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = file.syntax().token_at_offset(offset).pick_bext_token(token_precedence)?;
    let navs = Definition::resolution(&sema, token)?
        .into_iter()
        .map(|def| def.to_nav(db))
        .unique()
        .collect_vec();
    Some(RangeInfo::new(token.text_range()?, navs))
}

fn token_precedence(kind: TokenKind) -> usize {
    match kind {
        TokenKind::IDENTIFIER | TokenKind::SYSTEM_IDENTIFIER => 4,
        _ => 1,
    }
}
