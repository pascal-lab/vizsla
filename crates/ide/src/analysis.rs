use base_db::{Cancelled, salsa};
use ide_db::{line_index_db::LineIndexDb, root_db::RootDb};
use line_index::LineIndex;
use span::{FilePosition, RangeInfo};
use triomphe::Arc;
use vfs::FileId;

use crate::{
    document_highlight::{self, DocumentHighlight}, document_symbols::{self, DocumentSymbol}, goto_definition, navigation_target::NavTarget, references::{self, references, References}, Cancellable
};

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

impl Analysis {
    pub fn goto_definition(
        &self,
        position: FilePosition,
    ) -> Cancellable<Option<RangeInfo<Vec<NavTarget>>>> {
        self.with_db(|db| goto_definition::goto_definition(db, position))
    }

    pub fn document_symbol(&self, file_id: FileId) -> Cancellable<Vec<DocumentSymbol>> {
        self.with_db(|db| document_symbols::document_symbols(db, file_id))
    }

    pub fn document_highlight(
        &self,
        position: FilePosition,
    ) -> Cancellable<Option<Vec<DocumentHighlight>>> {
        self.with_db(|db| document_highlight::document_highlight(db, position))
    }

    pub fn references(
        &self,
        position: FilePosition,
    ) -> Cancellable<Option<Vec<References>>> {
        self.with_db(|db| references::references(db, position))
    }
}
