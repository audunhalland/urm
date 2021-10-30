use crate::build::QueryBuilder;

#[derive(Clone, Debug)]
pub enum Expr {
    TableColumn(TableExpr, &'static str),
}

impl Expr {
    pub fn build_expr(&self, builder: &mut QueryBuilder) {}
}

#[derive(Clone, Debug)]
pub enum TableExpr {
    This,
    Parent,
    Alias(TableAlias),
}

#[derive(Clone)]
pub struct TableAlias {
    pub table: &'static dyn crate::Table,
    pub alias: u16,
}

impl std::fmt::Debug for TableAlias {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "TableExpr({})", self.table.name())?;
        Ok(())
    }
}
