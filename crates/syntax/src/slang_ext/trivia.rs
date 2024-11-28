use core::str;

use slang::{SyntaxTrivia, Trivia, TriviaKind};
use smol_str::{SmolStr, ToSmolStr};

pub trait TriviaKindExt {
    fn is_whitespace(&self) -> bool;
    fn is_eol(&self) -> bool;
    fn is_comment(&self) -> bool;
}

impl TriviaKindExt for TriviaKind {
    fn is_whitespace(&self) -> bool {
        matches!(*self, Trivia![ws])
    }

    fn is_eol(&self) -> bool {
        matches!(*self, Trivia![eol])
    }

    fn is_comment(&self) -> bool {
        matches!(*self, Trivia![bc] | Trivia![lc])
    }
}

pub trait TriviaExt {
    fn is_region_begin(&self) -> Option<SmolStr>;
    fn is_region_end(&self) -> bool;
}

const REGION_BEGIN: &str = "region";
const REGION_END: &str = "endregion";

impl TriviaExt for SyntaxTrivia<'_> {
    #[inline]
    fn is_region_begin(&self) -> Option<SmolStr> {
        if !matches!(self.kind(), Trivia![lc]) {
            return None;
        }

        let bytes = self.get_raw_text().as_bytes();
        debug_assert!(str::from_utf8(bytes).is_ok());

        let text = unsafe { str::from_utf8_unchecked(bytes) };
        let captions = text
            .strip_prefix("//")
            .unwrap()
            .trim()
            .strip_prefix(REGION_BEGIN)?
            .strip_prefix(":")
            .unwrap_or(text)
            .trim();
        Some(captions.to_smolstr())
    }

    #[inline]
    fn is_region_end(&self) -> bool {
        if !matches!(self.kind(), Trivia![lc]) {
            return false;
        }

        let bytes = self.get_raw_text().as_bytes();
        debug_assert!(str::from_utf8(bytes).is_ok());

        let text = unsafe { str::from_utf8_unchecked(bytes) };
        text.strip_prefix("//").unwrap().trim_start().starts_with(REGION_END)
    }
}
