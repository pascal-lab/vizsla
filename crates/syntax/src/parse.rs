use crate::SyntaxNode;

#[derive(Debug, Clone)]
pub struct SyntaxTree(tree_sitter::Tree);

impl PartialEq for SyntaxTree {
    fn eq(&self, other: &Self) -> bool {
        self.0.root_node().id() == other.0.root_node().id()
    }
}

impl Eq for SyntaxTree {}

impl SyntaxTree {
    pub fn new(tree: tree_sitter::Tree) -> Self {
        Self(tree)
    }

    pub fn tree(&self) -> &tree_sitter::Tree {
        &self.0
    }

    pub fn root_node(&self) -> SyntaxNode {
        self.0.root_node()
    }
}
