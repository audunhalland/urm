use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;

use crate::field;
use crate::query;
use crate::{Table, UrmError, UrmResult};

pub struct QueryEngine {
    root_select: Select,
    setup_done_rx: mpsc::Receiver<()>,
    setup_done_tx: mpsc::Sender<()>,
    query_done: Arc<Semaphore>,
}

impl QueryEngine {
    pub fn new_select(from: &'static dyn Table) -> (Self, ProjectionSetup) {
        let (setup_done_tx, setup_done_rx) = mpsc::channel(1);
        let query_done = Arc::new(Semaphore::new(0));

        let projection = Arc::new(Mutex::new(Projection {
            fields: BTreeMap::new(),
        }));

        let query_engine = Self {
            root_select: Select {
                from,
                predicate: None,
                projection: projection.clone(),
            },
            setup_done_rx,
            setup_done_tx: setup_done_tx.clone(),
            query_done: query_done.clone(),
        };

        (
            query_engine,
            ProjectionSetup {
                projection,
                setup_done_tx,
                query_done,
            },
        )
    }

    pub async fn execute(mut self) -> UrmResult<()> {
        // Do this as many times as necessary:
        self.setup_done_rx.recv().await;

        let mut builder = query::PGQueryBuilder::new();
        self.root_select.build_query(&mut builder);
        // TODO Execute here

        // BUG: Add actual number of waiters
        self.query_done.add_permits(999999999);

        panic!();
    }
}

/// Each ProjectionSetup the QueryEngine
/// hands out must be completed by
/// calling the `complete` method.
pub struct ProjectionSetup {
    projection: Arc<Mutex<Projection>>,
    setup_done_tx: mpsc::Sender<()>,
    query_done: Arc<Semaphore>,
}

impl ProjectionSetup {
    pub fn projection(&self) -> &Arc<Mutex<Projection>> {
        &self.projection
    }

    /// Consume this setup and wait for the result.
    pub async fn complete(self) -> UrmResult<()> {
        self.setup_done_tx
            .send(())
            .await
            .map_err(|_| UrmError::Synchronization)?;
        self.query_done
            .acquire()
            .await
            .map_err(|_| UrmError::Synchronization)?
            .forget();

        Ok(())
    }
}

/// # Select
///
/// Encodes the intent of selecting *something* from a table.
///
/// It's an abstraction over an "SQL"-like select expression.
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

pub struct Projection {
    fields: BTreeMap<field::LocalId, QueryField>,
}

impl Projection {
    pub fn project_basic_field(&mut self, local_id: field::LocalId) {
        self.fields.insert(local_id, QueryField::Basic);
    }
}

enum QueryField {
    Basic,
    Foreign(Select),
}
