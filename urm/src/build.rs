pub trait BuildQuery {
    fn build_query(self, builder: &mut QueryBuilder);
}

pub struct QueryBuilder {}
