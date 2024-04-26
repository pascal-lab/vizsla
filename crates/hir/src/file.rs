use vfs::vfs::FileId;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct HirFileId(pub FileId);

impl From<FileId> for HirFileId {
    fn from(file_id: FileId) -> HirFileId {
        HirFileId(file_id)
    }
}

// Although `InFile` is similar to `InContainer`, we do not merge them because
// they are used in different contexts. `InFile` is used to represent a value
// that is associated with a file, while `InContainer` is used to represent a
// value that is associated with a container (semantically).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InFile<T> {
    pub file_id: HirFileId,
    pub value: T,
}

impl<T> InFile<T> {
    pub fn new(file_id: HirFileId, value: T) -> InFile<T> {
        InFile { file_id, value }
    }

    pub fn with_value<U>(&self, value: U) -> InFile<U> {
        InFile::new(self.file_id, value)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InFile<U> {
        InFile::new(self.file_id, f(self.value))
    }

    pub fn as_ref(&self) -> InFile<&T> {
        self.with_value(&self.value)
    }
}

impl<T: Clone> InFile<&T> {
    pub fn cloned(&self) -> InFile<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InFile<Option<T>> {
    pub fn transpose(self) -> Option<InFile<T>> {
        Some(InFile::new(self.file_id, self.value?))
    }
}
