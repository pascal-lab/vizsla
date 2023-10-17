#[derive(Clone, Copy)]
pub enum PositionEncoding {
    Utf8,
    Wide(WideEncoding),
}

/// A kind of wide character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WideEncoding {
    Utf16,
    Utf32,
}
