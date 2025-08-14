use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive macro generating an impl of the trait Event
#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl gawk::Event for #name {}
    };

    TokenStream::from(expanded)
}
