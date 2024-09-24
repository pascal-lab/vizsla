use base_db::{salsa, source_db::SourceDb};
use line_index::LineIndex;
use triomphe::Arc;
use vfs::FileId;

#[salsa::query_group(LineIndexDbStorage)]
pub trait LineIndexDb: SourceDb {
    fn line_index(&self, file_id: FileId) -> Arc<LineIndex>;
}

fn line_index(db: &dyn LineIndexDb, file_id: FileId) -> Arc<LineIndex> {
    let text = db.file_text(file_id);
    Arc::new(LineIndex::new(&text))
}
