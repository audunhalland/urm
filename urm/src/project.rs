//!
//! Things related to data projection, the shape of returned data.
//!

use crate::engine::Probing;
use crate::ty::Type;
use crate::{Database, Table, UrmResult};

pub mod foreign;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

/// # ProjectFrom
///
/// Types starting out as projection builders implement this trait.
///
/// To "project" usually means selecting specific columns from a table.
///
pub trait ProjectFrom: Sized + Send + Sync {
    /// The table projected from
    type Table: Table;

    /// Data type produced by the projection
    type Ty: Type;
}

/// ProjectAndProbe is the trait that is implemented for types
/// that are ready to be projected and/or probed.
///
/// This trait is the final typestate of a single projection, which is
/// no ready to be processed by `urm::project`.
///
/// Not all `ProjectFrom` types implement `ProjectAndProbe`, and
/// may need further mapping before reaching this typestate.
pub trait ProjectAndProbe<DB: Database> {
    fn project_and_probe(self, probing: &Probing<DB>) -> UrmResult<()>;
}
