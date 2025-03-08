macro_rules! intrinsics{
    ($($(#[$attr:meta])* pub fn $name:ident($($arg_name:ident:$arg:ty),+) -> $ret:ty;)+) => {
        $(
            paste::item! {
                #[allow(non_camel_case_types)]
                pub struct [<Intrinsic $name>];
            }
            paste::item! {
                impl Intrinsic for [<Intrinsic $name>] {
                    fn prototype(&self) -> FunctionPrototype {
                        FunctionPrototype {
                            name: stringify!($name).to_owned(),
                            arg_types: vec![$(<$arg as crate::type_info::HasStaticTypeId>::type_id().clone()),*],
                            ret_type: <$ret as crate::type_info::HasStaticTypeId>::type_id().clone()
                        }
                    }
                }
            }
            paste::item! {
                #[allow(non_upper_case_globals)]
                $(#[$attr])* pub const $name:[<Intrinsic $name>] = [<Intrinsic $name>];
            }
        )*
    };
    ($(#[$attr:meta])*) => {}
}
