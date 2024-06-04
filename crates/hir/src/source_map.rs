use std::hash::Hash;

use la_arena::{ArenaMap, Idx};
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceMap<Src, Hir>
where
    Src: PartialEq + Eq + Hash + Clone,
{
    pub src2hir: FxHashMap<Src, Idx<Hir>>,
    pub hir2src: ArenaMap<Idx<Hir>, Src>,
}

impl<Src, Hir> SourceMap<Src, Hir>
where
    Src: PartialEq + Eq + Hash + Clone,
{
    pub fn insert(&mut self, src: Src, idx: Idx<Hir>) {
        self.src2hir.insert(src.clone(), idx);
        self.hir2src.insert(idx, src);
    }

    pub fn get_idx(&self, src: &Src) -> Option<&Idx<Hir>> {
        self.src2hir.get(src)
    }

    pub fn get_src(&self, idx: Idx<Hir>) -> Option<&Src> {
        self.hir2src.get(idx)
    }
}

impl<Src, Hir> Default for SourceMap<Src, Hir>
where
    Src: PartialEq + Eq + Hash + Clone,
{
    fn default() -> Self {
        SourceMap { src2hir: FxHashMap::default(), hir2src: ArenaMap::default() }
    }
}
