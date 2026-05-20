use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{BrowseFacet, MediaItemId, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchDocument {
    pub item_id: MediaItemId,
    pub projection_version: u16,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub browse_facets: Vec<BrowseFacet>,
}

impl SearchDocument {
    #[must_use]
    pub fn from_facet_labels(
        item_id: MediaItemId,
        title: impl Into<String>,
        body: impl Into<String>,
        facets: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            item_id,
            projection_version: taru_core::CATALOG_SEARCH_PROJECTION_VERSION,
            title: title.into(),
            body: body.into(),
            aliases: Vec::new(),
            browse_facets: facets
                .into_iter()
                .map(|facet| BrowseFacet::parse_label(&facet))
                .collect::<Result<Vec<_>>>()?,
        })
    }

    #[must_use]
    pub fn facet_labels(&self) -> Vec<String> {
        self.browse_facets.iter().map(BrowseFacet::label).collect()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchQuery {
    pub query: String,
    #[serde(default)]
    pub browse_facets: Vec<BrowseFacet>,
    pub limit: u32,
    pub offset: u32,
}

impl SearchQuery {
    #[must_use]
    pub fn from_facet_labels(
        query: impl Into<String>,
        facets: Vec<String>,
        limit: u32,
        offset: u32,
    ) -> Result<Self> {
        Ok(Self {
            query: query.into(),
            browse_facets: facets
                .into_iter()
                .map(|facet| BrowseFacet::parse_label(&facet))
                .collect::<Result<Vec<_>>>()?,
            limit,
            offset,
        })
    }

    #[must_use]
    pub fn facet_labels(&self) -> Vec<String> {
        self.browse_facets.iter().map(BrowseFacet::label).collect()
    }
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

#[cfg(test)]
mod tests {
    use taru_core::{BrowseFacet, BrowseFacetKind, MediaItemId};

    use super::{SearchDocument, SearchQuery};

    #[test]
    fn search_query_uses_structured_browse_facets_not_raw_strings() {
        let query = SearchQuery {
            query: "matrix".to_owned(),
            browse_facets: vec![BrowseFacet::new(
                BrowseFacetKind::ExternalId("tmdb".to_owned()),
                "603",
            )],
            limit: 10,
            offset: 0,
        };

        assert_eq!(query.facet_labels(), vec!["external_id:tmdb:603"]);
    }

    #[test]
    fn legacy_facet_labels_are_parsed_into_semantic_facets() {
        let document = SearchDocument::from_facet_labels(
            MediaItemId::new(),
            "The Matrix",
            "wake up",
            vec!["genre:Science Fiction".to_owned()],
        )
        .unwrap();

        assert_eq!(document.facet_labels(), vec!["genre:Science Fiction"]);
    }
}
