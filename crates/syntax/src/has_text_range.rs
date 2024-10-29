use line_index::TextRange;
use slang::{SourceRange, SyntaxElement, SyntaxNode, SyntaxToken};

pub trait SourceRangeExt {
    fn to_text_range(self) -> TextRange;
}

impl SourceRangeExt for SourceRange {
    #[inline]
    fn to_text_range(self) -> TextRange {
        let start = self.start() as u32;
        let end = self.end() as u32;
        TextRange::new(start.into(), end.into())
    }
}

pub trait HasTextRange {
    fn text_range(&self) -> Option<TextRange>;
}

impl HasTextRange for SyntaxNode<'_> {
    #[inline]
    fn text_range(&self) -> Option<TextRange> {
        Some(self.range()?.to_text_range())
    }
}

impl HasTextRange for SyntaxToken<'_> {
    #[inline]
    fn text_range(&self) -> Option<TextRange> {
        Some(self.range()?.to_text_range())
    }
}

impl HasTextRange for SyntaxElement<'_> {
    #[inline]
    fn text_range(&self) -> Option<TextRange> {
        Some(self.range()?.to_text_range())
    }
}
