use crate::Database;

pub struct QueryBuilder<DB: Database> {
    db: std::marker::PhantomData<DB>,
    indent: u16,
    buf: String,
}

impl<DB: Database> QueryBuilder<DB> {
    pub fn new() -> Self {
        Self {
            db: std::marker::PhantomData,
            indent: 0,
            buf: String::new(),
        }
    }

    pub fn build(self) -> String {
        self.buf
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
