use urm::prelude::*;
use urm::*;

pub mod db {
    pub struct Publication;
    pub struct Edition;
    pub struct Module;
    pub struct Contribution;
    pub struct Contributor;

    #[urm::table("publication")]
    impl Table for Publication {
        fn id() -> String;

        #[foreign(Edition(publication_id) => Self(id))]
        fn editions(_: std::ops::Range<usize>) -> [Edition];
    }

    #[urm::table("edition")]
    impl Table for Edition {
        #[foreign(Self(publication_id) => Publication(id))]
        fn publication() -> Publication;
    }
}

/// GraphQL section

/// This object might either be the "root",
/// or it may be the child of an edition.
pub struct Publication(Node<db::Publication>);

pub struct Edition(Node<db::Edition>);

//#[DbObject]
impl Publication {
    /// Could do shorthand macro here:
    /// #[project(sql::Publication::id)]
    pub async fn id(self, ctx: &::async_graphql::context::Context<'_>) -> UrmResult<String> {
        self.0.project(&db::Publication::id()).await
    }

    // A GraphQL query we want to resolve to SQL
    pub async fn editions(
        self,
        ctx: &::async_graphql::context::Context<'_>,
        range: std::ops::Range<usize>,
    ) -> UrmResult<Vec<Edition>> {
        let (_id, editions) = self
            .0
            .project(&(
                db::Publication::id(),
                db::Publication::editions(range).probe_with(Edition, ctx),
            ))
            .await?;

        Ok(editions)
    }
}

// #[DbObject]
impl Edition {
    pub async fn publication(
        self,
        ctx: &::async_graphql::context::Context<'_>,
    ) -> UrmResult<Publication> {
        Ok(self
            .0
            .project(&db::Edition::publication().probe_with(Publication, ctx))
            .await?)
    }
}

mod gql_test_hack {
    use super::*;

    impl urm::Probe for Publication {
        fn probe(&self, ctx: &::async_graphql::context::Context<'_>) {}
    }

    impl ::async_graphql::Type for Edition {
        fn type_name() -> std::borrow::Cow<'static, str> {
            panic!()
        }

        fn create_type_info(registry: &mut ::async_graphql::registry::Registry) -> String {
            panic!()
        }
    }

    #[::async_trait::async_trait]
    impl ::async_graphql::OutputType for Edition {
        async fn resolve(
            &self,
            ctx: &::async_graphql::context::ContextSelectionSet<'_>,
            _field: &::async_graphql::Positioned<::async_graphql::parser::types::Field>,
        ) -> ::async_graphql::ServerResult<::async_graphql::Value> {
            ::async_graphql::resolver_utils::resolve_container(ctx, self).await
        }
    }

    // test impl
    #[::async_trait::async_trait]
    impl ::async_graphql::resolver_utils::ContainerType for Edition {
        async fn resolve_field(
            &self,
            ctx: &::async_graphql::context::Context<'_>,
        ) -> ::async_graphql::ServerResult<Option<::async_graphql::Value>> {
            let obj = Self(
                self.0
                    .clone_setup()
                    .map_err(|_| ::async_graphql::ServerError::new("Bad state", None))?,
            );

            if ctx.item.node.name.node == "publication" {
                let _ans = obj.publication(ctx).await;
            }

            panic!()
        }
    }

    impl urm::Probe for Edition {
        fn probe(&self, ctx: &::async_graphql::context::Context<'_>) {}
    }
}

pub struct Query;

// 'regular' GraphQL
impl Query {
    // Root query, where the urm query gets 'initialized'
    pub async fn editions(self) -> UrmResult<Vec<Edition>> {
        probe_select::<db::Edition>().map(Edition).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_test() -> UrmResult<()> {
        // Let's say we query:
        // {
        //    editions(...) {
        //       publication { id }
        //    }
        // }

        let query = Query;
        let editions = query.editions().await?;

        Ok(())
    }
}
