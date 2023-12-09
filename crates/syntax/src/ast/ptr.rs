mod generated;
pub use generated::*;

use crate::{syntax_kind, SyntaxNodePtr};

pub trait AstNodePtr {
    fn can_cast(kind_id: syntax_kind::SyntaxKindId) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNodePtr) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNodePtr;
}
