use std::sync::Arc;

use crate::{
    hir_def::{self, HirFile, NodeIdMap},
    HirFileId,
};
use base_db::{salsa, source_db::SourceRootDb};
use vfs::vfs::FileId;

#[salsa::query_group(HirDbStorage)]
pub trait HirDb: SourceRootDb {
    fn hir_file_id(&self, file_id: FileId) -> HirFileId;

    #[salsa::invoke(hir_def::hir_file_with_source_map_query)]
    fn hir_file_with_source_map(&self, file_id: HirFileId) -> (Arc<HirFile>, Arc<NodeIdMap>);

    fn hir_file(&self, file_id: HirFileId) -> Arc<HirFile>;

    // fn module_with_source_map(&self, module_id: ModuleId) -> (Arc<ModuleData>, Arc<ModuleSourceMap>);
}

pub fn hir_file_id(_db: &dyn HirDb, file_id: FileId) -> HirFileId {
    file_id.0
}

pub fn hir_file(db: &dyn HirDb, file_id: HirFileId) -> Arc<HirFile> {
    db.hir_file_with_source_map(file_id).0
}
