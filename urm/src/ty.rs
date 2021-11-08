use crate::database::Database;
use crate::quantify;

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

/// Trait implemented for types that are scalar (i.e. not a vector/collection)
pub trait ScalarType: Type {}

/// Trait implemented for types that are vector-valued
pub trait VectorType: Type {}

/// Any Rust type that should be interpreted as having an urm `Type`
pub trait Typed<DB: Database> {
    type Ty: Type;
}

pub trait ScalarTyped<DB, U> {}

impl<T, DB, U> ScalarTyped<DB, U> for T
where
    DB: Database,
    T: Typed<DB>,
    T::Ty: ScalarType<Unit = U>,
{
}

pub trait VectorTyped<DB, U> {}

impl<T, DB, U> VectorTyped<DB, U> for T
where
    DB: Database,
    T: Typed<DB>,
    T::Ty: VectorType<Unit = U>,
{
}

/// 'FlatMap' some Type into the type `U`
/// having the desired quantification.
pub trait MapTo<U>: Type {
    type Quantify: quantify::Quantify<U>;
}

/// Non-nullable unit type
pub struct Unit<U>(std::marker::PhantomData<U>);

impl<U> Type for Unit<U>
where
    U: Send + Sync + 'static,
{
    type Unit = U;
    type Output = U;
}

impl<U> ScalarType for Unit<U> where U: Send + Sync + 'static {}

impl<U, V> MapTo<V> for Unit<U>
where
    U: Send + Sync + 'static,
{
    type Quantify = quantify::AsSelf;
}

/// Nullable type
pub struct Nullable<U>(std::marker::PhantomData<U>);

impl<U> Type for Nullable<U>
where
    U: Send + Sync + 'static,
{
    type Unit = U;
    type Output = Option<U>;
}

impl<U> ScalarType for Nullable<U> where U: Send + Sync + 'static {}

impl<U, V> MapTo<V> for Nullable<U>
where
    U: Send + Sync + 'static,
{
    type Quantify = quantify::AsOption;
}

pub struct Vector<U>(std::marker::PhantomData<U>);

impl<U> Type for Vector<U>
where
    U: Send + Sync + 'static,
{
    type Unit = U;
    type Output = Vec<U>;
}

impl<U> VectorType for Vector<U> where U: Send + Sync + 'static {}

impl<U, V> MapTo<V> for Vector<U>
where
    U: Send + Sync + 'static,
{
    type Quantify = quantify::AsSelf;
}

/// Type that always resolves to 'no value', i.e. Option::None
pub struct Void<U>(std::marker::PhantomData<U>);

impl<U> Void<U> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<U> Type for Void<U>
where
    U: Send + Sync + 'static,
{
    type Unit = U;
    type Output = Option<U>;
}

impl<T> ScalarType for Void<T> where T: Send + Sync + 'static {}

impl<DB, U> Typed<DB> for Void<U>
where
    DB: Database,
    U: Send + Sync + 'static,
{
    type Ty = Self;
}

impl<DB, T> Typed<DB> for Option<T>
where
    DB: Database,
    T: Typed<DB>,
{
    type Ty = Nullable<<T::Ty as Type>::Unit>;
}
