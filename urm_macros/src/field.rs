use proc_macro2::Span;
use quote::*;
use syn::spanned::Spanned;

use crate::attr::{attr_util, foreign};

use crate::attr::foreign::Foreign;

pub struct Field {
    pub span: proc_macro2::Span,
    pub field_name: syn::LitStr,
    pub method_ident: syn::Ident,
    pub struct_ident: syn::Ident,
    pub meta: Meta,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    pub output: syn::Type,
}

pub struct Meta {
    pub foreign: Option<Foreign>,
}

impl Field {
    pub fn from(method: syn::TraitItemMethod) -> Result<Self, syn::Error> {
        let span = method.span();
        let meta = meta_from_attrs(method.attrs)?;

        let output_ty = Self::extract_output_ty(method.sig.output, &meta, span)?;

        let field_name = method.sig.ident.to_string();
        let mut method_chars = field_name.chars();
        let struct_ident = quote::format_ident!(
            "{}__",
            method_chars
                .next()
                .unwrap()
                .to_uppercase()
                .chain(method_chars)
                .collect::<String>()
        );

        let field_name = syn::LitStr::new(&field_name, method.sig.ident.span());

        Ok(Field {
            span,
            field_name,
            method_ident: method.sig.ident,
            struct_ident,
            meta,
            inputs: method.sig.inputs,
            output: output_ty,
        })
    }

    fn extract_output_ty(
        ret_ty: syn::ReturnType,
        meta: &Meta,
        field_span: Span,
    ) -> syn::Result<syn::Type> {
        let user_ty = match ret_ty {
            syn::ReturnType::Default => {
                return Err(syn::Error::new(field_span, "Expected return type"))
            }
            syn::ReturnType::Type(_, ty) => *ty,
        };

        let ty = match &meta.foreign {
            Some(foreign) if foreign.is_collection() => match user_ty {
                syn::Type::Slice(type_slice) => *type_slice.elem,
                _ => return Err(syn::Error::new(user_ty.span(), "Expected slice type")),
            },
            _ => match user_ty {
                syn::Type::Path(path) => syn::Type::Path(path),
                _ => return Err(syn::Error::new(user_ty.span(), "Expected a plain type")),
            },
        };

        Ok(ty)
    }
}

fn meta_from_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Meta> {
    let mut meta = Meta { foreign: None };

    for attr in attrs {
        if attr_util::attr_has_simple_ident(&attr, "foreign") {
            meta.foreign = Some(syn::parse2(attr.tokens)?);
        } else {
            return Err(syn::Error::new(
                attr.path.span(),
                format!("Unrecognized attribute"),
            ));
        }
    }

    Ok(meta)
}

pub fn gen_field_struct(
    field: &Field,
    impl_table: &crate::table::ImplTable,
) -> proc_macro2::TokenStream {
    let span = field.span;
    let field_name = &field.field_name;
    let struct_ident = &field.struct_ident;
    let output = &field.output;

    let table_path = &impl_table.path;

    let handler = if let Some(_) = field.meta.foreign.as_ref() {
        quote_spanned! {span=> ::urm::field::ForeignHandler<#output> }
    } else {
        quote_spanned! {span=> ::urm::field::BasicHandler<#output> }
    };

    quote_spanned! {span=>
        pub struct #struct_ident;

        impl ::urm::field::FieldBase for #struct_ident {
            fn name(&self) -> &'static str {
                #field_name
            }

            fn kind(&self) -> ::urm::field::FieldKind {
                unimplemented!("do not need this")
            }
        }

        impl ::urm::field::Field for #struct_ident {
            type Owner = #table_path;
            type Handler = #handler;
        }
    }
}

pub fn gen_method(field: &Field, impl_table: &crate::table::ImplTable) -> proc_macro2::TokenStream {
    let method_ident = &field.method_ident;
    let inputs = &field.inputs;

    let mod_ident = &impl_table.mod_ident;
    let struct_ident = &field.struct_ident;

    quote! {
        pub fn #method_ident(#inputs) -> #mod_ident::#struct_ident {
            panic!()
        }
    }
}
