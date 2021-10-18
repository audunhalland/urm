use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::field;
use crate::query;
use crate::{Table, UrmResult};

#[derive(Clone)]
pub struct Engine {
    pub query: Arc<Mutex<QueryEngine>>,
}

impl Engine {
    pub fn new_select(from: &'static dyn Table) -> (Self, Probing) {
        let projection = Arc::new(Mutex::new(Projection::new()));

        let query_engine = Arc::new(Mutex::new(QueryEngine {
            root_select: Select {
                from,
                predicate: None,
                projection: projection.clone(),
            },
        }));

        let engine = Self {
            query: query_engine.clone(),
        };

        (engine.clone(), Probing { projection, engine })
    }
}

#[derive(Debug)]
pub struct QueryEngine {
    root_select: Select,
}

impl QueryEngine {
    pub async fn execute(self) -> UrmResult<()> {
        let mut builder = query::PGQueryBuilder::new();
        self.root_select.build_query(&mut builder);
        // TODO Execute here

        panic!();
    }
}

/// Each Probing the QueryEngine
/// hands out must be completed by
/// calling the `complete` method.
#[derive(Clone)]
pub struct Probing {
    projection: Arc<Mutex<Projection>>,
    engine: Engine,
}

impl Probing {
    pub fn new(engine: Engine) -> Self {
        Self {
            projection: Arc::new(Mutex::new(Projection::new())),
            engine,
        }
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn projection(&self) -> &Arc<Mutex<Projection>> {
        &self.projection
    }
}

/// # Select
///
/// Encodes the intent of selecting *something* from a table.
///
/// It's an abstraction over an "SQL"-like select expression.
///
struct Select {
    /// TODO: we can have many FROMs in a select.
    from: &'static dyn Table,

    /// The projection, which is getting built dynamically. Eh...
    /// TODO: Does the projection contain all child "queries"?
    /// not likely.
    projection: Arc<Mutex<Projection>>,

    /// Where clause
    predicate: Option<query::Predicate>,
}

impl Select {
    fn build_query(&self, builder: &mut dyn query::QueryBuilder) {
        builder.enter_select();
        builder.exit_select();
    }
}

impl std::fmt::Debug for Select {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let lock = self.projection.lock();
        write!(fmt, "SELECT From '{}' {:?}", self.from.name(), *lock)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Projection {
    fields: BTreeMap<field::LocalId, QueryField>,
}

impl Projection {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
        }
    }

    pub fn project_primitive_field(&mut self, local_id: field::LocalId) {
        self.fields.insert(local_id, QueryField::Primitive);
    }

    pub fn foreign_subselect(
        &mut self,
        local_id: field::LocalId,
        from: &'static dyn Table,
        projection: Arc<Mutex<Projection>>,
    ) {
        self.fields.insert(
            local_id,
            QueryField::Foreign(Select {
                projection,
                from,
                predicate: None,
            }),
        );
    }
}

#[derive(Debug)]
enum QueryField {
    Primitive,
    Foreign(Select),
}
