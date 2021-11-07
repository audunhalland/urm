use crate::{Database, Table};

pub struct QueryBuilder<'b, DB: Database> {
    db: std::marker::PhantomData<DB>,
    indent: u16,
    buf: &'b mut String,

    pub table: &'static dyn Table<DB = DB>,
    pub parent_table: Option<&'static dyn Table<DB = DB>>,
}

impl<'b, DB: Database> QueryBuilder<'b, DB> {
    pub fn new(table: &'static dyn Table<DB = DB>, buf: &'b mut String) -> Self {
        Self {
            table,
            parent_table: None,
            db: std::marker::PhantomData,
            indent: 0,
            buf,
        }
    }

    pub fn push_table(&mut self, table: &'static dyn Table<DB = DB>) -> QueryBuilder<'_, DB> {
        QueryBuilder {
            db: std::marker::PhantomData,
            indent: self.indent,
            buf: self.buf,
            table,
            parent_table: Some(self.table),
        }
    }

    pub fn buf_mut(&mut self) -> &mut String {
        &mut self.buf
    }

    pub fn outdent(&mut self) {
        self.indent -= 1;
    }

    pub fn newline_indent(&mut self) {
        self.indent += 1;
        self.newline();
    }

    pub fn newline_outdent(&mut self) {
        self.indent -= 1;
        self.newline();
    }

    pub fn newline(&mut self) {
        self.push("\n");
        self.buf.extend((0..self.indent).map(|_| ' '));
    }

    pub fn push(&mut self, str: &str) {
        self.buf.push_str(str);
    }
}
