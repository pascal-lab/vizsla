use crate::FileId;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AnchoredPathBuf {
    pub anchor_id: FileId,
    pub path: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnchoredPath<'a> {
    pub anchor_id: FileId,
    pub path: &'a str,
}
