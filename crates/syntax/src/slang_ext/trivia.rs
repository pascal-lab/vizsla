use std::str;

use slang::{SyntaxTrivia, Trivia, TriviaKind};

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
    fn is_region_begin(&self) -> bool;
    fn is_region_end(&self) -> bool;
}

const REGION_BEGIN: &str = "region";
const REGION_END: &str = "endregion";

impl TriviaExt for SyntaxTrivia<'_> {
    #[inline]
    fn is_region_begin(&self) -> bool {
        // TODO: use from_utf8_unchecked?
        matches!(self.kind(), Trivia![lc])
            && str::from_utf8(self.get_raw_text().as_bytes())
                .is_ok_and(|s| s.strip_prefix("//").unwrap().trim_start().starts_with(REGION_BEGIN))
    }

    #[inline]
    fn is_region_end(&self) -> bool {
        matches!(self.kind(), Trivia![lc])
            && str::from_utf8(self.get_raw_text().as_bytes())
                .is_ok_and(|s| s.strip_prefix("//").unwrap().trim_start().starts_with(REGION_END))
    }
}
