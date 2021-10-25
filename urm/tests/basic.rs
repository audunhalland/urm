use urm::prelude::*;

pub mod db {
    use urm::prelude::*;

    pub struct Publication;
    pub struct Edition;
    pub struct Module;
    pub struct Contribution;
    pub struct Contributor;

    #[urm::table("publication")]
    impl Publication {
        fn all() -> [Self];

        fn id(self) -> String;

        #[foreign(Edition(publication_id) => Self(id))]
        fn editions(self) -> [Edition];
    }

    #[urm::table("edition")]
    impl Edition {
        fn all() -> [Self];

        fn id(self) -> String;
        fn publication_id(self) -> String;

        #[foreign(Self(publication_id) => Publication(id))]
        fn publication(self) -> Publication;
    }
}

/// GraphQL section

/// This object might either be the "root",
/// or it may be the child of an edition.
#[derive(urm::Probe)]
pub struct Publication(urm::Node<db::Publication>);

#[derive(urm::Probe)]
pub struct Edition(urm::Node<db::Edition>);

#[async_graphql::Object]
impl Publication {
    pub async fn id(&self) -> urm::UrmResult<String> {
        urm::project(self, db::Publication.id()).await
    }

    // A GraphQL query we want to resolve to SQL
    pub async fn editions(
        &self,
        ctx: &::async_graphql::Context<'_>,
        first: Option<usize>,
        offset: Option<usize>,
    ) -> urm::UrmResult<Vec<Edition>> {
        let (_id, editions) = urm::project(
            self,
            (
                db::Publication.id(),
                db::Publication
                    .editions()
                    .range(offset..first.or(Some(20)))
                    .probe_with(Edition, ctx),
            ),
        )
        .await?;

        Ok(editions)
    }
}

#[async_graphql::Object]
impl Edition {
    pub async fn publication(
        &self,
        ctx: &::async_graphql::Context<'_>,
    ) -> urm::UrmResult<Publication> {
        urm::project(self, db::Edition.publication().probe_with(Publication, ctx)).await
    }
}

pub struct Query;

// 'regular' GraphQL
#[async_graphql::Object]
impl Query {
    // Root query, where the urm query gets 'initialized'
    pub async fn editions(
        &self,
        ctx: &::async_graphql::Context<'_>,
    ) -> urm::UrmResult<Vec<Edition>> {
        urm::select().range(0..20).probe_with(Edition, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_test() -> urm::UrmResult<()> {
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
