#[cfg(feature = "postgres")]
pub mod postgres;

pub trait Database: std::fmt::Debug + Sync + Send + Clone + 'static {}
