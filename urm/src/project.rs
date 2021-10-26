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
pub trait ProjectFrom: Sized + Send + Sync {
    /// The table projected from
    type Table: Table;

    /// Outcome/value produced by the projection
    type Outcome: Outcome;
}

/// # Projection outcome
///
/// Represents the produced value of a projection.
pub trait Outcome: Sized + Send + Sync + 'static {
    /// Unit (unquantified) output of this outcome
    type Unit: Send + Sync + 'static;

    /// Final, quantified output of this outcome (possibly Vec<Self::Unit> or some other collection).
    type Output: Send + Sync + 'static;
}

/// # ProjectAndProbe
///
/// Any type representing a fully built projection must implement this trait.
pub trait ProjectAndProbe {
    fn project_and_probe(self, probing: &Probing) -> UrmResult<()>;
}
