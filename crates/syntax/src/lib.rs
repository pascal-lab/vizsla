pub mod ptr;
pub mod slang_ext;

pub use slang::*;
pub use slang_ext::*;

#[macro_export]
macro_rules! match_ast {
    ($node:ident in _ => $body:expr,) => { $body };

    ($node:ident in $path:ty as $it:pat => $body:expr, $($rest:tt)* ) => {{
        if let Some($it) = <$path as $crate::ast::AstNode>::cast($node) {
            $body
        } else {
            match_ast!($node in $($rest)*)
        }
    }};

    ($node:ident in $path:ty $(| $paths:ty)* => $body:expr, $($rest:tt)* ) => {{
        if <$path as $crate::ast::AstNode>::cast($node).is_some() $(|| <$paths as $crate::ast::AstNode>::cast($node).is_some())* {
            $body
        } else {
            match_ast!($node in $($rest)*)
        }
    }}
}
