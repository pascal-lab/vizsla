use base_db::{impl_intern_key, impl_intern_lookup, salsa, source_db::SourceDb};
use syntax::parse::SyntaxTree;
use triomphe::Arc;

use crate::{
    file::HirFileId,
    hir_def::{
        self,
        block::{self, Block, BlockId, BlockLoc, BlockSourceMap},
        module::{self, Module, ModuleSourceMap},
        FileSourceMap, HirFile, ModuleId,
    },
    scope::{BlockScope, ModuleScope, UnitScope},
};

#[macro_export]
macro_rules! impl_intern {
    ($id:ident, $loc:ident, $intern:ident, $lookup:ident) => {
        impl_intern_key!($id);
        impl_intern_lookup!(InternDb, $id, $loc, $intern, $lookup);
    };
}

#[salsa::query_group(InternDbStorage)]
pub trait InternDb: SourceDb {
    #[salsa::interned]
    fn intern_block(&self, loc: BlockLoc) -> BlockId;
}

impl_intern!(BlockId, BlockLoc, intern_block, lookup_intern_block);

#[salsa::query_group(HirDbStorage)]
pub trait HirDb: InternDb {
    fn hir_syntax_tree(&self, file_id: HirFileId) -> Option<Arc<SyntaxTree>>;

    fn hir_file_text(&self, file_id: HirFileId) -> Arc<str>;

    #[salsa::invoke(hir_def::hir_file_with_source_map_query)]
    fn hir_file_with_source_map(&self, file_id: HirFileId) -> (Arc<HirFile>, Arc<FileSourceMap>);

    fn hir_file(&self, file_id: HirFileId) -> Arc<HirFile>;

    #[salsa::invoke(module::module_with_source_map_query)]
    fn module_with_source_map(&self, module_id: ModuleId) -> (Arc<Module>, Arc<ModuleSourceMap>);

    fn module(&self, module_id: ModuleId) -> Arc<Module>;

    #[salsa::invoke(block::block_with_source_map_query)]
    fn block_with_source_map(&self, block_id: BlockId) -> (Arc<Block>, Arc<BlockSourceMap>);

    fn block(&self, block_id: BlockId) -> Arc<Block>;

    #[salsa::invoke(UnitScope::unit_scope_query)]
    fn unit_scope(&self) -> Arc<UnitScope>;

    #[salsa::invoke(ModuleScope::module_scope_query)]
    fn module_scope(&self, module_id: ModuleId) -> Arc<ModuleScope>;

    #[salsa::invoke(BlockScope::block_scope_query)]
    fn block_scope(&self, block_id: BlockId) -> Arc<BlockScope>;
}

pub fn hir_syntax_tree(db: &dyn HirDb, file_id: HirFileId) -> Option<Arc<SyntaxTree>> {
    db.syntax_tree(file_id.0)
}

pub fn hir_file_text(db: &dyn HirDb, file_id: HirFileId) -> Arc<str> {
    db.file_text(file_id.0)
}

fn hir_file(db: &dyn HirDb, file_id: HirFileId) -> Arc<HirFile> {
    db.hir_file_with_source_map(file_id).0
}

fn module(db: &dyn HirDb, module_id: ModuleId) -> Arc<Module> {
    db.module_with_source_map(module_id).0
}

fn block(db: &dyn HirDb, block_id: BlockId) -> Arc<Block> {
    db.block_with_source_map(block_id).0
}
