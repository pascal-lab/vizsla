use std::iter;

use hir::semantics::Semantics;
use ide_db::root_db::RootDb;
use itertools::Itertools;
use line_index::{TextRange, TextSize};
use span::FilePosition;
use syntax::{
    SyntaxCursor, SyntaxCursorExt, SyntaxElement, SyntaxNode, SyntaxNodeExt, TokenKind,
    ast::AstNode,
    has_text_range::HasTextRange,
    token::{SyntaxTokenExt, TokenKindExt},
};

pub(crate) fn selection_ranges(
    db: &RootDb,
    FilePosition { file_id, offset }: FilePosition,
) -> Vec<TextRange> {
    let sema = Semantics::new(db);
    let root = sema.parse(file_id).syntax();

    let mut res = Vec::new();

    let mut cursor = root.walk();

    match root.token_at_offset(offset).pick_bext_token(token_precedence) {
        Some(token) => {
            cursor.goto_first_tok_after_or_last(token.text_range().unwrap().start());
        }
        None => {
            cursor.goto_first_tok_after_or_last(offset);
            let token = cursor.to_token().unwrap();
            let start = token.trivias_with_range().unwrap().next().unwrap().0.start();
            let end = token.text_range().unwrap().start();
            res.push(TextRange::new(start, end));
        }
    };

    let mut range = cursor.to_token().unwrap().text_range().unwrap();
    res.push(range);

    while cursor.goto_parent() {
        let new_range = cursor.to_elem().text_range().unwrap();
        if new_range != range {
            res.push(new_range);
            range = new_range
        }
    }

    res
}

fn token_precedence(kind: TokenKind) -> usize {
    match kind {
        _ if kind.name_like() => 4,
        _ => 1,
    }
}
