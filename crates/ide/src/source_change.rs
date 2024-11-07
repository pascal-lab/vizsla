use std::collections::hash_map::Entry;

use nohash_hasher::IntMap;
use utils::text_edit::TextEdit;
use vfs::FileId;

#[derive(Default, Debug, Clone)]
pub struct SourceChange {
    pub text_edits: IntMap<FileId, TextEdit>,
}

impl SourceChange {
    pub fn insert_text_edit(&mut self, file_id: FileId, edit: TextEdit) {
        match self.text_edits.entry(file_id) {
            Entry::Occupied(mut e) => e.get_mut().union(edit).unwrap(),
            Entry::Vacant(e) => {
                e.insert(edit);
            }
        }
    }
}
