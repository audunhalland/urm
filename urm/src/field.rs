use crate::engine::Projection;
use crate::{Node, Table};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

pub trait Field: Sized + Send + Sync + 'static {
    type Owner;
    type Project: ProjectField;

    fn name() -> &'static str;

    fn local_id() -> LocalId;

    /// Map the field value into another type
    fn map<U, Func>(&self, func: Func) -> MappedField<Self, U, Func, Self::Project>
    where
        Self::Project: MapField<U>,
        Func: Fn(<Self::Project as ProjectField>::Value) -> U,
    {
        MappedField {
            project: MapProject::new(func),
            field: std::marker::PhantomData,
        }
    }
}

pub trait ProjectField: Sized {
    type Value;
    type Output;

    fn project<F: Field>(field: &F, projection: &mut Projection);
}

pub trait MapField<U>: ProjectField + Send + Sync + 'static {
    type Q: Quantify<U>;

    fn map<Func>(&self, func: Func) -> MapProject<U, Func, Self>
    where
        Func: Fn(<Self as ProjectField>::Value) -> U;
}

pub struct MappedField<F: Field, U, Func, P: MapField<U>> {
    project: MapProject<U, Func, P>,
    field: std::marker::PhantomData<F>,
}

impl<F, U, Func, P> Field for MappedField<F, U, Func, P>
where
    F: Field<Project = P>,
    U: Send + Sync + 'static,
    Func: Send + Sync + 'static,
    P: MapField<U>,
{
    type Owner = F::Owner;
    type Project = MapProject<U, Func, F::Project>;

    fn name() -> &'static str {
        F::name()
    }

    fn local_id() -> LocalId {
        F::local_id()
    }
}

pub struct Scalar<T> {
    ph: std::marker::PhantomData<T>,
}

impl<T> ProjectField for Scalar<T> {
    type Value = T;
    type Output = T;

    fn project<F: Field>(field: &F, projection: &mut Projection) {
        projection.project_basic_field(F::local_id());
    }
}

impl<T, U> MapField<U> for Scalar<T>
where
    T: Send + Sync + 'static,
{
    type Q = Unit;

    fn map<F>(&self, func: F) -> MapProject<U, F, Self>
    where
        F: Fn(<Self as ProjectField>::Value) -> U,
    {
        MapProject(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

pub struct ForeignOneToOne<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> ProjectField for ForeignOneToOne<T> {
    type Value = Node<T>;
    type Output = Node<T>;

    fn project<F: Field>(field: &F, projection: &mut Projection) {}
}

impl<T: Table, U> MapField<U> for ForeignOneToOne<T> {
    type Q = Unit;

    fn map<F>(&self, func: F) -> MapProject<U, F, Self>
    where
        F: Fn(<Self as ProjectField>::Value) -> U,
    {
        MapProject(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

pub struct ForeignOneToMany<T: Table> {
    ph: std::marker::PhantomData<T>,
}

impl<T: Table> ProjectField for ForeignOneToMany<T> {
    type Value = Node<T>;
    type Output = Vec<Node<T>>;

    fn project<F: Field>(field: &F, projection: &mut Projection) {}
}

impl<T: Table, U> MapField<U> for ForeignOneToMany<T> {
    type Q = Vector;

    fn map<F>(&self, func: F) -> MapProject<U, F, Self>
    where
        F: Fn(<Self as ProjectField>::Value) -> U,
    {
        MapProject(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

pub struct ForeignField {
    pub foreign_table: &'static dyn Table,
}

pub struct MapProject<T, Func, M: MapField<T>>(
    Func,
    std::marker::PhantomData<T>,
    std::marker::PhantomData<M>,
);

impl<T, Func, M> MapProject<T, Func, M>
where
    M: MapField<T>,
{
    fn new(func: Func) -> Self {
        Self(func, std::marker::PhantomData, std::marker::PhantomData)
    }
}

impl<T, Func, M> ProjectField for MapProject<T, Func, M>
where
    M: MapField<T>,
{
    type Value = T;
    type Output = <M::Q as Quantify<T>>::Output;

    fn project<F: Field>(field: &F, projection: &mut Projection) {}
}

pub struct Unit;
pub struct Vector;

pub trait Quantify<T> {
    type Output;
}

impl<T> Quantify<T> for Unit {
    type Output = T;
}

impl<T> Quantify<T> for Vector {
    type Output = Vec<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyTable;
    struct MyField;

    impl Field for MyField {
        type Owner = MyTable;
        type Project = Scalar<String>;

        fn name() -> &'static str {
            return "test";
        }

        fn local_id() -> LocalId {
            LocalId(0)
        }
    }
}
