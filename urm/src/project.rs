//!
//! Things related to data projection, the shape of returned data.
//!

use crate::engine::Probing;
use crate::{Table, UrmResult};

pub mod foreign;
pub mod primitive;

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

    /// Outcome/value produced by the projection
    type Outcome: Outcome;
}

/// Represents the output type/result of a projection.
/// Because the result of a projection may be a unit type
/// or a collection type, the `Outcome` trait associates both
/// types at once, so they can more easily be mapped in more intricate ways.
pub trait Outcome: Sized + Send + Sync + 'static {
    /// Unit (unquantified) output of this outcome
    type Unit: Send + Sync + 'static;

    /// Final, quantified output of this outcome (possibly `Vec<Self::Unit>` or some other collection).
    type Output: Send + Sync + 'static;
}

/// ProjectAndProbe is the trait that is implemented for types
/// that are ready to be projected and/or probed.
///
/// This trait is the final typestate of a single projection, which is
/// no ready to be processed by `urm::project`.
///
/// Not all `ProjectFrom` types implement `ProjectAndProbe`, and
/// may need further mapping before reaching this typestate.
pub trait ProjectAndProbe {
    fn project_and_probe(self, probing: &Probing) -> UrmResult<()>;
}
