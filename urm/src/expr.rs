use std::fmt::Write;

use crate::builder::QueryBuilder;
use crate::Database;

#[derive(Clone, Debug)]
pub enum Expr<DB: Database> {
    TableColumn(TableExpr<DB>, &'static str),
}

impl<DB: Database> Expr<DB> {
    pub fn build_expr(&self, builder: &mut QueryBuilder<DB>) {
        match self {
            Self::TableColumn(table_expr, name) => {
                table_expr.build(builder);
                builder.push(".");
                builder.push(name);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TableExpr<DB: Database> {
    This,
    Parent,
    Alias(TableAlias<DB>),
}

impl<DB: Database> TableExpr<DB> {
    pub fn build(&self, builder: &mut QueryBuilder<DB>) {
        match self {
            Self::This => builder.push(builder.table.name()),
            Self::Parent => builder.push(builder.parent_table.as_ref().unwrap().name()),
            Self::Alias(alias) => {
                write!(builder.buf_mut(), "a{}", alias.alias).unwrap();
            }
        }
    }
}

#[derive(Clone)]
pub struct TableAlias<DB: Database> {
    pub table: &'static dyn crate::Table<DB = DB>,
    pub alias: u16,
}

impl<DB: Database> std::fmt::Debug for TableAlias<DB> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "TableExpr({})", self.table.name())?;
        Ok(())
    }
}
