use std::sync::Arc;

use crate::{
    hir_def::{self, FileItems, NodeIdMap},
    HirFileId,
};
use base_db::{salsa, source_db::SourceRootDb, DbUpcast};
use vfs::vfs::FileId;

#[salsa::query_group(HirDbStorage)]
pub trait HirDb: SourceRootDb + DbUpcast<dyn SourceRootDb> {
    fn hir_file_id(&self, file_id: FileId) -> HirFileId;

    #[salsa::invoke(hir_def::file_items_with_source_map_query)]
    fn file_items_with_source_map(&self, file_id: HirFileId) -> (Arc<FileItems>, Arc<NodeIdMap>);

    fn file_items(&self, file_id: HirFileId) -> Arc<FileItems>;

    // fn module_with_source_map(&self, module_id: ModuleId) -> (Arc<ModuleData>, Arc<ModuleSourceMap>);
}

pub fn hir_file_id(db: &dyn HirDb, file_id: FileId) -> HirFileId {
    file_id.0
}

pub fn file_items(db: &dyn HirDb, file_id: HirFileId) -> Arc<FileItems> {
    db.file_items_with_source_map(file_id).0
}
