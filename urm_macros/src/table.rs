use quote::quote;
use syn::parse::ParseStream;

use crate::field;

pub struct ImplTable {
    pub path: syn::Path,
    pub mod_ident: syn::Ident,
    pub fields: Vec<field::Field>,
}

impl syn::parse::Parse for ImplTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: syn::token::Impl = input.parse()?;
        let table: syn::Ident = input.parse()?;
        let _: syn::token::For = input.parse()?;
        let path: syn::Path = input.parse()?;

        if table != "Table" {
            return Err(syn::Error::new(table.span(), "expected Table"));
        }

        let content;
        let _brace_token = syn::braced!(content in input);

        let mut fields = Vec::new();
        while !content.is_empty() {
            fields.push(field::Field::from(content.parse()?)?);
        }

        let ident = &path.segments.last().unwrap().ident;
        let ident_lower = ident.to_string().to_lowercase();
        let mod_ident = quote::format_ident!("__{}", ident_lower);

        Ok(ImplTable {
            path,
            mod_ident,
            fields,
        })
    }
}

pub fn gen_table(table_name: syn::LitStr, impl_table: ImplTable) -> proc_macro2::TokenStream {
    let path = &impl_table.path;
    let mod_ident = &impl_table.mod_ident;

    let field_structs = impl_table
        .fields
        .iter()
        .map(|field| field::gen_field_struct(field, &impl_table));

    let field_methods = impl_table
        .fields
        .iter()
        .map(|field| field::gen_method(field, &impl_table));

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

            #(#field_structs)*
        }
    }
}
