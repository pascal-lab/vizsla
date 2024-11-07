use std::{cell::RefCell, ops};

use itertools::Itertools;
use rustc_hash::FxHashMap;
use source_to_def::{Source2DefCache, Source2DefCtx};
use syntax::{
    SyntaxNode, SyntaxNodeExt,
    ast::{self, AstNode},
};
use vfs::FileId;

use crate::{db::HirDb, file::HirFileId};

pub mod pathres;
mod source_to_def;

pub struct Semantics<'db, DB> {
    pub db: &'db DB,
    impl_: SemanticsImpl<'db>,
}

impl<DB: HirDb> Semantics<'_, DB> {
    pub fn new(db: &DB) -> Semantics<'_, DB> {
        let impl_ = SemanticsImpl::new(db);
        Semantics { db, impl_ }
    }
}

impl<'db, DB> ops::Deref for Semantics<'db, DB> {
    type Target = SemanticsImpl<'db>;

    fn deref(&self) -> &Self::Target {
        &self.impl_
    }
}

pub struct SemanticsImpl<'db> {
    pub db: &'db dyn HirDb,

    // s2d_cache
    // Root -> HirFileId
    root2file_cache: RefCell<FxHashMap<SyntaxNode<'db>, HirFileId>>,
    source2def_cache: RefCell<Source2DefCache<'db>>,
}

impl<'db> SemanticsImpl<'db> {
    fn new(db: &'db dyn HirDb) -> Self {
        SemanticsImpl {
            db,
            root2file_cache: Default::default(),
            source2def_cache: Default::default(),
        }
    }

    pub fn parse(&self, file_id: FileId) -> ast::CompilationUnit {
        let tree = self.db.parse_src(file_id);

        // Unsafe: we garentee that the root node is valid for the lifetime of the db
        let root_node =
            unsafe { std::mem::transmute::<SyntaxNode<'_>, SyntaxNode<'db>>(tree.root().unwrap()) };
        self.cache_node2file(root_node, file_id.into());
        ast::CompilationUnit::cast(root_node).unwrap()
    }

    pub fn find_file(&self, node: SyntaxNode) -> HirFileId {
        let root_node = node.find_root();
        self.lookup_file_id(root_node).unwrap_or_else(|| {
            panic!(
                "\n\nFailed to lookup {:?}.\nroot node:   {:?}\nknown nodes: {}\n\n",
                node,
                root_node,
                self.root2file_cache.borrow().keys().map(|it| format!("{it:?}")).join(", ")
            )
        })
    }

    fn cache_node2file(&self, root_node: SyntaxNode<'db>, file_id: HirFileId) {
        debug_assert!(root_node.parent().is_none());
        let mut cache = self.root2file_cache.borrow_mut();
        let prev = cache.insert(root_node, file_id);
        debug_assert!(prev.is_none() || prev == Some(file_id))
    }

    fn lookup_file_id(&self, root_node: SyntaxNode) -> Option<HirFileId> {
        let cache = self.root2file_cache.borrow();
        cache.get(&root_node).copied()
    }

    fn with_ctx<F: FnOnce(&mut Source2DefCtx<'_, '_>) -> T, T>(&self, f: F) -> T {
        let mut ctx = Source2DefCtx { db: self.db, cache: &mut self.source2def_cache.borrow_mut() };
        f(&mut ctx)
    }
}
