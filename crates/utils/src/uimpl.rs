#[macro_export]
macro_rules! impl_from {
    ($enum:ty => $variant:ident, $($tt:tt)*) => {
        impl From<$variant> for $enum {
            fn from(it: $variant) -> $enum {
                <$enum>::$variant(it)
            }
        }

        $crate::impl_from! { $enum => $($tt)* }
    };

    ($enum:ty => $variant:ident($ty:ty), $($tt:tt)*) => {
        impl From<$ty> for $enum {
            fn from(it: $ty) -> $enum {
                <$enum>::$variant(it)
            }
        }

        $crate::impl_from! { $enum => $($tt)* }
    };

    ($enum:ty =>) => {};
}

#[macro_export]
macro_rules! define_enum_deriving_from {
    (#[$attr:meta] $vis:vis enum $name:ident { $($variant:ident),* $(,)? }) => {
        #[$attr]
        $vis enum $name { $($variant($variant)),* }
        $crate::impl_from! { $name =>
            $($variant,)*
        }
    };

    (#[$attr:meta] $vis:vis enum $name:ident { $($variant:ident($ty:ty)),* $(,)? }) => {
        #[$attr]
        $vis enum $name { $($variant($ty)),* }
        $crate::impl_from! { $name =>
            $($variant($ty),)*
        }
    };
}
