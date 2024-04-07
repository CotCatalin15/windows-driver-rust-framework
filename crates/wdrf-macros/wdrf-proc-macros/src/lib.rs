use irql_check::irql_check_attr_impl;
use proc_macro::TokenStream;

mod irql_check;

#[proc_macro_attribute]
pub fn irql_check(attr: TokenStream, item: TokenStream) -> TokenStream {
    irql_check_attr_impl(attr.into(), item.into())
        .unwrap()
        .into()
}
