use crate::FileId;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnchoredPathBuf {
    pub anchor: FileId,
    pub path: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnchoredPath<'a> {
    pub anchor: FileId,
    pub path: &'a str,
}
