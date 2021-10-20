#[derive(Debug)]
pub enum Predicate {
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Eq(Expr, Expr),
}

#[derive(Debug)]
pub enum Expr {
    TableColumn(TableExpr, &'static str),
}

#[derive(Clone)]
pub struct TableExpr {
    pub table: &'static dyn crate::Table,
    pub alias: u16,
}

impl std::fmt::Debug for TableExpr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "TableExpr({})", self.table.name())?;
        Ok(())
    }
}
