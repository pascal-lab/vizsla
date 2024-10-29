use line_index::TextRange;
use slang::{
    SyntaxElement, SyntaxElementKind, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTokenWithParent,
    SyntaxTree, TokenKind,
};

use crate::{SyntaxNodeExt, has_text_range::HasTextRange};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxNodePtr {
    kind: SyntaxKind,
    range: TextRange,
}

impl SyntaxNodePtr {
    #[inline]
    pub fn from_node(node: SyntaxNode) -> SyntaxNodePtr {
        SyntaxNodePtr { kind: node.kind(), range: node.text_range().unwrap() }
    }

    #[inline]
    pub fn to_node<'a>(&self, tree: &'a SyntaxTree) -> Option<SyntaxNode<'a>> {
        let root_node = tree.root()?;
        root_node.elem_at_range(self.range)?.as_node()
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTokenPtr {
    kind: TokenKind,
    range: TextRange,
}

impl SyntaxTokenPtr {
    pub fn from_token(token: SyntaxToken) -> SyntaxTokenPtr {
        SyntaxTokenPtr { kind: token.kind(), range: token.text_range().unwrap() }
    }

    pub fn to_token<'a>(&self, tree: &'a SyntaxTree) -> Option<SyntaxToken<'a>> {
        tree.root()?.elem_at_range(self.range)?.as_token()
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SyntaxElementPtr {
    Node(SyntaxNodePtr),
    Token { parent: SyntaxNodePtr, tok: SyntaxTokenPtr },
}

impl SyntaxElementPtr {
    pub fn from_element(element: SyntaxElement) -> SyntaxElementPtr {
        match element {
            SyntaxElement::Node(node) => SyntaxElementPtr::Node(SyntaxNodePtr::from_node(node)),
            SyntaxElement::Token(SyntaxTokenWithParent { parent, tok }) => {
                SyntaxElementPtr::Token {
                    parent: SyntaxNodePtr::from_node(parent),
                    tok: SyntaxTokenPtr::from_token(tok),
                }
            }
        }
    }

    pub fn to_elem<'a>(&self, tree: &'a SyntaxTree) -> Option<SyntaxElement<'a>> {
        match self {
            SyntaxElementPtr::Node(node) => node.to_node(tree).map(SyntaxElement::from_node),
            SyntaxElementPtr::Token { parent, tok } => {
                let parent = parent.to_node(tree)?;
                let tok = tok.to_token(tree)?;
                Some(SyntaxElement::from_token(SyntaxTokenWithParent { parent, tok }))
            }
        }
    }

    pub fn kind(&self) -> SyntaxElementKind {
        match self {
            SyntaxElementPtr::Node(SyntaxNodePtr { kind, .. }) => SyntaxElementKind::Node(*kind),
            SyntaxElementPtr::Token { tok, .. } => SyntaxElementKind::Token(tok.kind),
        }
    }
}
