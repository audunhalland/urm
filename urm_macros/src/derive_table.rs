use syn::parse::ParseStream;
use syn::spanned::Spanned;

pub struct TableStruct {
    pub item: syn::ItemStruct,
    pub name: Name,
}

pub struct Name(pub syn::LitStr);

fn find_attr<'a>(attrs: &'a [syn::Attribute], ident: &str) -> Option<&'a proc_macro2::TokenStream> {
    for attr in attrs {
        let path = &attr.path;
        if path.leading_colon.is_some() || path.segments.len() != 1 {
            continue;
        }

        let segment = path.segments.last().unwrap();

        if segment.ident == ident {
            return Some(&attr.tokens);
        }
    }

    None
}

impl syn::parse::Parse for TableStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::ItemStruct = input.parse()?;

        let stream = find_attr(&item.attrs, "table_name")
            .ok_or_else(|| syn::Error::new(item.span(), "#[table_name = \"?\"] not found"))?;

        let name: Name = syn::parse2(stream.clone())?;

        Ok(Self { item, name })
    }
}

impl syn::parse::Parse for Name {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: syn::token::Eq = input.parse()?;
        Ok(Self(input.parse()?))
    }
}
