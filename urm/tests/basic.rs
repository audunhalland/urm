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
        fn publication_id() -> String;

        #[foreign(Self(publication_id) => Publication(id))]
        fn publication() -> Publication;
    }
}

/// GraphQL section

/// This object might either be the "root",
/// or it may be the child of an edition.
#[derive(Probe)]
pub struct Publication(Node<db::Publication>);

#[derive(Probe)]
pub struct Edition(Node<db::Edition>);

#[async_graphql::Object]
impl Publication {
    /// Could do shorthand macro here:
    /// #[project(sql::Publication::id)]
    pub async fn id(&self) -> UrmResult<String> {
        urm::project(self, db::Publication::id()).await
    }

    // A GraphQL query we want to resolve to SQL
    pub async fn editions(
        &self,
        ctx: &::async_graphql::Context<'_>,
        first: Option<usize>,
        offset: Option<usize>,
    ) -> UrmResult<Vec<Edition>> {
        let (_id, editions) = urm::project(
            self,
            (
                db::Publication::id(),
                db::Publication::editions(offset.unwrap_or(0)..first.unwrap_or(20))
                    .probe_with(Edition, ctx),
            ),
        )
        .await?;

        Ok(editions)
    }
}

#[async_graphql::Object]
impl Edition {
    pub async fn publication(&self, ctx: &::async_graphql::Context<'_>) -> UrmResult<Publication> {
        urm::project(
            self,
            db::Edition::publication().probe_with(Publication, ctx),
        )
        .await
    }
}

pub struct Query;

// 'regular' GraphQL
#[async_graphql::Object]
impl Query {
    // Root query, where the urm query gets 'initialized'
    pub async fn editions(&self, ctx: &::async_graphql::Context<'_>) -> UrmResult<Vec<Edition>> {
        select::<db::Edition>().probe_with(Edition, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_test() -> UrmResult<()> {
        let schema = async_graphql::Schema::new(
            Query,
            async_graphql::EmptyMutation,
            async_graphql::EmptySubscription,
        );

        let response = schema
            .execute(
                r#"{
                    editions {
                        publication {
                            id
                        }
                    }
                }"#,
            )
            .await;

        println!("data: {}", response.data);
        println!("errors: {:?}", response.errors);

        panic!()
    }
}
