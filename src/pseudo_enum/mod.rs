#[macro_export]
macro_rules! pseudo_enum {
    (
        $vis:vis $name:ident: $type:ty;
        $(
            $(#[$attr:meta])*
            $variant:ident = $value:expr$(,)?
        )*
    ) => {
        #[allow(non_snake_case, non_upper_case_globals)]
        $vis mod $name {
            $(
                $(#[$attr])*
                pub const $variant: $type = $value;
            )*
        }
    }
}
