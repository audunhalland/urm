#![forbid(unsafe_code)]

extern crate proc_macro;

mod derive_table;
mod field;
mod table;

mod attr {
    pub mod attr_util;
    pub mod foreign;
}

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Table, attributes(table_name, id, relation))]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let table_struct = syn::parse_macro_input!(input as derive_table::TableStruct);

    let ident = &table_struct.item.ident;
    let name = &table_struct.name.0;

    let tokens = quote! {
        impl ::urm::Table for #ident {
            fn name(&self) -> &'static str {
                #name
            }
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro_attribute]
pub fn table(args: TokenStream, input: TokenStream) -> TokenStream {
    let name: syn::LitStr = syn::parse_macro_input!(args as syn::LitStr);
    let impl_table = syn::parse_macro_input!(input as table::ImplTable);

    TokenStream::from(table::gen_table(name, impl_table))
}

/// FIXME: Belongs in GraphQL crate
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn DbObject(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {})
}

#[proc_macro_derive(DbProxy, attributes(for_table))]
pub fn derive_db_object(input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {})
}
