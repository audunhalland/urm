use crate::engine::Probing;
use crate::{Table, UrmResult};

pub mod foreign;
pub mod primitive;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalId(pub u16);

///
/// A field having some data type, that can be found in some database.
///
pub trait Field: Sized + Send + Sync {
    /// The table that owns this field
    type Table: Table;

    /// The field 'mechanics', which determines how the field
    /// behaves in the API
    type Mechanics: FieldMechanics;
}

/// Field mechanics
pub trait FieldMechanics: Sized + Send + Sync + 'static {
    /// Unit of this field type, in case Output is quantified
    type Unit: Send + Sync + 'static;

    /// Final, quantified value of the field (possibly Vec<Self::Unit>).
    type Output: Send + Sync + 'static;
}

/// Something that can be probe-projected directly
pub trait ProjectAndProbe {
    fn project_and_probe(self, probing: &Probing) -> UrmResult<()>;
}
