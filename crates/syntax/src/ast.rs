pub mod ptr;
pub mod support;
mod symbol;
pub use symbol::*;

use crate::{syntax_kind, SyntaxNode};

pub trait AstNode<'a> {
    fn can_cast(kind_id: syntax_kind::SyntaxKindId) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode<'a>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &'a SyntaxNode;

    fn to_text(&'a self, text: &'a str) -> Option<&'a str> {
        self.syntax().utf8_text(text.as_bytes()).ok()
    }

    fn errors(&'a self) -> support::AstChildren<'a, symbol::ErrorNode<'a>> {
        support::children(self.syntax())
    }

    fn comments(&'a self) -> support::AstChildren<'a, symbol::Comment<'a>> {
        support::children(self.syntax())
    }
}
