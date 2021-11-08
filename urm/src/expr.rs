use crate::database::Database;

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
