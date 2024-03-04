use std::sync::Arc;

use crate::{
    hir_def::{
        self,
        module::{ModuleDecl, ModuleSourceMap},
        FileSourceMap, HirFile,
    },
    HirFileId,
};
use base_db::source_db::SourceDb;
use vfs::vfs::FileId;

#[salsa::query_group(HirDbStorage)]
pub trait HirDb: SourceDb {
    fn hir_file_id(&self, file_id: FileId) -> HirFileId;

    #[salsa::invoke(hir_def::hir_file_with_source_map_query)]
    fn hir_file_with_source_map(&self, file_id: HirFileId) -> (Arc<HirFile>, Arc<FileSourceMap>);

    fn hir_file(&self, file_id: HirFileId) -> Arc<HirFile>;

    // fn module_with_source_map(&self, module_id: ModuleId) -> (Arc<ModuleDecl>, Arc<ModuleSourceMap>);
}

pub fn hir_file_id(_db: &dyn HirDb, file_id: FileId) -> HirFileId {
    HirFileId(file_id)
}

pub fn hir_file(db: &dyn HirDb, file_id: HirFileId) -> Arc<HirFile> {
    db.hir_file_with_source_map(file_id).0
}
