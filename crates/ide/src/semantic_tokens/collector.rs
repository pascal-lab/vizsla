use std::iter;

use utils::text_edit::TextRange;

use super::SemaToken;

#[derive(Debug)]
pub(super) struct SemaTokenCollectorTree {
    root: Node,
}

#[derive(Debug)]
struct Node {
    tok: SemaToken,
    nested: Vec<Node>,
}

impl SemaTokenCollectorTree {
    pub(super) fn new(tok: SemaToken) -> SemaTokenCollectorTree {
        SemaTokenCollectorTree { root: Node::new(tok) }
    }

    pub(super) fn add(&mut self, tok: SemaToken) {
        self.root.add(tok);
    }

    pub(super) fn finish(&self) -> Vec<SemaToken> {
        let mut res = Vec::new();
        self.root.flatten(&mut res);
        res
    }
}

impl Node {
    fn new(tok: SemaToken) -> Node {
        Node { tok, nested: Vec::new() }
    }

    fn add(&mut self, tok: SemaToken) {
        assert!(self.tok.range.contains_range(tok.range));

        if let Some(last) = self.nested.last_mut() {
            if last.tok.range.contains_range(tok.range) {
                last.add(tok);
                return;
            }
            if last.tok.range.end() <= tok.range.start() {
                return self.nested.push(Node::new(tok));
            }
        }

        let overlapping = {
            let start = self.nested.partition_point(|it| it.tok.range.end() <= tok.range.start());
            let len = self.nested[start..]
                .partition_point(|it| it.tok.range.intersect(tok.range).is_some());
            start..start + len
        };

        if overlapping.len() == 1
            && self.nested[overlapping.start].tok.range.contains_range(tok.range)
        {
            return self.nested[overlapping.start].add(tok);
        }

        let nested = self.nested.splice(overlapping.clone(), iter::once(Node::new(tok))).collect();
        self.nested[overlapping.start].nested = nested;
    }

    fn flatten(&self, acc: &mut Vec<SemaToken>) {
        let SemaToken { range, tag, mods } = self.tok;
        let mut start = range.start();
        let mut nested = self.nested.iter();
        loop {
            let next = nested.next();
            let end = next.map_or(range.end(), |it| it.tok.range.start());
            if start < end {
                acc.push(SemaToken { range: TextRange::new(start, end), tag, mods });
            }
            start = match next {
                Some(child) => {
                    child.flatten(acc);
                    child.tok.range.end()
                }
                None => break,
            }
        }
    }
}
