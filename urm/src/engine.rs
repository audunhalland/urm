use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::expr;
use crate::field;
use crate::query;
use crate::{Table, UrmResult};

#[derive(Clone)]
pub struct Engine {
    pub query: Arc<Mutex<QueryEngine>>,
}

impl Engine {
    pub fn new_select(from: &'static dyn Table) -> (Self, Probing) {
        let root_select = Arc::new(Select {
            from: expr::TableAlias {
                table: from,
                alias: 0,
            },
            projection: Mutex::new(BTreeMap::new()),
            predicate: None,
        });

        let query_engine = Arc::new(Mutex::new(QueryEngine {
            root_select: root_select.clone(),
        }));

        let engine = Self {
            query: query_engine.clone(),
        };

        (engine.clone(), Probing::new(engine, root_select))
    }
}

#[derive(Debug)]
pub struct QueryEngine {
    root_select: Arc<Select>,
}

impl QueryEngine {
    pub fn new_select(&self, from: &'static dyn Table) -> Arc<Select> {
        Arc::new(Select {
            from: expr::TableAlias {
                table: from,
                alias: 0,
            },
            projection: Mutex::new(BTreeMap::new()),
            predicate: None,
        })
    }

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
    engine: Engine,
    select: Arc<Select>,
}

impl Probing {
    pub fn new(engine: Engine, select: Arc<Select>) -> Self {
        Self { engine, select }
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn select(&self) -> &Arc<Select> {
        &self.select
    }
}

/// # Select
///
/// Encodes the intent of selecting *something* from a table.
///
/// It's an abstraction over an "SQL"-like select expression.
///
pub struct Select {
    /// TODO: we can have many FROMs in a select.
    pub from: expr::TableAlias,

    /// The projection, which is getting built dynamically. Eh...
    /// TODO: Does the projection contain all child "queries"?
    /// not likely.
    pub projection: Mutex<BTreeMap<field::LocalId, QueryField>>,

    /// Where clause
    pub predicate: Option<expr::Predicate>,
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
        write!(fmt, "SELECT From '{}' {:?}", self.from.table.name(), *lock)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum QueryField {
    Primitive,
    Foreign {
        select: Arc<Select>,
        join_predicate: expr::Predicate,
    },
}
