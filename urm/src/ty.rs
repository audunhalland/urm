use crate::quantify;
use crate::Database;

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

/// Any Rust type that should be interpreted as having an urm `Type`
pub trait Typed<DB: Database> {
    type Ty: Type;
}

/// 'FlatMap' some Type into the type `U`
/// having the desired quantification.
pub trait MapTo<U>: Type {
    type Quantify: quantify::Quantify<U>;
}

/// Non-nullable unit type
pub struct Unit<T>(std::marker::PhantomData<T>);

impl<T> Type for Unit<T>
where
    T: Send + Sync + 'static,
{
    type Unit = T;
    type Output = T;
}

impl<T, U> MapTo<U> for Unit<T>
where
    T: Send + Sync + 'static,
{
    type Quantify = quantify::AsSelf;
}

/// Nullable type
pub struct Nullable<U>(std::marker::PhantomData<U>);

impl<T> Type for Nullable<T>
where
    T: Send + Sync + 'static,
{
    type Unit = T;
    type Output = Option<T>;
}

impl<T, U> MapTo<U> for Nullable<T>
where
    T: Send + Sync + 'static,
{
    type Quantify = quantify::AsOption;
}

pub struct Bool;

impl Type for Bool {
    type Unit = bool;
    type Output = bool;
}
