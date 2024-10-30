use line_index::{TextRange, TextSize};
use slang::{
    SyntaxAncestors, SyntaxCursor, SyntaxElement, SyntaxNode, SyntaxTokenWithParent, TokenKind,
    ast::AstNode,
};

use crate::{has_text_range::HasTextRange, ptr::SyntaxNodePtr};

pub mod token;

#[derive(Clone, Debug)]
pub enum TokenAtOffset<'a> {
    None,
    Single(SyntaxTokenWithParent<'a>),
    Between(SyntaxTokenWithParent<'a>, SyntaxTokenWithParent<'a>),
}

impl<'a> TokenAtOffset<'a> {
    pub fn pick_bext_token(
        self,
        f: impl Fn(TokenKind) -> usize,
    ) -> Option<SyntaxTokenWithParent<'a>> {
        match self {
            TokenAtOffset::None => None,
            TokenAtOffset::Single(n) => Some(n),
            TokenAtOffset::Between(a, b) => {
                if f(a.kind()) > f(b.kind()) {
                    Some(a)
                } else {
                    Some(b)
                }
            }
        }
    }
}

pub trait SyntaxNodeExt<'a> {
    fn elem_at_range(&self, range: TextRange) -> Option<SyntaxElement<'a>>;
    fn token_at_offset(&self, offset: TextSize) -> TokenAtOffset<'a>;
    fn find_root(&self) -> SyntaxNode<'a>;
    fn to_ptr(&self) -> SyntaxNodePtr;
}

impl<'a> SyntaxNodeExt<'a> for SyntaxNode<'a> {
    fn elem_at_range(&self, range: TextRange) -> Option<SyntaxElement<'a>> {
        let start = range.start();
        let start_offset: usize = start.into();
        let end = range.end();

        let mut cursor = self.walk();
        loop {
            let elem = cursor.to_elem();
            let range = elem.text_range()?;

            if !(range.contains_inclusive(start) && range.contains_inclusive(end)) {
                return None;
            }

            if range.start() == start && range.end() == end {
                return Some(elem);
            }

            cursor.goto_first_child_after_pos(start_offset);
        }
    }

    fn token_at_offset(&self, offset: TextSize) -> TokenAtOffset<'a> {
        let range = self.text_range().unwrap();
        if range.is_empty() || !(range.contains(offset)) {
            return TokenAtOffset::None;
        }

        let mut cursor = self.walk();
        cursor.goto_last_token_before_pos(offset);
        let left = cursor.to_tok_with_parent().unwrap();
        let left_range = left.text_range().unwrap();

        cursor.reset(*self);
        cursor.goto_first_token_after_pos(offset);
        let right = cursor.to_tok_with_parent().unwrap();
        let right_range = right.text_range().unwrap();

        if left_range.end() == right_range.start() {
            debug_assert!(left_range.contains_inclusive(offset));
            debug_assert!(right_range.contains(offset));
            TokenAtOffset::Between(left, right)
        } else if left_range.contains_inclusive(offset) {
            TokenAtOffset::Single(left)
        } else if right_range.contains(offset) {
            TokenAtOffset::Single(right)
        } else {
            debug_assert!(left_range.end() < offset);
            debug_assert!(right_range.start() > offset);
            TokenAtOffset::None
        }
    }

    #[inline]
    fn find_root(&self) -> SyntaxNode<'a> {
        SyntaxAncestors::start_from(*self).last().unwrap()
    }

    #[inline]
    fn to_ptr(&self) -> SyntaxNodePtr {
        SyntaxNodePtr::from_node(*self)
    }
}

pub mod support {
    use slang::{ast::AstNode, SyntaxNode, SyntaxToken, TokenKind};

    #[inline]
    pub fn child<'a, N: AstNode<'a>>(parent: SyntaxNode<'a>) -> Option<N> {
        parent.children().filter_map(|elem| elem.as_node()).find_map(N::cast)
    }

    #[inline]
    pub fn child_token<'a>(
        parent: SyntaxNode<'a>,
        kind: TokenKind,
    ) -> Option<SyntaxToken<'a>> {
        parent
            .children()
            .filter_map(|elem| elem.as_token())
            .find(|tok| tok.kind() == kind)
    }
}

pub trait SyntaxCursorExt {
    fn goto_first_token_after_pos(&mut self, offset: TextSize) -> bool;
    fn goto_last_token_before_pos(&mut self, offset: TextSize) -> bool;
}

impl SyntaxCursorExt for SyntaxCursor<'_> {
    fn goto_first_token_after_pos(&mut self, offset: TextSize) -> bool {
        let offset: usize = offset.into();
        let Some(end) = self.to_elem().range().map(|range| range.end()) else {
            return false;
        };
        if end <= offset {
            return false;
        }
        while self.to_node().is_some() {
            let success = self.goto_first_child_after_pos(offset);
            debug_assert!(success);
        }
        debug_assert!(self.to_token().is_some());
        true
    }

    fn goto_last_token_before_pos(&mut self, offset: TextSize) -> bool {
        let offset: usize = offset.into();
        let Some(start) = self.to_elem().range().map(|range| range.start()) else {
            return false;
        };
        if start >= offset {
            return false;
        }

        while self.to_node().is_some() {
            let success = self.goto_last_child_before_pos(offset);
            debug_assert!(success);
        }
        debug_assert!(self.to_token().is_some());
        true
    }
}

pub trait AstNodeExt {
    fn to_ptr(&self) -> SyntaxNodePtr;
}

impl<'a, T> AstNodeExt for T
where
    T: AstNode<'a>,
{
    #[inline]
    fn to_ptr(&self) -> SyntaxNodePtr {
        SyntaxNodePtr::from_node(self.syntax())
    }
}
