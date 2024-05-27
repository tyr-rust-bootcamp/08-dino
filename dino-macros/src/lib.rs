mod process_js;

use proc_macro::TokenStream;
use process_js::{process_from_js, process_into_js};

#[proc_macro_derive(IntoJs)]
pub fn derive_into_js(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    process_into_js(input).into()
}

#[proc_macro_derive(FromJs)]
pub fn derive_from_js(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    process_from_js(input).into()
}
