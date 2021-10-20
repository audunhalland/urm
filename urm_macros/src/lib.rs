#![forbid(unsafe_code)]

extern crate proc_macro;

mod derive_probe;
mod table_method;
mod table;

mod attr {
    pub mod attr_util;
    pub mod foreign;
}

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn table(args: TokenStream, input: TokenStream) -> TokenStream {
    let name: syn::LitStr = syn::parse_macro_input!(args as syn::LitStr);
    let impl_table = syn::parse_macro_input!(input as table::ImplTable);

    let tokens = table::gen_table(name, impl_table);

    TokenStream::from(tokens)
}

#[proc_macro_derive(Probe, attributes())]
pub fn derive_probe(input: TokenStream) -> TokenStream {
    let probe_struct = syn::parse_macro_input!(input as derive_probe::ProbeStruct);

    let ident = &probe_struct.item.ident;
    let table_ty = &probe_struct.table_ty;

    let tokens = quote! {
        impl ::urm::Probe for #ident {
            type Table = #table_ty;

            fn node(&self) -> &::urm::Node<Self::Table> {
                &self.0
            }
        }
    };

    TokenStream::from(tokens)
}
