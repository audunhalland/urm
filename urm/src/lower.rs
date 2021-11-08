use crate::builder::{Build, QueryBuilder};
use crate::ty::{ScalarTyped, Typed, Void};
use crate::Database;

pub trait Lower<DB: Database>: Typed<DB> + Send + Sync + 'static {
    type Target: Build<DB>;

    fn lower(self) -> Option<Self::Target>;
}

struct BuildLowered<T>(T);

impl<T, DB> Build<DB> for BuildLowered<T>
where
    T: Build<DB> + Send + Sync + 'static,
    DB: Database,
{
    fn build(&self, builder: &mut QueryBuilder<DB>) {
        self.0.build(builder);
    }
}

pub trait LowerWhere<DB>
where
    DB: Database,
{
    fn lower_where(self) -> Option<Box<dyn Build<DB>>>;
}

impl<T, DB> LowerWhere<DB> for T
where
    DB: Database,
    T: Lower<DB> + ScalarTyped<DB, bool>,
{
    fn lower_where(self) -> Option<Box<dyn Build<DB>>> {
        match self.lower() {
            Some(lowered) => Some(Box::new(BuildLowered(lowered))),
            None => None,
        }
    }
}

impl<DB, T> Lower<DB> for Void<T>
where
    DB: Database,
    T: Send + Sync + 'static,
{
    type Target = Self;

    fn lower(self) -> Option<Self> {
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
