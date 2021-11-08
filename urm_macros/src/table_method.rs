use proc_macro2::Span;
use quote::*;
use syn::spanned::Spanned;

use crate::attr::attr_util;
use crate::attr::foreign;

pub enum Method {
    /// An associated function (without a `self`) becomes a 'selector':
    /// TODO: Don't need this, with the Filter mechanism
    Selector,
    /// A proper method with `self` receiver becomes a Field:
    Field(Field),
    Error(syn::Error),
}

pub struct Field {
    pub span: proc_macro2::Span,
    pub field_idx: usize,
    pub field_name: syn::LitStr,
    pub method_ident: syn::Ident,
    pub struct_ident: syn::Ident,
    pub meta: Meta,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    return_type: Quantified<ReturnType>,
}

enum Quantified<T> {
    Unit(T),
    Slice(syn::token::Bracket, T),
}

enum ReturnType {
    Path(syn::TypePath),
    Zelf(syn::Ident),
}

impl syn::spanned::Spanned for ReturnType {
    fn span(&self) -> Span {
        match self {
            Self::Path(path) => path.span(),
            Self::Zelf(ident) => ident.span(),
        }
    }
}

pub struct Meta {
    pub foreign: Option<foreign::Foreign>,
}

impl Method {
    pub fn new(field_idx: usize, ast: syn::Result<syn::TraitItemMethod>) -> Self {
        match Self::try_from_ast(field_idx, ast) {
            Err(error) => Self::Error(error),
            Ok(method) => method,
        }
    }

    fn try_from_ast(field_idx: usize, ast: syn::Result<syn::TraitItemMethod>) -> syn::Result<Self> {
        let method = ast?;
        let span = method.span();
        let meta = meta_from_attrs(method.attrs)?;

        let return_type = Self::extract_quantified_return_type(method.sig.output, span)?;

        match method.sig.inputs.first() {
            Some(syn::FnArg::Receiver(_)) => {
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

                Ok(Self::Field(Field {
                    span,
                    field_idx,
                    field_name,
                    method_ident: method.sig.ident,
                    struct_ident,
                    meta,
                    inputs: method.sig.inputs,
                    return_type,
                }))
            }
            _ => Ok(Self::Selector),
        }
    }

    fn extract_quantified_return_type(
        ret_ty: syn::ReturnType,
        field_span: Span,
    ) -> syn::Result<Quantified<ReturnType>> {
        let user_ty = match ret_ty {
            syn::ReturnType::Default => {
                return Err(syn::Error::new(field_span, "Expected return type"))
            }
            syn::ReturnType::Type(_, ty) => *ty,
        };

        match user_ty {
            syn::Type::Slice(type_slice) => Ok(Quantified::Slice(
                type_slice.bracket_token,
                Self::extract_return_type(*type_slice.elem)?,
            )),
            ty => Ok(Quantified::Unit(Self::extract_return_type(ty)?)),
        }
    }

    fn extract_return_type(ty: syn::Type) -> syn::Result<ReturnType> {
        match ty {
            syn::Type::Path(path) => {
                if path.path.segments.len() == 1
                    && path.path.segments.first().as_ref().unwrap().ident == "Self"
                {
                    Ok(ReturnType::Zelf(
                        path.path.segments.into_iter().next().unwrap().ident,
                    ))
                } else {
                    Ok(ReturnType::Path(path))
                }
            }
            _ => return Err(syn::Error::new(ty.span(), "Expected simple Path-like type")),
        }
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

pub fn gen_method(
    method: &Method,
    impl_table: &crate::table::ImplTable,
) -> proc_macro2::TokenStream {
    let field = match method {
        Method::Selector => return quote! {},
        Method::Field(field) => field,
        Method::Error(error) => return error.to_compile_error(),
    };
    let method_ident = &field.method_ident;
    let inputs = &field.inputs;

    let struct_ident = &field.struct_ident;
    let local_table_path = &impl_table.path;

    let return_type_to_tokens = |return_type: &ReturnType| -> proc_macro2::TokenStream {
        match return_type {
            ReturnType::Zelf(zelf) => {
                let span = zelf.span();
                quote_spanned! {span=> #struct_ident }
            }
            ReturnType::Path(path) => quote! { #path },
        }
    };

    if let Some(foreign) = &field.meta.foreign {
        let span = foreign.span;
        let foreign_table_path = &foreign.foreign_table_path;

        let outcome = match foreign.direction {
            foreign::Direction::SelfReferencesForeign => {
                quote_spanned! {span=> OneToOne }
            }
            foreign::Direction::ForeignReferencesSelf => {
                quote_spanned! {span=> OneToMany }
            }
        };

        let gen_table_column = |table_path: &syn::Path, field_ident: &syn::Ident| {
            let span = field_ident.span();

            quote_spanned! {span=>
                #table_path.#field_ident()
            }
        };

        let gen_eq_pred = |p: &foreign::ColumnEqPredicate| {
            let local = gen_table_column(local_table_path, &p.local_ident);
            let foreign = gen_table_column(foreign_table_path, &p.foreign_ident);

            quote_spanned! {span=>
                ::urm::function::Equals(#local, #foreign)
            }
        };

        let eq_pred = match foreign.eq_predicates.len() {
            1 => gen_eq_pred(&foreign.eq_predicates[0]),
            _ => {
                let eq_preds = foreign.eq_predicates.iter().map(gen_eq_pred);

                quote_spanned! {span=>
                    ::urm::expr::Predicate::And(#(#eq_preds),*)
                }
            }
        };

        let output_type = match foreign.direction {
            foreign::Direction::SelfReferencesForeign => match &field.return_type {
                Quantified::Unit(return_type) => return_type_to_tokens(return_type),
                Quantified::Slice(bracket, _) => {
                    syn::Error::new(bracket.span, "Expected non-slice unit type").to_compile_error()
                }
            },
            foreign::Direction::ForeignReferencesSelf => match &field.return_type {
                Quantified::Unit(ty) => {
                    syn::Error::new(ty.span(), "Expected slice").to_compile_error()
                }
                Quantified::Slice(_, return_type) => return_type_to_tokens(return_type),
            },
        };

        quote! {
            pub fn #method_ident(#inputs) -> ::urm::foreign::Foreign<
                #local_table_path,
                #foreign_table_path,
                ::urm::foreign::#outcome<#output_type>,
                impl ::urm::lower::Lower<::urm::database::Postgres> + ::urm::ty::ScalarTyped<::urm::database::Postgres, bool>,
                ()
            > {
                urm::foreign::foreign(#eq_pred)
            }
        }
    } else {
        let field_name = &field.field_name;
        let field_id = field.field_idx as u16;

        let ty = match &field.return_type {
            Quantified::Unit(return_type) => match return_type {
                ReturnType::Zelf(zelf) => {
                    syn::Error::new(zelf.span(), "Expected a type, not Self").to_compile_error()
                }
                ReturnType::Path(path) => quote! { ::urm::ty::Unit<#path> },
            },
            Quantified::Slice(bracket, _) => {
                syn::Error::new(bracket.span, "Expected non-slice unit type").to_compile_error()
            }
        };

        quote! {
            pub fn #method_ident(#inputs) -> ::urm::column::Column<#local_table_path, #ty> {
                ::urm::column::Column::new(
                    #field_name,
                    ::urm::project::LocalId(#field_id)
                )
            }
        }
    }
}
