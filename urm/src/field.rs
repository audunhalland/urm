use crate::engine::Projection;
use crate::{Node, Table};

pub trait FieldBase: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn kind(&self) -> FieldKind;
}

pub trait Field: FieldBase {
    type Owner;
    type Handler: Handler;
}

pub trait Handler {
    type Output;

    fn project(field: &dyn FieldBase, projection: &mut Projection);
}

pub struct BasicHandler<T> {
    ph: std::marker::PhantomData<T>,
}

impl<T> Handler for BasicHandler<T> {
    type Output = T;

    fn project(field: &dyn FieldBase, projection: &mut Projection) {
        projection.add_basic_field();
    }
}

pub struct ForeignHandler<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> Handler for ForeignHandler<T> {
    type Output = Vec<Node<T>>;

    fn project(field: &dyn FieldBase, projection: &mut Projection) {}
}

pub enum FieldKind {
    Basic,
    Foreign(ForeignField),
}

pub struct ForeignField {
    pub foreign_table: &'static dyn Table,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyTable;
    struct MyField;

    impl FieldBase for MyField {
        fn name(&self) -> &'static str {
            return "test";
        }

        fn kind(&self) -> FieldKind {
            panic!()
        }
    }

    impl Field for MyField {
        type Owner = MyTable;
        type Handler = BasicHandler<String>;
    }
}
