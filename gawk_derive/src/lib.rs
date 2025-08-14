use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = quote! {
        impl Send for #name {}
        impl Sync for #name {}
        impl Clone for #name {
            fn clone(&self) -> Self {
                // Implement clone logic or use #[derive(Clone)] on the struct
                unimplemented!()
            }
        }
        impl std::panic::RefUnwindSafe for #name {}
        impl 'static for #name {}
    };

    TokenStream::from(expanded)
}
