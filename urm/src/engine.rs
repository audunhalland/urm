use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::build::BuildPredicate;
use crate::expr;
use crate::project;
use crate::query;
use crate::{Database, Table, UrmResult};

#[derive(Clone)]
pub struct Engine<DB: Database> {
    pub query: Arc<Mutex<QueryEngine<DB>>>,
}

impl<DB: Database> Engine<DB> {
    pub fn new_select(from: &'static dyn Table<DB = DB>) -> (Self, Probing<DB>) {
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
pub struct QueryEngine<DB: Database> {
    root_select: Arc<Select<DB>>,
}

impl<DB: Database> QueryEngine<DB> {
    pub fn new_select(
        &self,
        from: &'static dyn Table<DB = DB>,
        predicate: Option<Box<dyn BuildPredicate<DB>>>,
    ) -> Arc<Select<DB>> {
        Arc::new(Select {
            from: expr::TableAlias {
                table: from,
                alias: 0,
            },
            projection: Mutex::new(BTreeMap::new()),
            predicate,
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
pub struct Probing<DB: Database> {
    engine: Engine<DB>,
    select: Arc<Select<DB>>,
}

impl<DB: Database> Probing<DB> {
    pub fn new(engine: Engine<DB>, select: Arc<Select<DB>>) -> Self {
        Self { engine, select }
    }

    pub fn engine(&self) -> &Engine<DB> {
        &self.engine
    }

    pub fn select(&self) -> &Arc<Select<DB>> {
        &self.select
    }
}

/// # Select
///
/// Encodes the intent of selecting *something* from a table.
///
/// It's an abstraction over an "SQL"-like select expression.
///
pub struct Select<DB: Database> {
    /// TODO: we can have many FROMs in a select.
    pub from: expr::TableAlias<DB>,

    /// The projection, which is getting built dynamically. Eh...
    /// TODO: Does the projection contain all child "queries"?
    /// not likely.
    pub projection: Mutex<BTreeMap<project::LocalId, QueryField<DB>>>,

    /// Where clause
    pub predicate: Option<Box<dyn BuildPredicate<DB>>>,
}

impl<DB: Database> Select<DB> {
    fn build_query(&self, builder: &mut dyn query::QueryBuilder) {
        builder.enter_select();
        builder.exit_select();
    }
}

impl<DB: Database> std::fmt::Debug for Select<DB> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let lock = self.projection.lock();
        write!(fmt, "SELECT From '{}' {:?}", self.from.table.name(), *lock)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum QueryField<DB: Database> {
    Primitive,
    Foreign {
        select: Arc<Select<DB>>,
        join_predicate: Box<dyn BuildPredicate<DB>>,
    },
}
