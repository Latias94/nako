use std::cmp::Ordering;

use async_trait::async_trait;
use nako_core::{BrowseFacet, MediaItemId, PageRequest, Result};
use serde::{Deserialize, Serialize};

const TITLE_MATCH_SCORE: f32 = 1.0;
const ALIAS_MATCH_SCORE: f32 = 0.9;
const BODY_MATCH_SCORE: f32 = 0.7;
const FACET_MATCH_SCORE: f32 = 0.5;

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
            projection_version: nako_core::CATALOG_SEARCH_PROJECTION_VERSION,
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

    #[must_use]
    pub const fn uses_current_projection_version(&self) -> bool {
        self.projection_version == current_projection_version()
    }
}

#[must_use]
pub const fn current_projection_version() -> u16 {
    nako_core::CATALOG_SEARCH_PROJECTION_VERSION
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchEvaluationDocument {
    pub item_id: MediaItemId,
    pub projection_version: u16,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub facet_labels: Vec<String>,
}

impl SearchEvaluationDocument {
    #[must_use]
    pub fn from_search_document(document: SearchDocument) -> Self {
        let facet_labels = document.facet_labels();
        Self {
            item_id: document.item_id,
            projection_version: document.projection_version,
            title: document.title,
            body: document.body,
            aliases: document.aliases,
            facet_labels,
        }
    }

    #[must_use]
    pub fn from_facet_labels(
        item_id: MediaItemId,
        projection_version: u16,
        title: impl Into<String>,
        body: impl Into<String>,
        aliases: Vec<String>,
        facet_labels: Vec<String>,
    ) -> Self {
        Self {
            item_id,
            projection_version,
            title: title.into(),
            body: body.into(),
            aliases,
            facet_labels,
        }
    }

    #[must_use]
    pub const fn uses_current_projection_version(&self) -> bool {
        self.projection_version == current_projection_version()
    }
}

#[must_use]
pub fn evaluate_search_documents(
    query: &SearchQuery,
    documents: impl IntoIterator<Item = SearchEvaluationDocument>,
) -> Vec<SearchHit> {
    let required_facets = query
        .facet_labels()
        .into_iter()
        .map(|facet| normalized_facet_label(&facet))
        .collect::<Vec<_>>();
    let normalized_query = NormalizedSearchText::new(&query.query);
    let page = PageRequest::new(query.limit, u64::from(query.offset)).clamped();

    let mut hits = documents
        .into_iter()
        .filter(|document| {
            required_facets.iter().all(|required| {
                document
                    .facet_labels
                    .iter()
                    .any(|facet| normalized_facet_label(facet) == *required)
            })
        })
        .filter_map(|document| {
            score_document(&document, &normalized_query).map(|score| SearchHit {
                item_id: document.item_id,
                score,
            })
        })
        .collect::<Vec<_>>();

    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.item_id.cmp(&right.item_id))
    });

    hits.into_iter()
        .skip(page.offset as usize)
        .take(page.limit as usize)
        .collect()
}

fn score_document(
    document: &SearchEvaluationDocument,
    query: &NormalizedSearchText,
) -> Option<f32> {
    if query.is_empty() {
        return Some(FACET_MATCH_SCORE);
    }

    if search_text_matches(&document.title, query) {
        return Some(TITLE_MATCH_SCORE);
    }

    if document
        .aliases
        .iter()
        .any(|alias| search_text_matches(alias, query))
    {
        return Some(ALIAS_MATCH_SCORE);
    }

    if search_text_matches(&document.body, query) {
        return Some(BODY_MATCH_SCORE);
    }

    if document
        .facet_labels
        .iter()
        .any(|facet| search_text_matches(facet, query))
    {
        return Some(FACET_MATCH_SCORE);
    }

    None
}

fn search_text_matches(value: &str, query: &NormalizedSearchText) -> bool {
    let value = NormalizedSearchText::new(value);
    value.spaced.contains(&query.spaced) || value.compact.contains(&query.compact)
}

fn normalized_facet_label(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedSearchText {
    spaced: String,
    compact: String,
}

impl NormalizedSearchText {
    fn new(value: &str) -> Self {
        let mut spaced = String::new();
        let mut previous_was_separator = false;

        for character in value.chars().flat_map(char::to_lowercase) {
            if character.is_alphanumeric() {
                spaced.push(character);
                previous_was_separator = false;
            } else if !previous_was_separator && !spaced.is_empty() {
                spaced.push(' ');
                previous_was_separator = true;
            }
        }

        if previous_was_separator {
            spaced.pop();
        }

        let compact = spaced
            .chars()
            .filter(|character| *character != ' ')
            .collect();
        Self { spaced, compact }
    }

    fn is_empty(&self) -> bool {
        self.spaced.is_empty()
    }
}

#[async_trait]
pub trait SearchIndex: Send + Sync {
    async fn upsert(&self, document: SearchDocument) -> Result<()>;

    async fn delete(&self, item_id: MediaItemId) -> Result<()>;

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>>;
}

#[cfg(test)]
mod tests {
    use nako_core::{BrowseFacet, BrowseFacetKind, MediaItemId};

    use super::{
        SearchDocument, SearchEvaluationDocument, SearchQuery, current_projection_version,
        evaluate_search_documents,
    };

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

    #[test]
    fn search_documents_use_current_catalog_projection_version() {
        let document = SearchDocument::from_facet_labels(
            MediaItemId::new(),
            "The Matrix",
            "wake up",
            Vec::new(),
        )
        .unwrap();

        assert_eq!(document.projection_version, current_projection_version());
        assert!(document.uses_current_projection_version());
    }

    #[test]
    fn search_evaluation_documents_expose_projection_version_freshness() {
        let current = SearchEvaluationDocument::from_facet_labels(
            MediaItemId::new(),
            current_projection_version(),
            "Current",
            "",
            Vec::new(),
            Vec::new(),
        );
        let stale = SearchEvaluationDocument::from_facet_labels(
            MediaItemId::new(),
            current_projection_version() - 1,
            "Stale",
            "",
            Vec::new(),
            Vec::new(),
        );

        assert!(current.uses_current_projection_version());
        assert!(!stale.uses_current_projection_version());
    }

    #[test]
    fn search_evaluation_scores_title_alias_body_and_filters_facets() {
        let title_item = MediaItemId::new();
        let alias_item = MediaItemId::new();
        let body_item = MediaItemId::new();
        let wrong_facet_item = MediaItemId::new();
        let hits = evaluate_search_documents(
            &SearchQuery::from_facet_labels(
                "matrix",
                vec!["genre:Science Fiction".to_owned()],
                10,
                0,
            )
            .unwrap(),
            vec![
                SearchEvaluationDocument::from_facet_labels(
                    body_item,
                    current_projection_version(),
                    "Body Fixture",
                    "The Matrix is in the overview",
                    Vec::new(),
                    vec!["genre:Science Fiction".to_owned()],
                ),
                SearchEvaluationDocument::from_facet_labels(
                    alias_item,
                    current_projection_version(),
                    "Alias Fixture",
                    "overview",
                    vec!["The Matrix Original".to_owned()],
                    vec!["genre:Science Fiction".to_owned()],
                ),
                SearchEvaluationDocument::from_facet_labels(
                    title_item,
                    current_projection_version(),
                    "The Matrix",
                    "overview",
                    Vec::new(),
                    vec!["genre:Science Fiction".to_owned()],
                ),
                SearchEvaluationDocument::from_facet_labels(
                    wrong_facet_item,
                    current_projection_version(),
                    "The Matrix Reloaded",
                    "overview",
                    Vec::new(),
                    vec!["genre:Action".to_owned()],
                ),
            ],
        );

        assert_eq!(
            hits.iter().map(|hit| hit.item_id).collect::<Vec<_>>(),
            vec![title_item, alias_item, body_item]
        );
        assert!(hits[0].score > hits[1].score);
        assert!(hits[1].score > hits[2].score);
    }

    #[test]
    fn search_evaluation_matches_cjk_queries_without_whitespace_exactness() {
        let item_id = MediaItemId::new();
        let hits = evaluate_search_documents(
            &SearchQuery::from_facet_labels("千 尋", vec!["provider:bangumi".to_owned()], 10, 0)
                .unwrap(),
            vec![SearchEvaluationDocument::from_facet_labels(
                item_id,
                current_projection_version(),
                "Spirited Away",
                "宫崎骏动画",
                vec!["千と千尋の神隠し".to_owned()],
                vec!["provider:bangumi".to_owned()],
            )],
        );

        assert_eq!(hits[0].item_id, item_id);
    }
}
