#[cfg(feature = "postgres")]
mod postgres;

pub trait Database: std::fmt::Debug + Sync + Send + Clone + 'static {}

#[cfg(feature = "postgres")]
pub use postgres::Postgres;
