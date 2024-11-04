use nohash_hasher::IntMap;
use utils::text_edit::TextEdit;
use vfs::FileId;

#[derive(Default, Debug, Clone)]
pub struct SourceChange {
    pub text_edits: IntMap<FileId, TextEdit>,
}

impl Extend<(FileId, TextEdit)> for SourceChange {
    fn extend<T: IntoIterator<Item = (FileId, TextEdit)>>(&mut self, iter: T) {
        self.text_edits.extend(iter.into_iter());
    }
}
