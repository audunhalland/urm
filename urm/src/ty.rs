use crate::quantify::Quantify;

/// Represents the output type/result of a projection.
/// Because the result of a projection may be a unit type
/// or a collection type, the `Type` trait associates both
/// types at once, so they can more easily be mapped in more intricate ways.
pub trait Type: Sized + Send + Sync + 'static {
    /// Unit (unquantified) output of this type (i.e. no Option, Vec)
    type Unit: Send + Sync + 'static;

    /// Final, quantified output of this outcome (possibly `Vec<Self::Unit>` or some other collection).
    type Output: Send + Sync + 'static;
}

/// 'FlatMap' some Type into the type `U`
/// having the desired quantification.
pub trait FlatMapTo<U>: Type {
    type Quantify: Quantify<U>;
}
