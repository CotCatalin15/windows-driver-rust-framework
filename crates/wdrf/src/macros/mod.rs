///https://stackoverflow.com/questions/36928569/how-can-i-create-enums-with-constant-values-in-rust
#[macro_export]
macro_rules! def_enum {
    ($vis:vis $name:ident => $ty:ty {
        $($variant:ident => $val:expr),+
        $(,)?
    }) => {
        #[non_exhaustive]
        $vis struct $name;

        #[allow(non_upper_case_globals)]
        impl $name {
            $(
                pub const $variant: $ty = $val;
            )+

            pub const VARIANTS: &'static [$ty] = &[$(Self::$variant),+];
        }
    };
}
