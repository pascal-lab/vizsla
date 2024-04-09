use base_db::{change::Change, salsa::ParallelDatabase, source_db::SourceDb};
use ide_db::root_db::RootDb;
use hir::{db::HirDb, hir_def::FileItem, HirFileId, InFile};

use crate::analysis::Analysis;

pub struct AnalysisHost {
    db: RootDb,
}

impl AnalysisHost {
    pub fn new(lru_capacity: Option<usize>) -> AnalysisHost {
        AnalysisHost { db: RootDb::new(lru_capacity) }
    }

    pub fn make_analysis(&self) -> Analysis {
        Analysis { db: self.db.snapshot() }
    }

    pub fn apply_change(&mut self, change: Change) {
        self.db.apply_change(change);
        let file_id = self.db.files().iter().next().unwrap().clone().into();
        dbg!(self.db.syntax_tree(file_id));
        dbg!(self.db.hir_file(file_id.into()));
        let x: FileItem = self.db.hir_file(file_id.into()).items.clone().first().unwrap().clone();
        let x = match x {
            FileItem::Module(x) => x,
            _ => panic!(),
        };
        dbg!(self.db.module_with_source_map(InFile::new(file_id.into(), x)));
    }

    pub fn raw_db(&self) -> &RootDb {
        &self.db
    }

    pub fn raw_db_mut(&mut self) -> &mut RootDb {
        &mut self.db
    }
}

impl Default for AnalysisHost {
    fn default() -> AnalysisHost {
        AnalysisHost::new(None)
    }
}
