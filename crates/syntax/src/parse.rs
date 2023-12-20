use triomphe::Arc;
use utils::text_edit::{TextRange, TextSize};

use crate::SyntaxNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxError(String, TextRange);

impl SyntaxError {
    pub fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self(message.into(), range)
    }
    pub fn new_at_offset(message: impl Into<String>, offset: TextSize) -> Self {
        Self(message.into(), TextRange::empty(offset))
    }

    pub fn range(&self) -> TextRange {
        self.1
    }
}

#[derive(Debug)]
pub struct SyntaxTree(tree_sitter::Tree);

impl PartialEq for SyntaxTree {
    fn eq(&self, other: &Self) -> bool {
        self.0.root_node().id() == other.0.root_node().id()
    }
}

impl Eq for SyntaxTree {}

pub type SyntaxErrors = Arc<[SyntaxError]>;

// `Parse` is the result of parsing
#[derive(Debug, PartialEq, Eq)]
pub struct Parse {
    tree: SyntaxTree,
    errors: Option<Arc<[SyntaxError]>>,
}

impl Parse {
    fn new(tree: tree_sitter::Tree, errors: Vec<SyntaxError>) -> Parse {
        Parse {
            tree: SyntaxTree(tree),
            errors: if errors.is_empty() { None } else { Some(errors.into()) },
        }
    }

    pub fn tree(&self) -> &tree_sitter::Tree {
        &self.tree.0
    }

    pub fn root_node(&self) -> SyntaxNode {
        self.tree.0.root_node()
    }

    pub fn errors(&self) -> &[SyntaxError] {
        self.errors.as_deref().unwrap_or_default()
    }

    pub fn ok(&self) -> Result<&SyntaxTree, SyntaxErrors> {
        match self.errors.as_ref() {
            Some(e) => Err(e.clone()),
            None => Ok(&self.tree),
        }
    }
}
