use rustc_hash::FxHashSet;
use syntax::SyntaxTree;
use triomphe::Arc;
use vfs::{FileId, anchored_path::AnchoredPath};

use crate::source_root::{SourceRoot, SourceRootId};

pub trait FileLoader {
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId>;
}

// Source code, syntax tree and project model.
// Everything else is derived from these queries.
#[salsa::query_group(SourceDbStorage)]
pub trait SourceDb: FileLoader + std::fmt::Debug {
    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> Arc<str>;

    fn parse_src(&self, file_id: FileId) -> SyntaxTree;

    #[salsa::input]
    fn files(&self) -> Box<FxHashSet<FileId>>;
}

fn parse_src(db: &dyn SourceDb, file_id: FileId) -> SyntaxTree {
    let text = db.file_text(file_id);
    // TODO: use meaningful path
    SyntaxTree::from_text(&text, "", "")
}

// Don't expose source roots to HIR, so extract them in a separate DB.
#[salsa::query_group(SourceRootDbStorage)]
pub trait SourceRootDb: SourceDb {
    #[salsa::input]
    fn source_root_id(&self, file_id: FileId) -> SourceRootId;

    #[salsa::input]
    fn source_root(&self, id: SourceRootId) -> Arc<SourceRoot>;
}
