use syn::spanned::Spanned;

use crate::attr::attr_util;
use crate::attr::foreign::Foreign;

pub struct FieldMethod {
    ident: syn::Ident,
    meta: Meta,
    inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    output: syn::Type,
}

struct Meta {
    foreign: Option<Foreign>,
}

impl FieldMethod {
    pub fn from(method: syn::TraitItemMethod) -> Result<Self, syn::Error> {
        let span = method.span();
        let meta = meta_from_attrs(method.attrs)?;

        // TODO: parse attr for real. Hack
        // for the #[foreign] attr
        let output_ty = match method.sig.output {
            syn::ReturnType::Default => return Err(syn::Error::new(span, "Expected return type")),
            syn::ReturnType::Type(_, ty) => *ty,
        };

        Ok(FieldMethod {
            ident: method.sig.ident,
            meta,
            inputs: method.sig.inputs,
            output: output_ty,
        })
    }
}

fn meta_from_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Meta> {
    let mut meta = Meta { foreign: None };

    for attr in attrs {
        if attr_util::attr_has_simple_ident(&attr, "foreign") {
            meta.foreign = Some(syn::parse2(attr.tokens)?);
        } else {
            return Err(syn::Error::new(
                attr.span(),
                format!("Unrecognized attribute"),
            ));
        }
    }

    Ok(meta)
}
