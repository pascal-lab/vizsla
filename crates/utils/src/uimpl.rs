#[macro_export]
macro_rules! impl_from {
    ($($variant:ident$(<$V:ident>)?),* for $enum:ident) => {
        $(
            impl$(<$V>)? From<$variant$(<$V>)?> for $enum$(<$V>)? {
                fn from(it: $variant$(<$V>)?) -> $enum$(<$V>)? {
                    $enum::$variant(it)
                }
            }
        )*
    };

    // for paths
    ($($path:path as $ident:ident),* for $enum:ident) => {
        $(
            impl From<$path> for $enum {
                fn from(it: $path) -> $enum {
                    $enum::$ident(it)
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! define_enum_deriving_from {
    ($vis:vis enum $name:ident { $($variant:ident),* $(,)? }) => {
        $vis enum $name { $($variant($variant)),* }

        $crate::impl_from!($($variant),* for $name);
    };

    (#[$attr:meta] $vis:vis enum $name:ident { $($variant:ident),* $(,)? }) => {
        #[$attr]
        $vis enum $name { $($variant($variant)),* }
        $crate::impl_from!($($variant),* for $name);
    }
}
