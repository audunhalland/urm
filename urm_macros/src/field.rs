use proc_macro2::Span;
use quote::*;
use syn::spanned::Spanned;

use crate::attr::attr_util;
use crate::attr::foreign;

pub struct Field {
    pub span: proc_macro2::Span,
    pub field_idx: usize,
    pub field_name: syn::LitStr,
    pub method_ident: syn::Ident,
    pub struct_ident: syn::Ident,
    pub meta: Meta,
    pub inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    pub output: syn::Type,
}

pub struct Meta {
    pub foreign: Option<foreign::Foreign>,
}

impl Field {
    pub fn try_from(field_idx: usize, method: syn::TraitItemMethod) -> Result<Self, syn::Error> {
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
            field_idx,
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
    let field_id = field.field_idx as u16;
    let field_name = &field.field_name;
    let struct_ident = &field.struct_ident;
    let output = &field.output;

    let local_table_path = &impl_table.path;

    let mechanics = match field
        .meta
        .foreign
        .as_ref()
        .map(|foreign| &foreign.direction)
    {
        None => quote_spanned! {span=> Primitive },
        Some(foreign::Direction::SelfReferencesForeign) => quote_spanned! {span=> ForeignOneToOne },
        Some(foreign::Direction::ForeignReferencesSelf) => {
            quote_spanned! {span=> ForeignOneToMany }
        }
    };

    let foreign_field_impl = if let Some(foreign) = &field.meta.foreign {
        let span = foreign.span;
        let foreign_table_path = &foreign.foreign_table_path;

        let gen_table_column = |table_expr: proc_macro2::TokenStream,
                                table_path: &syn::Path,
                                field_ident: &syn::Ident| {
            let span = field_ident.span();

            quote_spanned! {span=>
                urm::expr::Expr::TableColumn(#table_expr.clone(), #table_path::#field_ident().name())
            }
        };

        let gen_eq_pred = |p: &foreign::ColumnEqPredicate| {
            let local = gen_table_column(quote! { local_table }, local_table_path, &p.local_ident);
            let foreign = gen_table_column(
                quote! { foreign_table },
                foreign_table_path,
                &p.foreign_ident,
            );

            quote_spanned! {span=>
                ::urm::expr::Predicate::Eq(#local, #foreign)
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

        Some(quote_spanned! {span=>
            impl ::urm::field::ForeignField for #struct_ident {
                type ForeignTable = #foreign_table_path;

                fn join_predicate(&self, local_table: ::urm::expr::TableExpr, foreign_table: ::urm::expr::TableExpr) -> ::urm::expr::Predicate {
                    #eq_pred
                }
            }
        })
    } else {
        None
    };

    quote_spanned! {span=>
        #[derive(Debug)]
        pub struct #struct_ident;

        impl ::urm::field::Field for #struct_ident {
            type Table = #local_table_path;
            type Mechanics = ::urm::field::#mechanics<#output>;

            fn name(&self) -> &'static str {
                #field_name
            }

            fn local_id() -> ::urm::field::LocalId {
                ::urm::field::LocalId(#field_id)
            }
        }

        #foreign_field_impl
    }
}

pub fn gen_method(field: &Field, impl_table: &crate::table::ImplTable) -> proc_macro2::TokenStream {
    let method_ident = &field.method_ident;
    let inputs = &field.inputs;

    let mod_ident = &impl_table.mod_ident;
    let struct_ident = &field.struct_ident;

    quote! {
        pub fn #method_ident(#inputs) -> #mod_ident::#struct_ident {
            #mod_ident::#struct_ident
        }
    }
}
