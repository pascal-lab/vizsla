use crate::Cancellable;
use base_db::{salsa, Cancelled};
use ide_db::line_index_db::LineIndexDb;
use ide_db::root_db::RootDb;
use line_index::LineIndex;
use triomphe::Arc;
use vfs::vfs::FileId;

#[derive(Debug)]
pub struct Analysis {
    pub(crate) db: salsa::Snapshot<RootDb>,
}

impl Analysis {
    fn with_db<F, T>(&self, f: F) -> Cancellable<T>
    where
        F: FnOnce(&RootDb) -> T + std::panic::UnwindSafe,
    {
        Cancelled::catch(|| f(&self.db))
    }

    pub fn line_index(&self, file_id: FileId) -> Cancellable<Arc<LineIndex>> {
        self.with_db(|db| db.line_index(file_id))
    }
}
