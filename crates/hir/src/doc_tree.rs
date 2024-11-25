use la_arena::{Arena, Idx};
use syntax::{
    SyntaxElemPreorder, SyntaxNode, SyntaxNodeExt, SyntaxToken, SyntaxTrivia,
    token::SyntaxTokenExt, trivia::TriviaExt,
};
use utils::text_edit::{TextRange, TextSize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum DocKind {
    Region,
    Segment { doc: Option<Idx<DocNode>> },
    Doc,
}

// 1. region .. endregion
// 2. comment (optional) ...more than one line declarators
#[derive(Default, Debug, PartialEq, Eq)]
pub struct DocTree {
    pub roots: Vec<Idx<DocNode>>,
    pub nodes: Arena<DocNode>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct DocNode {
    pub range: TextRange,
    pub kind: DocKind,
    pub children: Vec<Idx<DocNode>>,
}

impl DocTree {
    pub fn add_node(
        &mut self,
        range: TextRange,
        kind: DocKind,
        parent: Option<Idx<DocNode>>,
    ) -> Idx<DocNode> {
        let idx = self.nodes.alloc(DocNode { range, kind, children: Vec::new() });
        if let Some(parent) = parent {
            self.nodes[parent].children.push(idx);
        } else {
            self.roots.push(idx);
        }
        idx
    }
}

#[derive(Debug)]
pub(crate) struct DocTreeBuilder {
    tree: DocTree,
    stack: Vec<Idx<DocNode>>,
}

impl DocTreeBuilder {
    pub fn new() -> Self {
        Self { tree: DocTree::default(), stack: Vec::new() }
    }

    fn open_node(&mut self, start: usize, kind: DocKind) {
        let parent = self.stack.last().copied();
        let range = TextRange::empty(TextSize::new(start as u32));
        let node = self.tree.add_node(range, kind, parent);
        self.stack.push(node);
    }

    fn finish_node(&mut self, end: usize) {
        let node = &mut self.tree.nodes[*self.stack.last().unwrap()];
        let start = node.range.start();
        let end = TextSize::new(end as u32);
        node.range = TextRange::new(start, end);
        self.stack.pop();
    }

    fn add_node(&mut self, range: TextRange, kind: DocKind) {
        self.tree.add_node(range, kind, self.stack.last().copied());
    }

    // TODO: diagnostics for !is_empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn finish(self) -> DocTree {
        self.tree
    }

    pub fn handle_node(&mut self, node: SyntaxNode) {
        node.trivias_with_range().for_each(|(range, trivia)| self.handle_trivia(range, trivia));
    }

    pub fn handle_tok(&mut self, token: SyntaxToken) {
        token.trivias_with_range().for_each(|(range, trivia)| self.handle_trivia(range, trivia));
    }

    #[inline]
    fn handle_trivia(&mut self, range: TextRange, trivia: SyntaxTrivia) {
        if trivia.is_region_begin() {
            self.open_node(range.start().into(), DocKind::Region);
        } else if trivia.is_region_end() {
            self.finish_node(range.end().into());
        } else {
            // TODO: handle doc comments
        }
    }
}
