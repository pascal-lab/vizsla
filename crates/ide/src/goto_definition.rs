use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use itertools::Itertools;
use span::{FilePosition, RangeInfo};
use syntax::{
    ast::AstNode,
    syntax_kind,
    treesit_ext::{pick_best_token, token_at_offset},
};
use utils::text_edit::to_text_range;

use crate::{
    definitions::IdentClass,
    navigation_target::{NavTarget, ToNav},
};

pub(crate) fn goto_definition(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> Option<RangeInfo<Vec<NavTarget>>> {
    let sema = Semantics::new(db);
    let file = sema.parse(file_id);
    let token = pick_best_token(token_at_offset(file.syntax(), offset), token_precedence)?;
    let navs = IdentClass::classify(&sema, token)?
        .definitions()
        .into_iter()
        .map(|def| def.to_nav(db))
        .unique()
        .collect_vec();
    Some(RangeInfo::new(to_text_range(token.range()), navs))
}

fn token_precedence(kind: syntax_kind::SyntaxKindId) -> usize {
    match kind {
        syntax_kind::SIMPLE_IDENTIFIER => 4,
        _ => 1,
    }
}
