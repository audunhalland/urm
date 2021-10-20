use proc_macro2::Span;
use syn::parse::ParseStream;

pub struct Foreign {
    pub span: Span,
    pub foreign_table_path: syn::Path,
    pub eq_predicates: Vec<ColumnEqPredicate>,
    pub direction: Direction,
}

pub struct ColumnEqPredicate {
    pub local_ident: syn::Ident,
    pub foreign_ident: syn::Ident,
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

impl Foreign {
    pub fn is_collection(&self) -> bool {
        match &self.direction {
            Direction::ForeignReferencesSelf => true,
            Direction::SelfReferencesForeign => false,
        }
    }
}

impl syn::parse::Parse for Foreign {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _parent_token = syn::parenthesized!(content in input);
        let content_span = content.span();

        let first: ColumnTuple = content.parse()?;
        let _arrow: syn::token::FatArrow = content.parse()?;
        let second: ColumnTuple = content.parse()?;

        match (first.table, second.table) {
            (Table::Zelf, Table::Foreign(path)) => Ok(Self {
                span: content_span,
                foreign_table_path: path,
                eq_predicates: create_column_eq_predicates(
                    first.columns,
                    second.columns,
                    content_span,
                )?,
                direction: Direction::SelfReferencesForeign,
            }),
            (Table::Foreign(path), Table::Zelf) => Ok(Self {
                span: content_span,
                foreign_table_path: path,
                eq_predicates: create_column_eq_predicates(
                    second.columns,
                    first.columns,
                    content_span,
                )?,
                direction: Direction::ForeignReferencesSelf,
            }),
            (Table::Zelf, Table::Zelf) => Err(syn::Error::new(
                content_span,
                format!("Foreign self not supported (yet?)"),
            )),
            (Table::Foreign(_), Table::Foreign(_)) => {
                Err(syn::Error::new(content_span, format!("No Self(..) found")))
            }
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

fn create_column_eq_predicates(
    local: Vec<syn::Ident>,
    foreign: Vec<syn::Ident>,
    content_span: Span,
) -> syn::Result<Vec<ColumnEqPredicate>> {
    if local.len() != foreign.len() {
        return Err(syn::Error::new(
            content_span,
            format!("Must have the same number of columns in self and foreign"),
        ));
    }

    if local.len() == 0 {
        return Err(syn::Error::new(
            content_span,
            format!("Must specify at least one field"),
        ));
    }

    Ok(local
        .into_iter()
        .zip(foreign.into_iter())
        .map(|(local_ident, foreign_ident)| ColumnEqPredicate {
            local_ident,
            foreign_ident,
        })
        .collect())
}
