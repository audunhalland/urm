use crate::builder::{Build, QueryBuilder};
use crate::database::Database;
use crate::ty::{ScalarTyped, Typed, Void};

pub trait Lower<DB: Database>: Typed<DB> + Send + Sync + 'static {
    fn lower(self) -> Option<Lowered<DB>>;
}

impl<DB, T> Lower<DB> for Option<T>
where
    DB: Database,
    T: Lower<DB> + Typed<DB>,
{
    fn lower(self) -> Option<Lowered<DB>> {
        match self {
            Some(t) => t.lower(),
            None => None,
        }
    }
}

pub enum Lowered<DB> {
    And(Vec<Lowered<DB>>),
    Or(Vec<Lowered<DB>>),
    Expr(Box<dyn Build<DB>>),
}

impl<DB> Build<DB> for Lowered<DB>
where
    DB: Database,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        match self {
            Self::Expr(expr) => expr.build(builder),
            Self::And(clauses) => {
                build_multiline_infix("AND", &clauses, builder);
            }
            Self::Or(clauses) => {
                build_multiline_infix("OR", &clauses, builder);
            }
        }
    }
}

fn build_multiline_infix<DB: Database>(
    infix: &str,
    clauses: &[Lowered<DB>],
    builder: &mut QueryBuilder<DB>,
) {
    if clauses.is_empty() {
        return;
    }

    builder.push("(");
    builder.newline_indent();

    let mut iterator = clauses.into_iter();
    let mut item = iterator.next();

    while let Some(cur) = item {
        cur.build(builder);
        let next_item = iterator.next();
        if let Some(_) = next_item.as_ref() {
            builder.newline();
            builder.push(infix);
            builder.newline();
        }
        item = next_item;
    }

    builder.newline_outdent();
    builder.push(")");
}

pub trait LowerWhere<DB>
where
    DB: Database,
{
    fn lower_where(self) -> Option<Lowered<DB>>;
}

impl<T, DB> LowerWhere<DB> for T
where
    DB: Database,
    T: Lower<DB> + ScalarTyped<DB, bool>,
{
    fn lower_where(self) -> Option<Lowered<DB>> {
        self.lower()
    }
}

impl<DB, T> Lower<DB> for Void<T>
where
    DB: Database,
    T: Send + Sync + 'static,
{
    fn lower(self) -> Option<Lowered<DB>> {
        None
    }
}

impl<DB, T> Build<DB> for Void<T>
where
    DB: Database,
    T: Send + Sync + 'static,
{
    fn build(&self, _builder: &mut QueryBuilder<DB>) {
        unimplemented!()
    }
}

pub trait BuildRange<DB: Database>: std::fmt::Debug + Send + Sync + 'static {
    fn build_range(&self, builder: &mut QueryBuilder<DB>);
}

impl<DB: Database> BuildRange<DB> for () {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}

impl<DB: Database> BuildRange<DB> for ::std::ops::Range<usize> {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}

impl<DB: Database> BuildRange<DB> for ::std::ops::Range<Option<usize>> {
    fn build_range(&self, _builder: &mut QueryBuilder<DB>) {}
}
