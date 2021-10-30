pub trait BuildPredicate: std::fmt::Debug + Send + Sync + 'static {
    fn build_predicate(&self, builder: &mut QueryBuilder);
}

pub trait BuildRange: std::fmt::Debug + Send + Sync + 'static {
    fn build_range(&self, builder: &mut QueryBuilder);
}

pub struct QueryBuilder {}

impl BuildPredicate for () {
    fn build_predicate(&self, _builder: &mut QueryBuilder) {}
}

impl BuildRange for () {
    fn build_range(&self, _builder: &mut QueryBuilder) {}
}

impl BuildRange for ::std::ops::Range<usize> {
    fn build_range(&self, _builder: &mut QueryBuilder) {}
}

impl BuildRange for ::std::ops::Range<Option<usize>> {
    fn build_range(&self, _builder: &mut QueryBuilder) {}
}
