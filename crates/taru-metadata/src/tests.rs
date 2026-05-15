use std::sync::{
    Arc, Mutex as StdMutex,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap as AxumHeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use reqwest::header::HeaderMap;
use serde_json::json;
use taru_core::{
    CanonicalMetadata, CatalogRepository, ContentRating, CreditRole, ExternalId, ExternalProvider,
    ImageKind, JobId, JobKind, JobRepository, Library, LibraryId, LibraryOptions, LibraryPreset,
    LibraryRepository, MediaItem, MediaItemId, MediaKind, MediaRepository, MetadataField,
    MetadataFieldLock, MetadataMatchKind, MetadataProfile, MetadataProviderAttemptStatus,
    MetadataProviderErrorClass, MetadataRefreshMode, MetadataRepository, MetadataSource, NewJob,
    PageRequest, Result, TaruError, TransactionManager,
};
use taru_db::SqliteStore;
use taru_search::{SearchIndex, SearchQuery};
use tokio::{net::TcpListener, time::Instant};

use super::*;
use crate::providers::{
    BangumiSubject, DEFAULT_BANGUMI_IMAGE_BASE_URL, DEFAULT_TMDB_IMAGE_BASE_URL, DoubanSubject,
    TmdbMovieDetails, bangumi_subject_to_metadata, douban_subject_to_metadata,
    tmdb_movie_details_to_metadata,
};

mod fixtures;
#[test]
fn merge_preserves_locked_fields() {
    let item_id = MediaItemId::new();
    let policy = MetadataMergePolicy::from_locks(&[
        MetadataFieldLock {
            item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        },
        MetadataFieldLock {
            item_id,
            field: MetadataField::Genres,
            locked: true,
            source: MetadataSource::Nfo,
        },
    ]);
    let existing = CanonicalMetadata {
        title: "Local Title".to_owned(),
        overview: Some("old".to_owned()),
        genres: vec!["Local".to_owned()],
        ..CanonicalMetadata::default()
    };
    let incoming = CanonicalMetadata {
        title: "Provider Title".to_owned(),
        overview: Some("new".to_owned()),
        genres: vec!["Action".to_owned()],
        tagline: Some("Wake up.".to_owned()),
        ..CanonicalMetadata::default()
    };

    let merged = policy.merge(&existing, &incoming);

    assert_eq!(merged.title, "Local Title");
    assert_eq!(merged.overview, Some("new".to_owned()));
    assert_eq!(merged.genres, vec!["Local"]);
    assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
}

#[test]
fn missing_only_merge_fills_empty_fields_without_replacing_existing_values() {
    let policy = MetadataMergePolicy::from_locks_and_mode(&[], MetadataRefreshMode::MissingOnly);
    let existing = CanonicalMetadata {
        title: "Local Title".to_owned(),
        overview: Some("old".to_owned()),
        genres: Vec::new(),
        ..CanonicalMetadata::default()
    };
    let incoming = CanonicalMetadata {
        title: "Provider Title".to_owned(),
        overview: Some("new".to_owned()),
        genres: vec!["Action".to_owned()],
        tagline: Some("Wake up.".to_owned()),
        ..CanonicalMetadata::default()
    };

    let merged = policy.merge(&existing, &incoming);

    assert_eq!(merged.title, "Local Title");
    assert_eq!(merged.overview, Some("old".to_owned()));
    assert_eq!(merged.genres, vec!["Action"]);
    assert_eq!(merged.tagline, Some("Wake up.".to_owned()));
}

#[test]
fn full_refresh_replaces_unlocked_existing_values() {
    let policy = MetadataMergePolicy::from_locks_and_mode(&[], MetadataRefreshMode::FullRefresh);
    let existing = CanonicalMetadata {
        title: "Local Title".to_owned(),
        overview: Some("old".to_owned()),
        genres: vec!["Local".to_owned()],
        ..CanonicalMetadata::default()
    };
    let incoming = CanonicalMetadata {
        title: "Provider Title".to_owned(),
        overview: Some("new".to_owned()),
        genres: vec!["Action".to_owned()],
        ..CanonicalMetadata::default()
    };

    let merged = policy.merge(&existing, &incoming);

    assert_eq!(merged.title, "Provider Title");
    assert_eq!(merged.overview, Some("new".to_owned()));
    assert_eq!(merged.genres, vec!["Action"]);
}

#[tokio::test]
async fn refresh_searches_fetches_caches_raw_and_preserves_locks() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();

    let item = seed_movie(&store, "Local Matrix", Some("1999".to_owned()), vec![]).await;
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();

    let provider = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
        MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: "603".to_owned(),
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                overview: Some("A hacker discovers the nature of reality.".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
            raw_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
        },
    );
    let search_count = provider.search_count.clone();
    let fetch_count = provider.fetch_count.clone();
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    let summary = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap();
    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let raw = store
        .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "603")
        .await
        .unwrap()
        .unwrap();
    let persisted_attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
    let genres = store.list_genres(PageRequest::first_page()).await.unwrap();
    let hits = store
        .search(SearchQuery {
            query: "Science Fiction".to_owned(),
            facets: vec!["genre:Science Fiction".to_owned()],
            limit: 10,
            offset: 0,
        })
        .await
        .unwrap();

    assert_eq!(summary.provider_key, "603");
    assert_eq!(summary.provider, ExternalProvider::Tmdb);
    assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
    assert_eq!(summary.matched_by, MetadataMatchKind::Search);
    assert_eq!(summary.refresh_mode, MetadataRefreshMode::Default);
    assert_eq!(
        summary.attempted_providers,
        vec![MetadataProviderAttempt {
            provider: ExternalProvider::Tmdb,
            status: MetadataProviderAttemptStatus::Succeeded,
            message: None,
            provider_key: Some("603".to_owned()),
            matched_by: Some(MetadataMatchKind::Search),
            error_class: None,
        }]
    );
    assert!(summary.updated);
    assert_eq!(persisted_attempts.len(), 1);
    assert_eq!(
        persisted_attempts[0].status,
        MetadataProviderAttemptStatus::Succeeded
    );
    assert_eq!(
        persisted_attempts[0].matched_by,
        Some(MetadataMatchKind::Search)
    );
    assert_eq!(persisted_attempts[0].provider_key.as_deref(), Some("603"));
    assert_eq!(loaded.metadata.title, "Local Matrix");
    assert_eq!(
        loaded.metadata.overview,
        Some("A hacker discovers the nature of reality.".to_owned())
    );
    assert_eq!(
        loaded.metadata.genres,
        vec!["Action".to_owned(), "Science Fiction".to_owned()]
    );
    assert_eq!(raw.body_json, r#"{"id":603,"title":"The Matrix"}"#);
    assert!(genres.iter().any(|genre| genre.name == "Science Fiction"));
    assert_eq!(hits[0].item_id, item.id);
    assert_eq!(search_count.load(Ordering::SeqCst), 1);
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn refresh_uses_existing_external_id_without_search() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(
        &store,
        "The Matrix",
        Some("1999".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "603".to_owned(),
        }],
    )
    .await;
    let provider = mock_provider(
        ExternalProvider::Tmdb,
        Vec::new(),
        MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: "603".to_owned(),
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                runtime_minutes: Some(136),
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "603".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
            raw_json: r#"{"id":603,"runtime":136}"#.to_owned(),
        },
    );
    let search_count = provider.search_count.clone();
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    let summary = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap();

    assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
    assert_eq!(search_count.load(Ordering::SeqCst), 0);
    assert_eq!(
        store
            .get_media_item(item.id)
            .await
            .unwrap()
            .unwrap()
            .metadata
            .runtime_minutes,
        Some(136)
    );
}

#[tokio::test]
async fn strategy_falls_back_from_unimplemented_bangumi_to_tmdb() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "Anime Movie", Some("2024".to_owned()), vec![]).await;
    let job_id = seed_metadata_job(&store, &item).await;
    let tmdb = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "100", "Anime Movie")],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "100",
            CanonicalMetadata {
                title: "Anime Movie Provider Title".to_owned(),
                external_ids: vec![ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "100".to_owned(),
                }],
                ..CanonicalMetadata::default()
            },
        ),
    );
    let mut registry = MetadataProviderRegistry::new();
    registry.register(tmdb);
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    let summary = executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Anime),
            force: false,
        })
        .await
        .unwrap();

    assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
    assert_eq!(summary.provider_key, "100");
    assert_eq!(
        attempt_statuses(&summary),
        vec![
            (
                ExternalProvider::Bangumi,
                MetadataProviderAttemptStatus::NotImplemented
            ),
            (
                ExternalProvider::Tmdb,
                MetadataProviderAttemptStatus::Succeeded
            )
        ]
    );
    let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(
        attempts[0].status,
        MetadataProviderAttemptStatus::NotImplemented
    );
    assert_eq!(attempts[1].status, MetadataProviderAttemptStatus::Succeeded);
    assert_eq!(
        store
            .get_media_item(item.id)
            .await
            .unwrap()
            .unwrap()
            .metadata
            .title,
        "Anime Movie Provider Title"
    );
}

#[tokio::test]
async fn strategy_skips_disabled_provider() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "The Matrix", Some("1999".to_owned()), vec![]).await;
    let job_id = seed_metadata_job(&store, &item).await;
    let tmdb = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "603",
            CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        ),
    );
    let mut registry = MetadataProviderRegistry::new();
    registry.register_disabled(ExternalProvider::Douban, "disabled by config");
    registry.register(tmdb);
    let mut profile = MetadataProfile::from_preset(LibraryPreset::Movies);
    profile.metadata_providers = vec![ExternalProvider::Douban, ExternalProvider::Tmdb];
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    let summary = executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile,
            force: false,
        })
        .await
        .unwrap();

    assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
    assert_eq!(
        attempt_statuses(&summary),
        vec![
            (
                ExternalProvider::Douban,
                MetadataProviderAttemptStatus::SkippedDisabled
            ),
            (
                ExternalProvider::Tmdb,
                MetadataProviderAttemptStatus::Succeeded
            )
        ]
    );
    assert_eq!(
        store
            .list_metadata_provider_attempts(job_id)
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn strategy_fails_when_all_providers_fail() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "Unknown Movie", None, vec![]).await;
    let job_id = seed_metadata_job(&store, &item).await;
    let mut tmdb = mock_provider(
        ExternalProvider::Tmdb,
        Vec::new(),
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "never",
            CanonicalMetadata::default(),
        ),
    );
    tmdb.search_result = Ok(Vec::new());
    let mut registry = MetadataProviderRegistry::new();
    registry.register_unavailable(ExternalProvider::Bangumi, "credentials missing");
    registry.register(tmdb);
    let mut profile = MetadataProfile::from_preset(LibraryPreset::Anime);
    profile.metadata_providers = vec![ExternalProvider::Bangumi, ExternalProvider::Tmdb];
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    let err = executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile,
            force: false,
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("bangumi=skipped_unavailable"));
    assert!(message.contains("tmdb=no_match"));
    let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(
        attempts[0].status,
        MetadataProviderAttemptStatus::SkippedUnavailable
    );
    assert_eq!(attempts[1].status, MetadataProviderAttemptStatus::NoMatch);
}

#[tokio::test]
async fn strategy_persists_rate_limited_attempts() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(
        &store,
        "Rate Limited Movie",
        Some("2026".to_owned()),
        vec![],
    )
    .await;
    let job_id = seed_metadata_job(&store, &item).await;
    let mut tmdb = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(
            ExternalProvider::Tmdb,
            "rate-limited",
            "Rate Limited Movie",
        )],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "rate-limited",
            CanonicalMetadata::default(),
        ),
    );
    tmdb.fetch_result = Err(TaruError::Provider {
        provider: "tmdb".to_owned(),
        message: "fetch returned HTTP 429: rate limit exceeded".to_owned(),
    });
    let registry = MetadataProviderRegistry::new().with_provider(tmdb);
    let mut profile = MetadataProfile::from_preset(LibraryPreset::Movies);
    profile.metadata_providers = vec![ExternalProvider::Tmdb];
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    let err = executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile,
            force: false,
        })
        .await
        .unwrap_err();

    let TaruError::Provider { provider, message } = err else {
        panic!("expected provider exhaustion error");
    };
    assert_eq!(provider, "metadata_strategy");
    assert!(message.contains("tmdb=rate_limited"));
    let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        attempts[0].status,
        MetadataProviderAttemptStatus::RateLimited
    );
    assert_eq!(
        attempts[0].error_class,
        Some(MetadataProviderErrorClass::RateLimited)
    );
    assert!(attempts[0].status.is_retryable());
    assert!(attempts[0].error_class.unwrap().is_retryable());
}

#[tokio::test]
async fn strategy_short_circuits_after_first_success() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "The Matrix", Some("1999".to_owned()), vec![]).await;
    let job_id = seed_metadata_job(&store, &item).await;
    let tmdb = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "603",
            CanonicalMetadata {
                title: "The Matrix".to_owned(),
                ..CanonicalMetadata::default()
            },
        ),
    );
    let douban = mock_provider(
        ExternalProvider::Douban,
        vec![mock_candidate(
            ExternalProvider::Douban,
            "douban-1",
            "The Matrix",
        )],
        mock_fetch_result(
            ExternalProvider::Douban,
            "douban-1",
            CanonicalMetadata {
                title: "The Matrix Douban".to_owned(),
                ..CanonicalMetadata::default()
            },
        ),
    );
    let douban_search_count = douban.search_count.clone();
    let douban_fetch_count = douban.fetch_count.clone();
    let mut registry = MetadataProviderRegistry::new();
    registry.register(tmdb);
    registry.register(douban);
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    let summary = executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap();

    assert_eq!(summary.selected_provider, ExternalProvider::Tmdb);
    assert_eq!(summary.attempted_providers.len(), 1);
    assert_eq!(
        store
            .list_metadata_provider_attempts(job_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(douban_search_count.load(Ordering::SeqCst), 0);
    assert_eq!(douban_fetch_count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn strategy_preserves_locked_fields() {
    let store = SqliteStore::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "Local Matrix", Some("1999".to_owned()), vec![]).await;
    let job_id = seed_metadata_job(&store, &item).await;
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();
    let tmdb = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "603",
            CanonicalMetadata {
                title: "The Matrix".to_owned(),
                overview: Some("A hacker discovers the nature of reality.".to_owned()),
                ..CanonicalMetadata::default()
            },
        ),
    );
    let mut registry = MetadataProviderRegistry::new();
    registry.register(tmdb);
    let executor = MetadataStrategyExecutor::new(registry, store.clone());

    executor
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap();

    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    assert_eq!(loaded.metadata.title, "Local Matrix");
    assert_eq!(
        loaded.metadata.overview,
        Some("A hacker discovers the nature of reality.".to_owned())
    );
}

#[test]
fn tmdb_movie_details_maps_core_metadata() {
    let details: TmdbMovieDetails = serde_json::from_str(
        r#"
            {
              "id": 603,
              "title": "The Matrix",
              "original_title": "The Matrix",
              "overview": "A hacker discovers the nature of reality.",
              "release_date": "1999-03-31",
              "runtime": 136,
              "tagline": "Welcome to the Real World",
              "genres": [{"id": 28, "name": "Action"}],
              "belongs_to_collection": {"id": 2344, "name": "The Matrix Collection"},
              "production_companies": [{"id": 79, "name": "Village Roadshow Pictures"}],
              "poster_path": "/poster.jpg",
              "backdrop_path": "/backdrop.jpg",
              "external_ids": {"imdb_id": "tt0133093"},
              "credits": {
                "cast": [
                  {"id": 6384, "name": "Keanu Reeves", "character": "Neo", "order": 0}
                ],
                "crew": [
                  {"id": 9339, "name": "Lana Wachowski", "job": "Director"}
                ]
              },
              "images": {
                "posters": [
                  {"file_path": "/poster.jpg", "width": 1000, "height": 1500, "iso_639_1": "en"}
                ],
                "backdrops": [],
                "logos": []
              },
              "release_dates": {
                "results": [
                  {"iso_3166_1": "US", "release_dates": [{"certification": "R"}]}
                ]
              }
            }
            "#,
    )
    .unwrap();

    let metadata = tmdb_movie_details_to_metadata(details, DEFAULT_TMDB_IMAGE_BASE_URL);

    assert_eq!(metadata.title, "The Matrix");
    assert_eq!(metadata.runtime_minutes, Some(136));
    assert_eq!(metadata.genres, vec!["Action"]);
    assert_eq!(
        metadata.ratings,
        vec![ContentRating {
            source: "TMDB:US".to_owned(),
            value: "R".to_owned()
        }]
    );
    assert!(metadata.images.iter().any(|image| {
        image.kind == ImageKind::Poster
            && image.uri == "https://image.tmdb.org/t/p/original/poster.jpg"
    }));
    assert!(
        metadata.credits.iter().any(|credit| {
            credit.name == "Lana Wachowski" && credit.role == CreditRole::Director
        })
    );
    assert_eq!(metadata.collections[0].name, "The Matrix Collection");
    assert_eq!(metadata.studios[0].name, "Village Roadshow Pictures");
    assert!(metadata.external_ids.iter().any(|external_id| {
        external_id.provider == ExternalProvider::Imdb && external_id.value == "tt0133093"
    }));
}

#[test]
fn bangumi_subject_maps_core_metadata() {
    let subject: BangumiSubject = serde_json::from_str(
        r#"
            {
              "id": 8,
              "name": "Cowboy Bebop",
              "name_cn": "星际牛仔",
              "summary": "Whatever happens, happens.",
              "date": "1998-04-03",
              "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
              "infobox": [
                {"key": "动画制作", "value": "SUNRISE"}
              ],
              "tags": [{"name": "科幻"}, {"name": "原创"}],
              "rating": {"score": 9.1}
            }
            "#,
    )
    .unwrap();

    let metadata = bangumi_subject_to_metadata(subject, DEFAULT_BANGUMI_IMAGE_BASE_URL);

    assert_eq!(metadata.title, "星际牛仔");
    assert_eq!(metadata.original_title.as_deref(), Some("Cowboy Bebop"));
    assert_eq!(metadata.release_date.as_deref(), Some("1998-04-03"));
    assert_eq!(metadata.tags, vec!["科幻", "原创"]);
    assert_eq!(metadata.studios[0].name, "SUNRISE");
    assert!(metadata.images.iter().any(|image| {
        image.provider == ExternalProvider::Bangumi
            && image.uri == "https://lain.bgm.tv/pic/cover/l/8.jpg"
    }));
    assert!(metadata.external_ids.iter().any(|external_id| {
        external_id.provider == ExternalProvider::Bangumi && external_id.value == "8"
    }));
}

#[test]
fn douban_subject_maps_core_metadata() {
    let subject: DoubanSubject = serde_json::from_str(
        r#"
            {
              "id": "1292052",
              "title": "肖申克的救赎",
              "original_title": "The Shawshank Redemption",
              "summary": "Hope is a good thing.",
              "year": "1994",
              "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
              "genres": ["剧情", "犯罪"],
              "countries": ["美国"],
              "directors": [{"id": "1047973", "name": "Frank Darabont"}],
              "casts": [{"id": "1054521", "name": "Tim Robbins"}],
              "rating": {"average": 9.7}
            }
            "#,
    )
    .unwrap();

    let metadata = douban_subject_to_metadata(subject, None);

    assert_eq!(metadata.title, "肖申克的救赎");
    assert_eq!(
        metadata.original_title.as_deref(),
        Some("The Shawshank Redemption")
    );
    assert_eq!(metadata.release_date.as_deref(), Some("1994-01-01"));
    assert_eq!(metadata.genres, vec!["剧情", "犯罪"]);
    assert!(
        metadata.credits.iter().any(|credit| {
            credit.name == "Frank Darabont" && credit.role == CreditRole::Director
        })
    );
    assert!(metadata.external_ids.iter().any(|external_id| {
        external_id.provider == ExternalProvider::Douban && external_id.value == "1292052"
    }));
}

#[tokio::test]
async fn metadata_http_runtime_retries_and_sends_user_agent() {
    let server = MockMetadataServer::start().await;
    let runtime = MetadataHttpRuntime::new(MetadataHttpRuntimeConfig {
        max_attempts: 2,
        min_interval_ms: 0,
        user_agent: "taru-test-agent".to_owned(),
        ..MetadataHttpRuntimeConfig::default()
    })
    .unwrap();

    let body = runtime
        .get_json("mock", "flaky", server.url("/flaky"), &[], HeaderMap::new())
        .await
        .unwrap();

    assert_eq!(body["ok"], true);
    assert_eq!(server.request_count(), 2);
    assert_eq!(
        server.user_agents(),
        vec!["taru-test-agent", "taru-test-agent"]
    );
}

#[tokio::test]
async fn metadata_http_runtime_rate_limits_requests() {
    let server = MockMetadataServer::start().await;
    let runtime = MetadataHttpRuntime::new(MetadataHttpRuntimeConfig {
        min_interval_ms: 40,
        max_attempts: 1,
        ..MetadataHttpRuntimeConfig::default()
    })
    .unwrap();
    let started = Instant::now();

    runtime
        .get_json("mock", "ok", server.url("/ok"), &[], HeaderMap::new())
        .await
        .unwrap();
    runtime
        .get_json("mock", "ok", server.url("/ok"), &[], HeaderMap::new())
        .await
        .unwrap();

    assert!(started.elapsed().as_millis() >= 35);
}

#[tokio::test]
async fn tmdb_provider_uses_runtime_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = TmdbMetadataProvider::new(TmdbProviderConfig {
        read_access_token: fixtures::TMDB_TOKEN.to_owned(),
        api_base_url: server.url("/tmdb"),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "taru-tmdb-test".to_owned(),
            ..MetadataHttpRuntimeConfig::default()
        },
        ..TmdbProviderConfig::new(fixtures::TMDB_TOKEN.to_owned())
    })
    .unwrap();

    let candidates = provider
        .search(MetadataLookup {
            kind: Some(MediaKind::Movie),
            title: "The Matrix".to_owned(),
            year: Some(1999),
            language: Some("en-US".to_owned()),
            external_ids: Vec::new(),
        })
        .await
        .unwrap();
    let fetched = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Movie,
            provider_key: candidates[0].provider_key.clone(),
            language: Some("en-US".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidates[0].provider, ExternalProvider::Tmdb);
    assert_eq!(fetched.metadata.title, "The Matrix");
    assert_eq!(
        server.user_agents(),
        vec!["taru-tmdb-test", "taru-tmdb-test"]
    );
    assert_eq!(
        server.authorizations(),
        vec![
            format!("Bearer {}", fixtures::TMDB_TOKEN),
            format!("Bearer {}", fixtures::TMDB_TOKEN)
        ]
    );
}

#[tokio::test]
async fn bangumi_provider_uses_runtime_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = BangumiMetadataProvider::new(BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.to_owned()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "taru-bangumi-test".to_owned(),
            ..MetadataHttpRuntimeConfig::default()
        },
        ..BangumiProviderConfig::default()
    })
    .unwrap();

    let candidates = provider
        .search(MetadataLookup {
            kind: Some(MediaKind::Series),
            title: "Cowboy Bebop".to_owned(),
            year: Some(1998),
            language: Some("zh-CN".to_owned()),
            external_ids: Vec::new(),
        })
        .await
        .unwrap();
    let fetched = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Series,
            provider_key: candidates[0].provider_key.clone(),
            language: Some("zh-CN".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidates[0].provider, ExternalProvider::Bangumi);
    assert_eq!(fetched.metadata.title, "星际牛仔");
    assert!(
        server
            .authorizations()
            .iter()
            .any(|value| value == &format!("Bearer {}", fixtures::BANGUMI_TOKEN))
    );
}

#[tokio::test]
async fn douban_provider_uses_api_key_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = DoubanMetadataProvider::new(DoubanProviderConfig {
        api_key: Some(fixtures::DOUBAN_API_KEY.to_owned()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            ..MetadataHttpRuntimeConfig::default()
        },
        headers: vec![("X-Douban-Test".to_owned(), "ok".to_owned())],
        ..DoubanProviderConfig::default()
    })
    .unwrap();

    let candidates = provider
        .search(MetadataLookup {
            kind: Some(MediaKind::Movie),
            title: "肖申克的救赎".to_owned(),
            year: Some(1994),
            language: Some("zh-CN".to_owned()),
            external_ids: Vec::new(),
        })
        .await
        .unwrap();
    let fetched = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Movie,
            provider_key: candidates[0].provider_key.clone(),
            language: Some("zh-CN".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidates[0].provider, ExternalProvider::Douban);
    assert_eq!(fetched.metadata.title, "肖申克的救赎");
    assert!(
        server
            .uris()
            .iter()
            .any(|uri| uri.contains(&format!("apikey={}", fixtures::DOUBAN_API_KEY)))
    );
    assert!(
        server
            .headers("x-douban-test")
            .iter()
            .any(|value| value == "ok")
    );
}

async fn seed_movie(
    store: &SqliteStore,
    title: &str,
    release_date: Option<String>,
    external_ids: Vec<ExternalId>,
) -> MediaItem {
    let library = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            release_date,
            external_ids,
            ..CanonicalMetadata::default()
        },
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    item
}

async fn seed_metadata_job(store: &SqliteStore, item: &MediaItem) -> JobId {
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataRefresh,
            resource_class: "metadata.test".to_owned(),
            library_id: None,
            source_id: None,
            input_json: Some(
                serde_json::to_string(&MetadataRefreshJobInput {
                    item_id: item.id,
                    provider: None,
                    force: false,
                    language: None,
                    refresh_mode: MetadataRefreshMode::Default,
                })
                .unwrap(),
            ),
        })
        .await
        .unwrap()
        .id
}

struct MockMetadataProvider {
    provider: ExternalProvider,
    search_count: Arc<AtomicUsize>,
    fetch_count: Arc<AtomicUsize>,
    search_result: Result<Vec<MetadataCandidate>>,
    fetch_result: Result<MetadataFetchResult>,
}

#[async_trait]
impl MetadataProvider for MockMetadataProvider {
    fn provider(&self) -> ExternalProvider {
        self.provider.clone()
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    async fn search(&self, _lookup: MetadataLookup) -> Result<Vec<MetadataCandidate>> {
        self.search_count.fetch_add(1, Ordering::SeqCst);
        self.search_result.clone()
    }

    async fn fetch(&self, _request: MetadataFetchRequest) -> Result<MetadataFetchResult> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        self.fetch_result.clone()
    }
}

fn mock_provider(
    provider: ExternalProvider,
    search_candidates: Vec<MetadataCandidate>,
    fetch_result: MetadataFetchResult,
) -> MockMetadataProvider {
    MockMetadataProvider {
        provider,
        search_count: Arc::new(AtomicUsize::new(0)),
        fetch_count: Arc::new(AtomicUsize::new(0)),
        search_result: Ok(search_candidates),
        fetch_result: Ok(fetch_result),
    }
}

fn mock_candidate(
    provider: ExternalProvider,
    provider_key: &str,
    title: &str,
) -> MetadataCandidate {
    MetadataCandidate {
        provider,
        provider_key: provider_key.to_owned(),
        score: 0.95,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    }
}

fn mock_fetch_result(
    provider: ExternalProvider,
    provider_key: &str,
    metadata: CanonicalMetadata,
) -> MetadataFetchResult {
    MetadataFetchResult {
        provider,
        provider_key: provider_key.to_owned(),
        metadata,
        raw_json: format!(r#"{{"id":"{provider_key}"}}"#),
    }
}

fn attempt_statuses(
    summary: &MetadataRefreshSummary,
) -> Vec<(ExternalProvider, MetadataProviderAttemptStatus)> {
    summary
        .attempted_providers
        .iter()
        .map(|attempt| (attempt.provider.clone(), attempt.status))
        .collect()
}

#[derive(Clone)]
struct MockMetadataServer {
    base_url: String,
    state: MockMetadataState,
}

#[derive(Clone, Default)]
struct MockMetadataState {
    requests: Arc<StdMutex<Vec<MockRequest>>>,
}

#[derive(Clone, Debug)]
struct MockRequest {
    uri: String,
    user_agent: Option<String>,
    authorization: Option<String>,
    headers: Vec<(String, String)>,
}

impl MockMetadataServer {
    async fn start() -> Self {
        let state = MockMetadataState::default();
        let router = Router::new()
            .route("/ok", get(mock_ok))
            .route("/flaky", get(mock_flaky))
            .route("/tmdb/search/movie", get(mock_tmdb_search_movie))
            .route("/tmdb/movie/{id}", get(mock_tmdb_movie_details))
            .route("/v0/search/subjects", post(mock_bangumi_search))
            .route("/v0/subjects/{id}", get(mock_bangumi_subject))
            .route("/movie/search", get(mock_douban_search))
            .route("/movie/subject/{id}", get(mock_douban_subject))
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self {
            base_url: format!("http://{addr}"),
            state,
        }
    }

    fn base_url(&self) -> String {
        self.base_url.clone()
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn requests(&self) -> Vec<MockRequest> {
        self.state.requests.lock().unwrap().clone()
    }

    fn request_count(&self) -> usize {
        self.requests().len()
    }

    fn user_agents(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .filter_map(|request| request.user_agent)
            .collect()
    }

    fn authorizations(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .filter_map(|request| request.authorization)
            .collect()
    }

    fn uris(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .map(|request| request.uri)
            .collect()
    }

    fn headers(&self, name: &str) -> Vec<String> {
        self.requests()
            .into_iter()
            .flat_map(|request| request.headers.into_iter())
            .filter(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
            .collect()
    }
}

fn record_request(state: &MockMetadataState, headers: &AxumHeaderMap, uri: &Uri) -> usize {
    let request = MockRequest {
        uri: uri.to_string(),
        user_agent: headers
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        authorization: headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        headers: headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.as_str().to_owned(), value.to_owned()))
            })
            .collect(),
    };
    let mut requests = state.requests.lock().unwrap();
    requests.push(request);
    requests.len()
}

async fn mock_ok(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({"ok": true}))
}

async fn mock_flaky(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Response {
    let count = record_request(&state, &headers, &uri);
    if count == 1 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "retry"})),
        )
            .into_response();
    }

    Json(json!({"ok": true})).into_response()
}

async fn mock_bangumi_search(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "data": [{
            "id": 8,
            "name": "Cowboy Bebop",
            "name_cn": "星际牛仔",
            "summary": "Whatever happens, happens.",
            "date": "1998-04-03",
            "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
            "tags": [{"name": "科幻"}],
            "rating": {"score": 9.1}
        }]
    }))
}

async fn mock_tmdb_search_movie(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "results": [{
            "id": 603,
            "title": "The Matrix",
            "original_title": "The Matrix",
            "overview": "A hacker discovers the nature of reality.",
            "release_date": "1999-03-31",
            "poster_path": "/matrix.jpg",
            "popularity": 99.0
        }]
    }))
}

async fn mock_tmdb_movie_details(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": 603,
        "title": "The Matrix",
        "original_title": "The Matrix",
        "overview": "A hacker discovers the nature of reality.",
        "release_date": "1999-03-31",
        "runtime": 136,
        "tagline": "Welcome to the Real World.",
        "genres": [{"name": "Action"}, {"name": "Science Fiction"}],
        "poster_path": "/matrix.jpg",
        "credits": {"cast": [], "crew": []},
        "images": {"posters": [], "backdrops": []},
        "release_dates": {"results": []},
        "external_ids": {"imdb_id": "tt0133093"}
    }))
}

async fn mock_bangumi_subject(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": 8,
        "name": "Cowboy Bebop",
        "name_cn": "星际牛仔",
        "summary": "Whatever happens, happens.",
        "date": "1998-04-03",
        "images": {"large": "https://lain.bgm.tv/pic/cover/l/8.jpg"},
        "infobox": [{"key": "动画制作", "value": "SUNRISE"}],
        "tags": [{"name": "科幻"}],
        "rating": {"score": 9.1}
    }))
}

async fn mock_douban_search(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "subjects": [{
            "id": "1292052",
            "title": "肖申克的救赎",
            "original_title": "The Shawshank Redemption",
            "summary": "Hope is a good thing.",
            "year": "1994",
            "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
            "genres": ["剧情", "犯罪"],
            "rating": {"average": 9.7}
        }]
    }))
}

async fn mock_douban_subject(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": "1292052",
        "title": "肖申克的救赎",
        "original_title": "The Shawshank Redemption",
        "summary": "Hope is a good thing.",
        "year": "1994",
        "images": {"large": "https://img.doubanio.com/view/photo/l/public/p480747492.webp"},
        "genres": ["剧情", "犯罪"],
        "countries": ["美国"],
        "directors": [{"id": "1047973", "name": "Frank Darabont"}],
        "casts": [{"id": "1054521", "name": "Tim Robbins"}],
        "rating": {"average": 9.7}
    }))
}
