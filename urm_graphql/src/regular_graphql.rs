use async_graphql::*;
use async_trait::*;

struct Test;

#[async_graphql::Object]
impl Test {
    async fn rec(&self) -> Test {
        Test
    }
}
