use nako_core::*;
use nako_search::{SearchDocument, SearchIndex, SearchQuery};

use crate::NakoDatabase;

async fn migrated_store() -> NakoDatabase {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    store
}

fn movie_item(id: MediaItemId, title: &str) -> MediaItem {
    MediaItem {
        id,
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    }
}

async fn upsert_indexed_movie(
    store: &NakoDatabase,
    title: &str,
    body: &str,
    facets: Vec<String>,
    aliases: Vec<String>,
) -> MediaItemId {
    let item_id = MediaItemId::new();
    let item = movie_item(item_id, title);
    let mut document = SearchDocument::from_facet_labels(item_id, title, body, facets).unwrap();
    document.aliases = aliases;

    store.upsert_media_item(&item).await.unwrap();
    store.upsert(document).await.unwrap();

    item_id
}

#[tokio::test]
async fn nako_database_sqlite_matches_browse_facets_exactly() {
    let store = migrated_store().await;
    let item_id = upsert_indexed_movie(
        &store,
        "Exact Facet Fixture",
        "semantic search fixture",
        vec!["genre:Science Fiction".to_owned()],
        Vec::new(),
    )
    .await;

    let partial = store
        .search(
            SearchQuery::from_facet_labels("fixture", vec!["genre:Science".to_owned()], 10, 0)
                .unwrap(),
        )
        .await
        .unwrap();
    let exact = store
        .search(
            SearchQuery::from_facet_labels(
                "fixture",
                vec!["genre:Science Fiction".to_owned()],
                10,
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();

    assert!(partial.is_empty());
    assert_eq!(exact[0].item_id, item_id);
}

#[tokio::test]
async fn nako_database_sqlite_uses_shared_search_semantics_for_cjk_aliases() {
    let store = migrated_store().await;
    let item_id = upsert_indexed_movie(
        &store,
        "Spirited Away",
        "宫崎骏动画",
        vec!["provider:bangumi".to_owned()],
        vec!["千と千尋の神隠し".to_owned()],
    )
    .await;

    let hits = store
        .search(
            SearchQuery::from_facet_labels("千 尋", vec!["provider:bangumi".to_owned()], 10, 0)
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(hits[0].item_id, item_id);
}

#[tokio::test]
async fn nako_database_sqlite_searches_aliases_but_keeps_them_structured() {
    let store = migrated_store().await;
    let item_id = upsert_indexed_movie(
        &store,
        "Alias Fixture",
        "primary body",
        vec!["genre:Drama".to_owned()],
        vec!["Hidden Original Title".to_owned()],
    )
    .await;

    let hits = store
        .search(
            SearchQuery::from_facet_labels(
                "hidden original",
                vec!["genre:Drama".to_owned()],
                10,
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(hits[0].item_id, item_id);
}
