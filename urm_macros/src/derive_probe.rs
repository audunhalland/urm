use syn::parse::ParseStream;
use syn::spanned::Spanned;

pub struct ProbeStruct {
    pub item: syn::ItemStruct,
    pub table_ty: syn::Type,
}

impl syn::parse::Parse for ProbeStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::ItemStruct = input.parse()?;

        let node_field = match &item.fields {
            syn::Fields::Unnamed(fields_unnamed) => match fields_unnamed.unnamed.len() {
                1 => &fields_unnamed.unnamed[0],
                _ => return Err(syn::Error::new(item.fields.span(), "Expected one field")),
            },
            _ => {
                return Err(syn::Error::new(
                    item.fields.span(),
                    "Expected one unnamed field",
                ))
            }
        };

        let node_path = match &node_field.ty {
            syn::Type::Path(path) => path,
            _ => return Err(syn::Error::new(node_field.span(), "Expected a type path")),
        };

        let node_segments = &node_path.path.segments;
        let last_node_segment = node_segments.last().unwrap();

        if last_node_segment.ident != "Node" {
            return Err(syn::Error::new(last_node_segment.span(), "Expected Node"));
        }

        let generics = match &last_node_segment.arguments {
            syn::PathArguments::AngleBracketed(args) => args,
            _ => {
                return Err(syn::Error::new(
                    last_node_segment.span(),
                    "Expected Node<T>",
                ))
            }
        };

        let generic_arg = match generics.args.len() {
            1 => generics.args.last().unwrap(),
            _ => {
                return Err(syn::Error::new(
                    generics.span(),
                    "Expected one generic argument",
                ))
            }
        };

        let table_ty = match generic_arg {
            syn::GenericArgument::Type(ty) => ty.clone(),
            _ => {
                return Err(syn::Error::new(
                    generic_arg.span(),
                    "Expected one type argument",
                ))
            }
        };

        Ok(Self { item, table_ty })
    }
}
