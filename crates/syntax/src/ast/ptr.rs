mod generated;
pub use generated::*;

use crate::SyntaxNodePtr;

pub trait AstNodePtr {
    fn can_cast(syntax: &SyntaxNodePtr) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNodePtr) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNodePtr;
}
