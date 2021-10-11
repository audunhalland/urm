use urm::*;

pub mod sql {
    use urm::*;

    pub struct Publication;
    pub struct Edition;
    pub struct Module;
    pub struct Contribution;
    pub struct Contributor;

    #[table("publication")]
    impl Table for Publication {
        fn id() -> String;

        #[foreign(Edition(publication_id) => Self(id))]
        fn editions(_: std::ops::Range<usize>) -> [Edition];
    }

    #[table("edition")]
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
        let id = self.0.project(sql::Publication::id()).await?;

        Ok(id)
    }

    // A GraphQL query we want to resolve to SQL
    pub async fn editions(self, range: std::ops::Range<usize>) -> UrmResult<Vec<Edition>> {
        let (_id, editions) = self
            .0
            .project((sql::Publication::id(), sql::Publication::editions(range)))
            .await?;

        Ok(editions.into_iter().map(|node| Edition(node)).collect())
    }
}

#[DbObject]
impl Edition {
    fn publication(table: &sql::Edition) -> Publication {}
}

pub struct Query {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
