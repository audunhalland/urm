use async_trait::*;

use crate::UrmResult;

///
/// The 'probe' procedure involves eagerly walking a tree
/// to figure out the static structure of a query.
///
#[async_trait]
pub trait Probe {
    #[cfg(feature = "async_graphql")]
    async fn probe(&self, ctx: &::async_graphql::context::Context<'_>) -> UrmResult<()>;
}
