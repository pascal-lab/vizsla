use slang::{Trivia, TriviaKind};

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
