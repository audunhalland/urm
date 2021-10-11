use syn::parse::ParseStream;

pub struct Foreign {
    pub self_columns: Vec<syn::Ident>,
    pub foreign_tuple: ColumnTuple,
    pub direction: Direction,
}

pub struct ColumnTuple {
    pub table: Table,
    pub columns: Vec<syn::Ident>,
}

pub enum Table {
    Zelf,
    Foreign(syn::Path),
}

impl Table {
    fn is_self(&self) -> bool {
        match self {
            Table::Zelf => true,
            _ => false,
        }
    }
}

pub enum Direction {
    SelfReferencesForeign,
    ForeignReferencesSelf,
}

impl syn::parse::Parse for Foreign {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _parent_token = syn::parenthesized!(content in input);
        let content_span = content.span();

        let first: ColumnTuple = content.parse()?;
        let _arrow: syn::token::FatArrow = content.parse()?;
        let second: ColumnTuple = content.parse()?;

        if first.table.is_self() && second.table.is_self() {
            return Err(syn::Error::new(
                content_span,
                format!("Foreign self not supported (yet?)"),
            ));
        }

        if first.table.is_self() {
            Ok(Self {
                self_columns: first.columns,
                foreign_tuple: second,
                direction: Direction::SelfReferencesForeign,
            })
        } else if second.table.is_self() {
            Ok(Self {
                self_columns: second.columns,
                foreign_tuple: first,
                direction: Direction::ForeignReferencesSelf,
            })
        } else {
            Err(syn::Error::new(content_span, format!("No Self(..) found")))
        }
    }
}

impl syn::parse::Parse for ColumnTuple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let table = if input.peek(syn::token::SelfType) {
            let _: syn::token::SelfType = input.parse()?;
            Table::Zelf
        } else {
            Table::Foreign(input.parse()?)
        };

        let content;
        let _parent_token = syn::parenthesized!(content in input);

        let mut columns = vec![];

        while !content.is_empty() {
            columns.push(content.parse()?);
        }

        Ok(Self { table, columns })
    }
}
