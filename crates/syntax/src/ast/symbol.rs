mod generated;

use crate::{
    ast::{AstNode, SyntaxNode},
    syntax_kind,
};

pub use generated::*;

pub struct ErrorNode<'a> {
    syntax: SyntaxNode<'a>,
}

impl<'a> AstNode<'a> for ErrorNode<'a> {
    fn can_cast(kind_id: syntax_kind::SyntaxKindId) -> bool {
        kind_id == syntax_kind::ERROR
    }

    fn cast(syntax: SyntaxNode<'a>) -> Option<Self> {
        Self::can_cast(syntax.kind_id()).then_some(ErrorNode { syntax })
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
