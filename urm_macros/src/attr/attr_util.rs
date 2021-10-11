pub fn attr_has_simple_ident(attr: &syn::Attribute, name: &str) -> bool {
    let path = &attr.path;
    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return false;
    }

    let segment = path.segments.last().unwrap();

    segment.ident == name
}
