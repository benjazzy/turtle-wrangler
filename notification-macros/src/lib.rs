use proc_macro::TokenStream;
use syn::parse_macro_input;

mod register_listener_impl;

#[proc_macro_attribute]
pub fn register_listener(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr);
    let item = parse_macro_input!(item);

    register_listener_impl::register_listener_impl(attr, item).into()
}
