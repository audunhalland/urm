use syn::parse::ParseStream;

use crate::field;

pub struct ImplTable {
    pub ident: syn::Ident,
    pub methods: Vec<field::FieldMethod>,
}

impl syn::parse::Parse for ImplTable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: syn::token::Impl = input.parse()?;
        let _table_ident: syn::Ident = input.parse()?;
        let _: syn::token::For = input.parse()?;
        let ident: syn::Ident = input.parse()?;

        let content;
        let _brace_token = syn::braced!(content in input);

        let mut methods = Vec::new();
        while !content.is_empty() {
            methods.push(field::FieldMethod::from(content.parse()?)?);
        }

        Ok(ImplTable { ident, methods })
    }
}
