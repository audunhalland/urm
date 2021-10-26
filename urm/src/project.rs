use crate::engine::Probing;
use crate::{Table, UrmResult};

pub mod foreign;
pub mod primitive;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

///
/// Anything that can be projected
///
pub trait Projectable: Sized + Send + Sync {
    /// The table projected from
    type Table: Table;

    /// The mechanics which determines the complexity of the projection
    type Mechanics: ProjectionMechanics;
}

/// Field mechanics
pub trait ProjectionMechanics: Sized + Send + Sync + 'static {
    /// Unit of this field type, in case Output is quantified
    type Unit: Send + Sync + 'static;

    /// Final, quantified value of the field (possibly Vec<Self::Unit>).
    type Output: Send + Sync + 'static;
}

/// Something that can be probe-projected directly
pub trait ProjectAndProbe {
    fn project_and_probe(self, probing: &Probing) -> UrmResult<()>;
}
