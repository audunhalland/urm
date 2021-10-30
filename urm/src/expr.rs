use crate::build::QueryBuilder;
use crate::Database;

#[derive(Clone, Debug)]
pub enum Expr<DB: Database> {
    TableColumn(TableExpr<DB>, &'static str),
}

impl<DB: Database> Expr<DB> {
    pub fn build_expr(&self, builder: &mut QueryBuilder) {}
}

#[derive(Clone, Debug)]
pub enum TableExpr<DB: Database> {
    This,
    Parent,
    Alias(TableAlias<DB>),
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
