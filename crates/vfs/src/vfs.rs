use std::{fmt, hash::BuildHasherDefault, mem};

use crate::vfs_path::VfsPath;
use indexmap::IndexSet;
use rustc_hash::FxHasher;
use utils::text_edit::SourceEdit;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

impl nohash_hasher::IsEnabled for FileId {}

#[derive(Default)]
pub struct Vfs {
    paths: IndexSet<VfsPath, BuildHasherDefault<FxHasher>>,
    data: Vec<Option<Vec<u8>>>,
    changes: Vec<ChangedFile>,
}

#[derive(Debug)]
pub struct ChangedFile {
    pub file_id: FileId,
    pub change_kind: ChangeKind,
}

impl ChangedFile {
    pub fn exists(&self) -> bool {
        self.change_kind != ChangeKind::Delete
    }

    pub fn is_created_or_deleted(&self) -> bool {
        matches!(self.change_kind, ChangeKind::Create | ChangeKind::Delete)
    }

    pub fn is_created_or_modified(&self) -> bool {
        matches!(self.change_kind, ChangeKind::Create | ChangeKind::Modify(_))
    }

    pub fn source_edits(&self) -> Option<&Vec<SourceEdit>> {
        match self.change_kind {
            ChangeKind::Create | ChangeKind::Delete => None,
            ChangeKind::Modify(Some(ref edits)) => Some(edits),
            ChangeKind::Modify(None) => None,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChangeKind {
    Create,
    // None means modifications is unknown, so we need to update the whole text
    Modify(Option<Vec<SourceEdit>>),
    Delete,
}

impl Vfs {
    pub fn file_id(&self, path: &VfsPath) -> Option<FileId> {
        self.paths.get_index_of(path).map(|id| FileId(id as u32))
    }

    pub fn file_path(&self, file_id: FileId) -> &VfsPath {
        self.paths.get_index(file_id.0 as usize).unwrap()
    }

    pub fn file_contents(&self, file_id: FileId) -> Option<&[u8]> {
        self.get_file_contents(file_id).as_deref()
    }

    pub fn memory_usage(&self) -> usize {
        self.data.iter().flatten().map(|d| d.capacity()).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (FileId, &VfsPath)> + '_ {
        (0..self.data.len())
            .map(|it| FileId(it as u32))
            .filter(move |&file_id| self.exists(file_id))
            .map(move |file_id| (file_id, self.file_path(file_id)))
    }

    pub fn set_file_contents(
        &mut self,
        path: VfsPath,
        mut contents: Option<Vec<u8>>,
        source_edits: Option<Vec<SourceEdit>>,
    ) {
        let file_id = self.file_id_or_alloc(path);
        let change_kind = match (self.get_file_contents(file_id), &mut contents) {
            (None, None) => return,
            (None, Some(_)) => ChangeKind::Create,
            (Some(_), None) => ChangeKind::Delete,
            // TODO: should we add this redundant comparison?
            // (Some(old), Some(new)) if old == new => return,
            (Some(_), Some(_)) => ChangeKind::Modify(source_edits),
        };

        if let ChangeKind::Modify(Some(edits)) = &change_kind
            && edits.is_empty()
        {
            return;
        }

        *self.get_file_contents_mut(file_id) = contents;
        self.changes.push(ChangedFile { file_id, change_kind });
    }

    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    pub fn take_changes(&mut self) -> Vec<ChangedFile> {
        mem::take(&mut self.changes)
    }

    pub fn exists(&self, file_id: FileId) -> bool {
        self.get_file_contents(file_id).is_some()
    }

    fn file_id_or_alloc(&mut self, path: VfsPath) -> FileId {
        let (id, _) = self.paths.insert_full(path);
        assert!(id < u32::MAX as usize);

        let len = self.data.len().max(id + 1);
        self.data.resize(len, None);
        FileId(id as u32)
    }

    fn get_file_contents(&self, file_id: FileId) -> &Option<Vec<u8>> {
        &self.data[file_id.0 as usize]
    }

    fn get_file_contents_mut(&mut self, file_id: FileId) -> &mut Option<Vec<u8>> {
        &mut self.data[file_id.0 as usize]
    }
}

impl fmt::Debug for Vfs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs").field("n_files", &self.data.len()).finish()
    }
}
