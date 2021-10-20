use quote::quote;
use syn::parse::ParseStream;

use crate::field;

pub struct ImplTable {
    pub path: syn::Path,
    pub mod_ident: syn::Ident,
    pub field_results: Vec<syn::Result<field::Field>>,
}

impl syn::parse::Parse for ImplTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: syn::token::Impl = input.parse()?;
        let path: syn::Path = input.parse()?;

        let content;
        let _brace_token = syn::braced!(content in input);

        let mut field_results = Vec::new();
        while !content.is_empty() {
            field_results.push(
                content
                    .parse::<syn::TraitItemMethod>()
                    .and_then(|method| field::Field::try_from(field_results.len(), method)),
            );
        }

        let ident = &path.segments.last().unwrap().ident;
        let ident_lower = ident.to_string().to_lowercase();
        let mod_ident = quote::format_ident!("__{}", ident_lower);

        Ok(ImplTable {
            path,
            mod_ident,
            field_results,
        })
    }
}

pub fn gen_table(table_name: syn::LitStr, impl_table: ImplTable) -> proc_macro2::TokenStream {
    let path = &impl_table.path;
    let mod_ident = &impl_table.mod_ident;

    let field_structs = impl_table.field_results.iter().map(|result| {
        result
            .as_ref()
            .map(|field| field::gen_field_struct(field, &impl_table))
            .unwrap_or_else(|err| err.to_compile_error())
    });

    let field_methods = impl_table.field_results.iter().map(|result| {
        result
            .as_ref()
            .map(|field| field::gen_method(field, &impl_table))
            .unwrap_or_else(|err| err.to_compile_error())
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
