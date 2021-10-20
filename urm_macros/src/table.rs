use quote::quote;
use syn::parse::ParseStream;

use crate::table_method;

pub struct ImplTable {
    pub path: syn::Path,
    pub mod_ident: syn::Ident,
    pub methods: Vec<table_method::Method>,
}

impl syn::parse::Parse for ImplTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: syn::token::Impl = input.parse()?;
        let path: syn::Path = input.parse()?;

        let content;
        let _brace_token = syn::braced!(content in input);

        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(table_method::Method::new(methods.len(), content.parse()));
        }

        let ident = &path.segments.last().unwrap().ident;
        let ident_lower = ident.to_string().to_lowercase();
        let mod_ident = quote::format_ident!("__{}", ident_lower);

        Ok(ImplTable {
            path,
            mod_ident,
            methods,
        })
    }
}

pub fn gen_table(table_name: syn::LitStr, impl_table: ImplTable) -> proc_macro2::TokenStream {
    let path = &impl_table.path;
    let mod_ident = &impl_table.mod_ident;

    let field_structs = impl_table.methods.iter().map(|method| {
        table_method::gen_field_struct(method, &impl_table)
    });

    let field_methods = impl_table.methods.iter().map(|method| {
        table_method::gen_method(method, &impl_table)
    });

    quote! {
        impl ::urm::Table for #path {
            fn name(&self) -> &'static str {
                #table_name
            }
        }

        impl #path {
            #(#field_methods)*
        }

        mod #mod_ident {
            use super::*;
            use ::urm::prelude::*;

            static INSTANCE: #path = #path;

            impl ::urm::Instance for #path {
                fn instance() -> &'static Self {
                    &#mod_ident::INSTANCE
                }
            }

            #(#field_structs)*
        }
    }
}
