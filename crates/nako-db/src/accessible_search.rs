use nako_core::{AuthenticatedPrincipal, Result};
use nako_search::{SearchHit, SearchQuery};

#[async_trait::async_trait]
pub(crate) trait AccessibleSearchIndex: Send + Sync {
    async fn search_accessible(
        &self,
        principal: &AuthenticatedPrincipal,
        query: SearchQuery,
    ) -> Result<Vec<SearchHit>>;
}
