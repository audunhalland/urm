use urm::prelude::*;
use urm::*;

use urm_graphql;

pub mod sql {
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
pub struct Publication(Node<sql::Publication>);

pub struct Edition(Node<sql::Edition>);

//#[DbObject]
impl Publication {
    /// Could do shorthand macro here:
    /// #[project(sql::Publication::id)]
    pub async fn id(self) -> UrmResult<String> {
        self.0.project(sql::Publication::id()).await
    }

    // A GraphQL query we want to resolve to SQL
    pub async fn editions(self, range: std::ops::Range<usize>) -> UrmResult<Vec<Edition>> {
        let (_id, editions) = self
            .0
            .project((
                sql::Publication::id(),
                sql::Publication::editions(range).map(Edition),
            ))
            .await?;

        Ok(editions)
    }
}

// #[DbObject]
impl Edition {
    pub async fn publication(self) -> UrmResult<Publication> {
        Ok(self
            .0
            .project(sql::Edition::publication().map(Publication))
            .await?)
    }
}

pub struct Query;

// 'regular' GraphQL
impl Query {
    // Root query, where the urm query gets 'initialized'
    pub async fn editions(self) -> UrmResult<Vec<Edition>> {
        urm_graphql::select::<sql::Edition>().map(Edition).await
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
