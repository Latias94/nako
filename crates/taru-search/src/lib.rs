use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{MediaItemId, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchDocument {
    pub item_id: MediaItemId,
    pub title: String,
    pub body: String,
    pub facets: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchQuery {
    pub query: String,
    pub facets: Vec<String>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchHit {
    pub item_id: MediaItemId,
    pub score: f32,
}

#[async_trait]
pub trait SearchIndex: Send + Sync {
    async fn upsert(&self, document: SearchDocument) -> Result<()>;

    async fn delete(&self, item_id: MediaItemId) -> Result<()>;

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>>;
}
