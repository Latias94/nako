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
use nako_core::{
    CanonicalMetadata, CatalogRepository, ContentRating, CreditRole, DatabaseLifecycle, ExternalId,
    ExternalProvider, ImageKind, JobId, JobKind, JobRepository, Library, LibraryId,
    LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryPreset, LibraryRepository,
    MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource, MediaSourceId,
    MetadataCandidateGraph, MetadataCandidateNode, MetadataCandidateRecord,
    MetadataCandidateRelationship, MetadataCandidateRelationshipKind, MetadataCandidateSource,
    MetadataCandidateSubject, MetadataField, MetadataFieldLock, MetadataMatchKind, MetadataProfile,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, MetadataRefreshMode,
    MetadataRepository, MetadataSource, NakoError, NewJob, PageRequest, ProviderMappingRepository,
    ProviderMappingStatus, ProviderSubjectKind, Result,
};
use nako_db::NakoDatabase;
use nako_search::{SearchIndex, SearchQuery};
use reqwest::header::HeaderMap;
use serde_json::json;
use tokio::{net::TcpListener, time::Instant};

use super::*;
use crate::mapping::{
    bangumi_subject_to_metadata, douban_subject_to_metadata, tmdb_movie_details_to_metadata,
};
use crate::providers::{
    BangumiSubject, DEFAULT_BANGUMI_IMAGE_BASE_URL, DEFAULT_TMDB_IMAGE_BASE_URL, DoubanSubject,
    TmdbMovieDetails,
};

mod fixtures;

#[test]
fn registry_diagnostics_include_provider_capabilities() {
    let registry = MetadataProviderRegistry::new().with_provider(mock_provider(
        ExternalProvider::Tmdb,
        vec![],
        mock_fetch_result(ExternalProvider::Tmdb, "603", CanonicalMetadata::default()),
    ));

    let diagnostic = registry.describe(&ExternalProvider::Tmdb).unwrap();
    let capabilities = diagnostic.capabilities.unwrap();

    assert_eq!(capabilities.provider, ExternalProvider::Tmdb);
    assert_eq!(capabilities.provider_name, "mock");
    assert!(capabilities.supports_search);
    assert!(capabilities.supports_fetch);
    assert!(capabilities.supports_external_id_match);
    assert!(!capabilities.supports_hierarchy);
    assert!(
        capabilities
            .supported_media_kinds
            .contains(&MediaKind::Movie)
    );
    assert!(
        capabilities
            .supported_subject_kinds
            .contains(&ProviderSubjectKind::Movie)
    );
    assert_eq!(
        capabilities.credential_requirement,
        MetadataProviderCredentialRequirement::Optional
    );
}

#[test]
fn built_in_provider_capabilities_are_diagnostics_safe() {
    let tmdb = TmdbMetadataProvider::new(TmdbProviderConfig::new(fixtures::TMDB_TOKEN)).unwrap();
    let bangumi = BangumiMetadataProvider::new(BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.into()),
        ..BangumiProviderConfig::default()
    })
    .unwrap();
    let douban = DoubanMetadataProvider::new(DoubanProviderConfig {
        api_key: Some(fixtures::DOUBAN_API_KEY.into()),
        headers: vec![("X-Douban-Secret".to_owned(), "header-secret".into())],
        ..DoubanProviderConfig::default()
    })
    .unwrap();

    let tmdb_capabilities = tmdb.capabilities();
    let bangumi_capabilities = bangumi.capabilities();
    let douban_capabilities = douban.capabilities();
    let debug = format!("{tmdb_capabilities:?}\n{bangumi_capabilities:?}\n{douban_capabilities:?}");

    assert_eq!(
        tmdb_capabilities.credential_requirement,
        MetadataProviderCredentialRequirement::Required
    );
    assert!(
        tmdb_capabilities
            .supported_media_kinds
            .contains(&MediaKind::Series)
    );
    assert!(tmdb_capabilities.supports_hierarchy);
    assert_eq!(
        bangumi_capabilities.credential_requirement,
        MetadataProviderCredentialRequirement::Optional
    );
    assert!(
        bangumi_capabilities
            .supported_media_kinds
            .contains(&MediaKind::Series)
    );
    assert!(
        !bangumi_capabilities
            .supported_media_kinds
            .contains(&MediaKind::Episode)
    );
    assert!(
        !bangumi_capabilities
            .supported_subject_kinds
            .contains(&ProviderSubjectKind::Episode)
    );
    assert!(!bangumi_capabilities.supports_hierarchy);
    assert_eq!(
        douban_capabilities.credential_requirement,
        MetadataProviderCredentialRequirement::Optional
    );
    assert!(
        douban_capabilities
            .supported_media_kinds
            .contains(&MediaKind::Movie)
    );
    assert!(!debug.contains(fixtures::TMDB_TOKEN));
    assert!(!debug.contains(fixtures::BANGUMI_TOKEN));
    assert!(!debug.contains(fixtures::DOUBAN_API_KEY));
    assert!(!debug.contains("header-secret"));
}

#[test]
fn matching_policy_accepts_rejects_and_requires_confirmation_with_reasons() {
    let policy = MetadataCandidateMatchingPolicy::strict();
    let lookup = MetadataLookup {
        kind: Some(MediaKind::Movie),
        title: "The Matrix".to_owned(),
        year: Some(1999),
        language: None,
        external_ids: Vec::new(),
    };
    let matches = policy.evaluate_for_lookup(
        &lookup,
        vec![
            scored_candidate_with_release_date(
                ExternalProvider::Tmdb,
                "603",
                "The Matrix",
                Some("1999-03-31"),
                0.95,
            ),
            scored_candidate_with_release_date(
                ExternalProvider::Douban,
                "1292052",
                "The Matrix",
                Some("1999"),
                0.72,
            ),
            scored_candidate_with_release_date(
                ExternalProvider::Bangumi,
                "8",
                "Cowboy Bebop",
                Some("1998-04-03"),
                0.30,
            ),
        ],
    );

    assert_eq!(
        matches[0].decision,
        MetadataCandidateMatchDecision::Accepted
    );
    assert!(
        matches[0]
            .reasons
            .contains(&MetadataCandidateMatchReason::ExactTitle)
    );
    assert!(
        matches[0]
            .reasons
            .contains(&MetadataCandidateMatchReason::ReleaseYearMatch)
    );
    assert!(matches[0].message.contains("accepted"));
    assert_eq!(
        matches[1].decision,
        MetadataCandidateMatchDecision::NeedsConfirmation
    );
    assert!(matches[1].needs_confirmation());
    assert!(matches[1].message.contains("needs confirmation"));
    assert_eq!(
        matches[2].decision,
        MetadataCandidateMatchDecision::Rejected
    );
    assert!(
        matches[2]
            .reasons
            .contains(&MetadataCandidateMatchReason::DifferentTitle)
    );
    assert!(
        matches[2]
            .reasons
            .contains(&MetadataCandidateMatchReason::ReleaseYearMismatch)
    );
    assert!(matches[2].message.contains("below confirmation threshold"));
}

#[test]
fn matching_policy_requires_confirmation_for_conflicting_high_confidence_candidates() {
    let policy = MetadataCandidateMatchingPolicy::strict();
    let matches = policy.evaluate(vec![
        scored_candidate(ExternalProvider::Tmdb, "603", "The Matrix", 0.96),
        scored_candidate(
            ExternalProvider::Douban,
            "conflict",
            "The Matrix Reloaded",
            0.94,
        ),
    ]);

    assert_eq!(
        matches[0].decision,
        MetadataCandidateMatchDecision::NeedsConfirmation
    );
    assert_eq!(
        matches[1].decision,
        MetadataCandidateMatchDecision::NeedsConfirmation
    );
    assert!(
        matches[0]
            .reasons
            .contains(&MetadataCandidateMatchReason::NearbyHighConfidenceConflict)
    );
    assert!(
        matches[1]
            .reasons
            .contains(&MetadataCandidateMatchReason::NearbyHighConfidenceConflict)
    );
    assert!(matches[0].message.contains("conflict"));
    assert!(matches[1].message.contains("conflict"));
}

#[test]
fn candidate_conflict_review_collects_cross_provider_decisions_without_canonical_commit() {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            release_date: Some("1999-03-31".to_owned()),
            ..CanonicalMetadata::default()
        },
    };

    let review = build_candidate_conflict_review(
        &item,
        Some("en-US".to_owned()),
        vec![
            scored_candidate_with_release_date(
                ExternalProvider::Tmdb,
                "603",
                "The Matrix",
                Some("1999-03-31"),
                0.96,
            ),
            scored_candidate_with_release_date(
                ExternalProvider::Douban,
                "conflict",
                "The Matrix Reloaded",
                Some("2003"),
                0.94,
            ),
        ],
    );

    assert_eq!(review.item_id, item.id);
    assert_eq!(
        review.status,
        MetadataCandidateConflictReviewStatus::NeedsConfirmation
    );
    assert!(review.requires_confirmation());
    assert_eq!(review.lookup.title, "The Matrix");
    assert_eq!(review.lookup.year, Some(1999));
    assert_eq!(review.decisions.len(), 2);
    assert!(
        review
            .decisions
            .iter()
            .all(|decision| decision.decision == MetadataCandidateMatchDecision::NeedsConfirmation)
    );
    assert!(
        review
            .decisions
            .iter()
            .any(|decision| decision.provider == ExternalProvider::Tmdb)
    );
    assert!(
        review
            .decisions
            .iter()
            .any(|decision| decision.provider == ExternalProvider::Douban)
    );
    assert!(review.message.contains("manual confirmation"));
}

#[test]
fn candidate_conflict_review_marks_all_rejected_candidates_as_no_acceptable_candidates() {
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            release_date: Some("1999".to_owned()),
            ..CanonicalMetadata::default()
        },
    };

    let review = build_candidate_conflict_review(
        &item,
        None,
        vec![scored_candidate_with_release_date(
            ExternalProvider::Bangumi,
            "weak",
            "Cowboy Bebop",
            Some("1998"),
            0.30,
        )],
    );

    assert_eq!(
        review.status,
        MetadataCandidateConflictReviewStatus::NoAcceptableCandidates
    );
    assert_eq!(
        review.decisions[0].decision,
        MetadataCandidateMatchDecision::Rejected
    );
    assert!(review.message.contains("all were rejected"));
}

#[tokio::test]
async fn hierarchy_confirmation_confirms_provisional_items_in_place() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = Library {
        id: LibraryId::new(),
        name: "TV".to_owned(),
        roots: vec!["local:///TV".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Tv),
    };
    let series = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Fireflie".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let season = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Season,
        parent_id: Some(series.id),
        metadata: CanonicalMetadata {
            title: "Season 1".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let episode = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: Some(season.id),
        metadata: CanonicalMetadata {
            title: "Episode 2".to_owned(),
            ..CanonicalMetadata::default()
        },
    };

    store.upsert_library(&library).await.unwrap();
    for item in [&series, &season, &episode] {
        store.upsert_media_item(item).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id: library.id,
                item_id: item.id,
                provisional: true,
            })
            .await
            .unwrap();
    }
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: episode.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();

    let service = HierarchyConfirmationService::new(store.clone());
    let summary = service
        .confirm_hierarchy(HierarchyConfirmationRequest {
            library_id: library.id,
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
            refresh_mode: MetadataRefreshMode::FullRefresh,
            items: vec![
                HierarchyConfirmationItem {
                    item_id: series.id,
                    kind: MediaKind::Series,
                    parent_id: None,
                    metadata: CanonicalMetadata {
                        title: "Firefly".to_owned(),
                        release_date: Some("2002-09-20".to_owned()),
                        ..CanonicalMetadata::default()
                    },
                    provider_subject: Some(HierarchyProviderSubject {
                        provider: ExternalProvider::Tmdb,
                        subject_kind: ProviderSubjectKind::Series,
                        subject_key: "1437".to_owned(),
                        title: Some("Firefly".to_owned()),
                        release_year: Some(2002),
                        locale: Some("en-US".to_owned()),
                    }),
                    confidence_milli: Some(980),
                },
                HierarchyConfirmationItem {
                    item_id: season.id,
                    kind: MediaKind::Season,
                    parent_id: Some(series.id),
                    metadata: CanonicalMetadata {
                        title: "Season 1".to_owned(),
                        release_date: Some("2002".to_owned()),
                        ..CanonicalMetadata::default()
                    },
                    provider_subject: Some(HierarchyProviderSubject {
                        provider: ExternalProvider::Tmdb,
                        subject_kind: ProviderSubjectKind::Season,
                        subject_key: "1437/1".to_owned(),
                        title: Some("Season 1".to_owned()),
                        release_year: Some(2002),
                        locale: Some("en-US".to_owned()),
                    }),
                    confidence_milli: Some(980),
                },
                HierarchyConfirmationItem {
                    item_id: episode.id,
                    kind: MediaKind::Episode,
                    parent_id: Some(season.id),
                    metadata: CanonicalMetadata {
                        title: "The Train Job".to_owned(),
                        overview: Some("The crew takes a train heist job.".to_owned()),
                        release_date: Some("2002-09-20".to_owned()),
                        ..CanonicalMetadata::default()
                    },
                    provider_subject: Some(HierarchyProviderSubject {
                        provider: ExternalProvider::Tmdb,
                        subject_kind: ProviderSubjectKind::Episode,
                        subject_key: "1437/1/2".to_owned(),
                        title: Some("The Train Job".to_owned()),
                        release_year: Some(2002),
                        locale: Some("en-US".to_owned()),
                    }),
                    confidence_milli: Some(980),
                },
            ],
        })
        .await
        .unwrap();

    let confirmed_series = store.get_media_item(series.id).await.unwrap().unwrap();
    let confirmed_episode = store.get_media_item(episode.id).await.unwrap().unwrap();
    let episode_mappings = store
        .list_provider_mappings_for_item(episode.id, PageRequest::first_page())
        .await
        .unwrap();
    let episode_subjects = store
        .list_provider_subjects_for_item(episode.id, PageRequest::first_page())
        .await
        .unwrap();
    let hits = store
        .search(SearchQuery::from_facet_labels("heist", Vec::new(), 10, 0).unwrap())
        .await
        .unwrap();

    assert_eq!(summary.confirmed_items, 3);
    assert_eq!(summary.updated_items, 3);
    assert_eq!(summary.provider_mappings, 3);
    assert_eq!(confirmed_series.id, series.id);
    assert_eq!(confirmed_series.metadata.title, "Firefly");
    assert_eq!(confirmed_episode.id, episode.id);
    assert_eq!(confirmed_episode.parent_id, Some(season.id));
    assert_eq!(confirmed_episode.metadata.title, "Episode 2");
    assert_eq!(
        confirmed_episode.metadata.overview,
        Some("The crew takes a train heist job.".to_owned())
    );
    for item_id in [series.id, season.id, episode.id] {
        assert!(
            !store
                .get_library_item_state(library.id, item_id)
                .await
                .unwrap()
                .unwrap()
                .provisional
        );
    }
    assert_eq!(episode_mappings.len(), 1);
    assert_eq!(episode_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(episode_mappings[0].confidence_milli, Some(980));
    assert_eq!(
        episode_subjects[0].subject_kind,
        ProviderSubjectKind::Episode
    );
    assert_eq!(episode_subjects[0].subject_key, "1437/1/2");
    assert_eq!(hits[0].item_id, episode.id);
}

#[tokio::test]
async fn hierarchy_confirmation_rejects_confirmed_structure_changes() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let library = Library {
        id: LibraryId::new(),
        name: "TV".to_owned(),
        roots: vec!["local:///TV".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Tv),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Confirmed".to_owned(),
            ..CanonicalMetadata::default()
        },
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    let service = HierarchyConfirmationService::new(store);
    let err = service
        .confirm_hierarchy(HierarchyConfirmationRequest {
            library_id: library.id,
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
            refresh_mode: MetadataRefreshMode::FullRefresh,
            items: vec![HierarchyConfirmationItem {
                item_id: item.id,
                kind: MediaKind::Movie,
                parent_id: None,
                metadata: item.metadata.clone(),
                provider_subject: None,
                confidence_milli: None,
            }],
        })
        .await
        .unwrap_err();

    assert!(err.to_string().contains("use hierarchy repair"));
}

#[tokio::test]
async fn hierarchy_confirmation_allows_source_authority_to_refresh_own_locked_fields() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
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
            title: "Provider Old Title".to_owned(),
            overview: Some("User overview".to_owned()),
            ..CanonicalMetadata::default()
        },
    };

    store.upsert_library(&library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: true,
        })
        .await
        .unwrap();
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
        })
        .await
        .unwrap();
    store
        .upsert_field_lock(&MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Overview,
            locked: true,
            source: MetadataSource::User,
        })
        .await
        .unwrap();

    HierarchyConfirmationService::new(store.clone())
        .confirm_hierarchy(HierarchyConfirmationRequest {
            library_id: library.id,
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
            refresh_mode: MetadataRefreshMode::FullRefresh,
            items: vec![HierarchyConfirmationItem {
                item_id: item.id,
                kind: MediaKind::Movie,
                parent_id: None,
                metadata: CanonicalMetadata {
                    title: "Provider New Title".to_owned(),
                    overview: Some("Provider overview".to_owned()),
                    ..CanonicalMetadata::default()
                },
                provider_subject: None,
                confidence_milli: None,
            }],
        })
        .await
        .unwrap();

    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();

    assert_eq!(loaded.metadata.title, "Provider New Title");
    assert_eq!(loaded.metadata.overview, Some("User overview".to_owned()));
}

#[tokio::test]
async fn metadata_refresh_confirms_provider_state_across_all_library_memberships() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "The Matrix".to_owned(),
            release_date: Some("1999".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "603".to_owned(),
            }],
            ..CanonicalMetadata::default()
        },
    };
    let first_library = Library {
        id: LibraryId::new(),
        name: "Movies A".to_owned(),
        roots: vec!["local:///A".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let second_library = Library {
        id: LibraryId::new(),
        name: "Movies B".to_owned(),
        roots: vec!["local:///B".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let source_a = MediaSource {
        id: MediaSourceId::new(),
        library_id: first_library.id,
        item_id: item.id,
        locator: "local:///A/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1),
        fingerprint: None,
    };
    let source_b = MediaSource {
        id: MediaSourceId::new(),
        library_id: second_library.id,
        item_id: item.id,
        locator: "local:///B/The Matrix.mkv".to_owned(),
        file_name: "The Matrix.mkv".to_owned(),
        size_bytes: Some(1),
        fingerprint: None,
    };
    let provider = mock_provider(
        ExternalProvider::Tmdb,
        vec![mock_candidate(ExternalProvider::Tmdb, "603", "The Matrix")],
        mock_fetch_result(
            ExternalProvider::Tmdb,
            "603",
            CanonicalMetadata {
                title: "The Matrix".to_owned(),
                release_date: Some("1999-03-31".to_owned()),
                ..CanonicalMetadata::default()
            },
        ),
    );
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    store.upsert_library(&first_library).await.unwrap();
    store.upsert_library(&second_library).await.unwrap();
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source_a).await.unwrap();
    store.upsert_media_source(&source_b).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: first_library.id,
            item_id: item.id,
            provisional: true,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: second_library.id,
            item_id: item.id,
            provisional: true,
        })
        .await
        .unwrap();

    service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap();

    assert!(
        !store
            .get_library_item_state(first_library.id, item.id)
            .await
            .unwrap()
            .unwrap()
            .provisional
    );
    assert!(
        !store
            .get_library_item_state(second_library.id, item.id)
            .await
            .unwrap()
            .unwrap()
            .provisional
    );
}

#[tokio::test]
async fn metadata_refresh_accepts_douban_and_bangumi_provider_mappings() {
    let server = MockMetadataServer::start().await;
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let douban_item = seed_media_item(
        &store,
        LibraryPreset::Movies,
        MediaKind::Movie,
        "肖申克的救赎",
        Some("1994".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Douban,
            value: "1292052".to_owned(),
        }],
    )
    .await;
    let bangumi_item = seed_media_item(
        &store,
        LibraryPreset::Anime,
        MediaKind::Series,
        "Cowboy Bebop",
        Some("1998".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: "8".to_owned(),
        }],
    )
    .await;
    let douban = DoubanMetadataProvider::new(DoubanProviderConfig {
        api_key: Some(fixtures::DOUBAN_API_KEY.into()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            ..MetadataHttpRuntimeConfig::default()
        },
        ..DoubanProviderConfig::default()
    })
    .unwrap();
    let bangumi = BangumiMetadataProvider::new(BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.into()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            ..MetadataHttpRuntimeConfig::default()
        },
        ..BangumiProviderConfig::default()
    })
    .unwrap();
    let mut douban_profile = MetadataProfile::from_preset(LibraryPreset::Movies);
    douban_profile.metadata_providers = vec![ExternalProvider::Douban];
    let mut bangumi_profile = MetadataProfile::from_preset(LibraryPreset::Anime);
    bangumi_profile.metadata_providers = vec![ExternalProvider::Bangumi];

    MetadataRefreshService::new(douban, store.clone())
        .refresh_item(MetadataRefreshRequest {
            job_id: seed_metadata_job(&store, &douban_item).await,
            item_id: douban_item.id,
            profile: douban_profile,
            force: false,
        })
        .await
        .unwrap();
    MetadataRefreshService::new(bangumi, store.clone())
        .refresh_item(MetadataRefreshRequest {
            job_id: seed_metadata_job(&store, &bangumi_item).await,
            item_id: bangumi_item.id,
            profile: bangumi_profile,
            force: false,
        })
        .await
        .unwrap();

    let douban_subjects = store
        .list_provider_subjects_for_item(douban_item.id, PageRequest::first_page())
        .await
        .unwrap();
    let bangumi_subjects = store
        .list_provider_subjects_for_item(bangumi_item.id, PageRequest::first_page())
        .await
        .unwrap();
    let douban_mappings = store
        .list_provider_mappings_for_item(douban_item.id, PageRequest::first_page())
        .await
        .unwrap();
    let bangumi_mappings = store
        .list_provider_mappings_for_item(bangumi_item.id, PageRequest::first_page())
        .await
        .unwrap();

    assert_eq!(douban_subjects[0].provider, ExternalProvider::Douban);
    assert_eq!(douban_subjects[0].subject_kind, ProviderSubjectKind::Movie);
    assert_eq!(douban_subjects[0].subject_key, "1292052");
    assert_eq!(douban_subjects[0].title.as_deref(), Some("肖申克的救赎"));
    assert_eq!(douban_subjects[0].release_year, Some(1994));
    assert_eq!(douban_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(bangumi_subjects[0].provider, ExternalProvider::Bangumi);
    assert_eq!(
        bangumi_subjects[0].subject_kind,
        ProviderSubjectKind::Series
    );
    assert_eq!(bangumi_subjects[0].subject_key, "8");
    assert_eq!(bangumi_subjects[0].title.as_deref(), Some("星际牛仔"));
    assert_eq!(bangumi_subjects[0].release_year, Some(1998));
    assert_eq!(bangumi_mappings[0].status, ProviderMappingStatus::Accepted);
}

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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
            graph: MetadataCandidateGraph::from_canonical(
                MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                MediaKind::Movie,
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    overview: Some("A hacker discovers the nature of reality.".to_owned()),
                    genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
            ),
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
    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let provider_subjects = store
        .list_provider_subjects_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let persisted_attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();
    let genres = store.list_genres(PageRequest::first_page()).await.unwrap();
    let hits = store
        .search(
            SearchQuery::from_facet_labels(
                "Science Fiction",
                vec!["genre:Science Fiction".to_owned()],
                10,
                0,
            )
            .unwrap(),
        )
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
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(
        provider_mappings[0].source,
        MetadataSource::Provider(ExternalProvider::Tmdb)
    );
    assert_eq!(
        provider_subjects[0].subject_kind,
        ProviderSubjectKind::Movie
    );
    assert_eq!(provider_subjects[0].subject_key, "603");
    assert!(genres.iter().any(|genre| genre.name == "Science Fiction"));
    assert_eq!(hits[0].item_id, item.id);
    assert_eq!(search_count.load(Ordering::SeqCst), 1);
    assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn refresh_search_requires_confirmation_for_ambiguous_candidate_without_commit() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_movie(&store, "Ambiguous Matrix", Some("1999".to_owned()), vec![]).await;
    let provider = mock_provider(
        ExternalProvider::Tmdb,
        vec![scored_candidate(
            ExternalProvider::Tmdb,
            "weak-603",
            "The Matrix",
            0.72,
        )],
        MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: "weak-603".to_owned(),
            graph: MetadataCandidateGraph::from_canonical(
                MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                MediaKind::Movie,
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    overview: Some("Should not be committed.".to_owned()),
                    ..CanonicalMetadata::default()
                },
            ),
            raw_json: r#"{"id":"weak-603"}"#.to_owned(),
        },
    );
    let search_count = provider.search_count.clone();
    let fetch_count = provider.fetch_count.clone();
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    let err = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Movies),
            force: false,
        })
        .await
        .unwrap_err();

    let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
    let raw = store
        .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "weak-603")
        .await
        .unwrap();
    let mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let attempts = store.list_metadata_provider_attempts(job_id).await.unwrap();

    assert!(err.to_string().contains("needs confirmation"));
    assert_eq!(loaded.metadata.title, "Ambiguous Matrix");
    assert_eq!(loaded.metadata.overview, None);
    assert!(raw.is_none());
    assert!(mappings.is_empty());
    assert_eq!(search_count.load(Ordering::SeqCst), 1);
    assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].status, MetadataProviderAttemptStatus::NoMatch);
    assert_eq!(
        attempts[0].error_class,
        Some(MetadataProviderErrorClass::NoMatch)
    );
    assert!(
        attempts[0]
            .message
            .as_deref()
            .unwrap()
            .contains("needs confirmation")
    );
}

#[tokio::test]
async fn refresh_uses_existing_external_id_without_search() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
            graph: MetadataCandidateGraph::from_canonical(
                MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
                MediaKind::Movie,
                CanonicalMetadata {
                    title: "The Matrix".to_owned(),
                    runtime_minutes: Some(136),
                    external_ids: vec![ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    }],
                    ..CanonicalMetadata::default()
                },
            ),
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
async fn refresh_persists_only_root_provider_mapping_from_provider_graph_preview() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_media_item(
        &store,
        LibraryPreset::Tv,
        MediaKind::Series,
        "Firefly",
        Some("2002-09-20".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "1437".to_owned(),
        }],
    )
    .await;
    let mut graph = MetadataCandidateGraph::for_provider(
        ExternalProvider::Tmdb,
        MediaKind::Series,
        ProviderSubjectKind::Series,
        "1437",
        MetadataCandidateRecord::from(CanonicalMetadata {
            title: "Firefly".to_owned(),
            release_date: Some("2002-09-20".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "1437".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    );
    let series_subject = graph.root_provider_subject().unwrap().clone();
    let season_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Season,
        subject_key: "1437/1".to_owned(),
        title: Some("Season 1".to_owned()),
        release_year: Some(2002),
        locale: None,
    };
    graph.related.push(MetadataCandidateNode {
        source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
        kind: MediaKind::Season,
        subject: Some(season_subject.clone()),
        metadata: MetadataCandidateRecord::from(CanonicalMetadata {
            title: "Season 1".to_owned(),
            release_date: Some("2002-09-20".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "3624".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    });
    graph.relationships.push(MetadataCandidateRelationship {
        parent_subject: series_subject,
        child_subject: season_subject,
        kind: MetadataCandidateRelationshipKind::Contains,
    });
    let provider = mock_provider(
        ExternalProvider::Tmdb,
        Vec::new(),
        MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: "1437".to_owned(),
            graph,
            raw_json: r#"{"id":1437,"name":"Firefly","seasons":[{"id":3624}]}"#.to_owned(),
        },
    );
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    let summary = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Tv),
            force: false,
        })
        .await
        .unwrap();

    let media_items = store
        .list_media_items(PageRequest::first_page())
        .await
        .unwrap();
    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let provider_subjects = store
        .list_provider_subjects_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let raw = store
        .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "1437")
        .await
        .unwrap()
        .unwrap();
    let season_subject = store
        .find_provider_subject(
            &ExternalProvider::Tmdb,
            &ProviderSubjectKind::Season,
            "1437/1",
        )
        .await
        .unwrap();

    assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
    assert_eq!(summary.provider_key, "1437");
    assert!(raw.body_json.contains(r#""seasons""#));
    assert_eq!(media_items.len(), 1);
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(provider_subjects.len(), 1);
    assert_eq!(
        provider_subjects[0].subject_kind,
        ProviderSubjectKind::Series
    );
    assert_eq!(provider_subjects[0].subject_key, "1437");
    assert!(season_subject.is_none());
}

#[tokio::test]
async fn refresh_persists_only_root_provider_mapping_from_season_episode_graph_preview() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_media_item(
        &store,
        LibraryPreset::Tv,
        MediaKind::Season,
        "Season 1",
        Some("2002-09-20".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Tmdb,
            value: "1437/1".to_owned(),
        }],
    )
    .await;
    let mut graph = MetadataCandidateGraph::for_provider(
        ExternalProvider::Tmdb,
        MediaKind::Season,
        ProviderSubjectKind::Season,
        "1437/1",
        MetadataCandidateRecord::from(CanonicalMetadata {
            title: "Season 1".to_owned(),
            release_date: Some("2002-09-20".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "3624".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    );
    let season_subject = graph.root_provider_subject().unwrap().clone();
    let episode_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "1437/1/2".to_owned(),
        title: Some("The Train Job".to_owned()),
        release_year: Some(2002),
        locale: None,
    };
    graph.related.push(MetadataCandidateNode {
        source: MetadataCandidateSource::Provider(ExternalProvider::Tmdb),
        kind: MediaKind::Episode,
        subject: Some(episode_subject.clone()),
        metadata: MetadataCandidateRecord::from(CanonicalMetadata {
            title: "The Train Job".to_owned(),
            release_date: Some("2002-09-20".to_owned()),
            runtime_minutes: Some(45),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "12345".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    });
    graph.relationships.push(MetadataCandidateRelationship {
        parent_subject: season_subject,
        child_subject: episode_subject,
        kind: MetadataCandidateRelationshipKind::Contains,
    });
    let provider = mock_provider(
        ExternalProvider::Tmdb,
        Vec::new(),
        MetadataFetchResult {
            provider: ExternalProvider::Tmdb,
            provider_key: "1437/1".to_owned(),
            graph,
            raw_json: r#"{"id":3624,"name":"Season 1","episodes":[{"id":12345}]}"#.to_owned(),
        },
    );
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;

    let summary = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile: MetadataProfile::from_preset(LibraryPreset::Tv),
            force: false,
        })
        .await
        .unwrap();

    let media_items = store
        .list_media_items(PageRequest::first_page())
        .await
        .unwrap();
    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let provider_subjects = store
        .list_provider_subjects_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let raw = store
        .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "1437/1")
        .await
        .unwrap()
        .unwrap();
    let episode_subject = store
        .find_provider_subject(
            &ExternalProvider::Tmdb,
            &ProviderSubjectKind::Episode,
            "1437/1/2",
        )
        .await
        .unwrap();

    assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
    assert_eq!(summary.provider_key, "1437/1");
    assert!(raw.body_json.contains(r#""episodes""#));
    assert_eq!(media_items.len(), 1);
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(provider_subjects.len(), 1);
    assert_eq!(
        provider_subjects[0].subject_kind,
        ProviderSubjectKind::Season
    );
    assert_eq!(provider_subjects[0].subject_key, "1437/1");
    assert!(episode_subject.is_none());
}

#[tokio::test]
async fn refresh_persists_only_root_provider_mapping_from_bangumi_episode_graph_preview() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    store.migrate().await.unwrap();
    let item = seed_media_item(
        &store,
        LibraryPreset::Anime,
        MediaKind::Series,
        "星际牛仔",
        Some("1998-04-03".to_owned()),
        vec![ExternalId {
            provider: ExternalProvider::Bangumi,
            value: "8".to_owned(),
        }],
    )
    .await;
    let mut graph = MetadataCandidateGraph::for_provider(
        ExternalProvider::Bangumi,
        MediaKind::Series,
        ProviderSubjectKind::Series,
        "8",
        MetadataCandidateRecord::from(CanonicalMetadata {
            title: "星际牛仔".to_owned(),
            original_title: Some("Cowboy Bebop".to_owned()),
            release_date: Some("1998-04-03".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Bangumi,
                value: "8".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    );
    let series_subject = graph.root_provider_subject().unwrap().clone();
    let episode_subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Episode,
        subject_key: "101".to_owned(),
        title: Some("阿斯特罗蓝调".to_owned()),
        release_year: Some(1998),
        locale: None,
    };
    graph.related.push(MetadataCandidateNode {
        source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
        kind: MediaKind::Episode,
        subject: Some(episode_subject.clone()),
        metadata: MetadataCandidateRecord::from(CanonicalMetadata {
            title: "阿斯特罗蓝调".to_owned(),
            original_title: Some("Asteroid Blues".to_owned()),
            release_date: Some("1998-04-03".to_owned()),
            runtime_minutes: Some(24),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Bangumi,
                value: "101".to_owned(),
            }],
            ..CanonicalMetadata::default()
        }),
    });
    graph.relationships.push(MetadataCandidateRelationship {
        parent_subject: series_subject,
        child_subject: episode_subject,
        kind: MetadataCandidateRelationshipKind::Contains,
    });
    let provider = mock_provider(
        ExternalProvider::Bangumi,
        Vec::new(),
        MetadataFetchResult {
            provider: ExternalProvider::Bangumi,
            provider_key: "8".to_owned(),
            graph,
            raw_json: r#"{"subject":{"id":8},"episodes":{"data":[{"id":101}]}}"#.to_owned(),
        },
    );
    let service = MetadataRefreshService::new(provider, store.clone());
    let job_id = seed_metadata_job(&store, &item).await;
    let mut profile = MetadataProfile::from_preset(LibraryPreset::Anime);
    profile.metadata_providers = vec![ExternalProvider::Bangumi];

    let summary = service
        .refresh_item(MetadataRefreshRequest {
            job_id,
            item_id: item.id,
            profile,
            force: false,
        })
        .await
        .unwrap();

    let media_items = store
        .list_media_items(PageRequest::first_page())
        .await
        .unwrap();
    let provider_mappings = store
        .list_provider_mappings_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let provider_subjects = store
        .list_provider_subjects_for_item(item.id, PageRequest::first_page())
        .await
        .unwrap();
    let raw = store
        .get_provider_raw_response(item.id, &ExternalProvider::Bangumi, "8")
        .await
        .unwrap()
        .unwrap();
    let episode_subject = store
        .find_provider_subject(
            &ExternalProvider::Bangumi,
            &ProviderSubjectKind::Episode,
            "101",
        )
        .await
        .unwrap();

    assert_eq!(summary.matched_by, MetadataMatchKind::ExternalId);
    assert_eq!(summary.provider_key, "8");
    assert!(raw.body_json.contains(r#""episodes""#));
    assert_eq!(media_items.len(), 1);
    assert_eq!(provider_mappings.len(), 1);
    assert_eq!(provider_mappings[0].status, ProviderMappingStatus::Accepted);
    assert_eq!(provider_subjects.len(), 1);
    assert_eq!(
        provider_subjects[0].subject_kind,
        ProviderSubjectKind::Series
    );
    assert_eq!(provider_subjects[0].subject_key, "8");
    assert!(episode_subject.is_none());
}

#[tokio::test]
async fn strategy_falls_back_from_unimplemented_bangumi_to_tmdb() {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

    let NakoError::Provider { provider, message } = err else {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    tmdb.fetch_result = Err(NakoError::Provider {
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

    let NakoError::Provider { provider, message } = err else {
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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
    let store = NakoDatabase::connect_in_memory().await.unwrap();
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

    assert_eq!(metadata.title.as_deref(), Some("The Matrix"));
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
fn metadata_candidate_graph_projects_provider_payload_without_becoming_raw_payload() {
    let details: TmdbMovieDetails = serde_json::from_str(
        r#"
            {
              "id": 603,
              "title": "The Matrix",
              "original_title": "The Matrix",
              "overview": "A hacker discovers the nature of reality.",
              "release_date": "1999-03-31",
              "runtime": 136,
              "genres": [{"id": 28, "name": "Action"}],
              "poster_path": "/poster.jpg",
              "credits": {"cast": [], "crew": []},
              "images": {"posters": [], "backdrops": [], "logos": []},
              "release_dates": {"results": []},
              "external_ids": {"imdb_id": "tt0133093"}
            }
            "#,
    )
    .unwrap();

    let graph = MetadataCandidateGraph::for_provider(
        ExternalProvider::Tmdb,
        MediaKind::Movie,
        ProviderSubjectKind::Movie,
        details.id.to_string(),
        tmdb_movie_details_to_metadata(details, DEFAULT_TMDB_IMAGE_BASE_URL),
    );
    let canonical = graph.canonical_metadata();
    let subject = graph.root_provider_subject().unwrap();

    assert_eq!(graph.root.kind, MediaKind::Movie);
    assert_eq!(canonical.title, "The Matrix");
    assert_eq!(canonical.runtime_minutes, Some(136));
    assert!(canonical.external_ids.iter().any(|external_id| {
        external_id.provider == ExternalProvider::Tmdb && external_id.value == "603"
    }));
    assert_eq!(subject.provider, ExternalProvider::Tmdb);
    assert_eq!(subject.subject_kind, ProviderSubjectKind::Movie);
    assert_eq!(subject.subject_key, "603");
    assert_eq!(subject.title.as_deref(), Some("The Matrix"));
    assert_eq!(subject.release_year, Some(1999));
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

    assert_eq!(metadata.title.as_deref(), Some("星际牛仔"));
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

    assert_eq!(metadata.title.as_deref(), Some("肖申克的救赎"));
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
        user_agent: "nako-test-agent".to_owned(),
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
        vec!["nako-test-agent", "nako-test-agent"]
    );
    let status = runtime.status();
    assert!(!status.circuit_open);
    assert_eq!(status.consecutive_failures, 0);
    assert_eq!(status.last_error, None);
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
    assert!(runtime.status().last_rate_limit_wait_ms > 0);
}

#[tokio::test]
async fn metadata_http_runtime_records_failure_status() {
    let server = MockMetadataServer::start().await;
    let runtime = MetadataHttpRuntime::new(MetadataHttpRuntimeConfig {
        max_attempts: 1,
        min_interval_ms: 0,
        circuit_breaker_failures: 1,
        ..MetadataHttpRuntimeConfig::default()
    })
    .unwrap();

    let err = runtime
        .get_json(
            "mock",
            "bad request",
            server.url("/bad-request"),
            &[],
            HeaderMap::new(),
        )
        .await
        .unwrap_err();
    let status = runtime.status();

    assert!(err.to_string().contains("HTTP 400"));
    assert!(status.circuit_open);
    assert!(status.circuit_open_until_ms.is_some());
    assert_eq!(status.consecutive_failures, 1);
    assert!(
        status
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("HTTP 400"))
    );
    let second = runtime
        .get_json(
            "mock",
            "bad request",
            server.url("/bad-request"),
            &[],
            HeaderMap::new(),
        )
        .await
        .unwrap_err();
    assert!(second.to_string().contains("circuit breaker is open"));
    assert_eq!(server.request_count(), 1);
}

#[tokio::test]
async fn tmdb_provider_uses_runtime_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = TmdbMetadataProvider::new(TmdbProviderConfig {
        read_access_token: fixtures::TMDB_TOKEN.into(),
        api_base_url: server.url("/tmdb"),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "nako-tmdb-test".to_owned(),
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
    assert_eq!(fetched.metadata().title, "The Matrix");
    assert_eq!(
        server.user_agents(),
        vec!["nako-tmdb-test", "nako-tmdb-test"]
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
async fn tmdb_provider_supports_series_season_and_episode_fetches() {
    let server = MockMetadataServer::start().await;
    let provider = TmdbMetadataProvider::new(TmdbProviderConfig {
        read_access_token: fixtures::TMDB_TOKEN.into(),
        api_base_url: server.url("/tmdb"),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "nako-tmdb-test".to_owned(),
            ..MetadataHttpRuntimeConfig::default()
        },
        ..TmdbProviderConfig::new(fixtures::TMDB_TOKEN.to_owned())
    })
    .unwrap();

    let candidates = provider
        .search(MetadataLookup {
            kind: Some(MediaKind::Series),
            title: "Firefly".to_owned(),
            year: Some(2002),
            language: Some("en-US".to_owned()),
            external_ids: Vec::new(),
        })
        .await
        .unwrap();
    let series = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Series,
            provider_key: candidates[0].provider_key.clone(),
            language: Some("en-US".to_owned()),
        })
        .await
        .unwrap();
    let season = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Season,
            provider_key: "1437/1".to_owned(),
            language: Some("en-US".to_owned()),
        })
        .await
        .unwrap();
    let episode = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Episode,
            provider_key: "1437/1/2".to_owned(),
            language: Some("en-US".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidates[0].provider_key, "1437");
    assert_eq!(candidates[0].metadata().title, "Firefly");
    assert_eq!(series.provider_key, "1437");
    assert_eq!(series.metadata().title, "Firefly");
    let series_subject = series.graph.root.subject.as_ref().unwrap();
    assert_eq!(series_subject.subject_kind, ProviderSubjectKind::Series);
    assert_eq!(series_subject.subject_key, "1437");
    assert_eq!(series.graph.related.len(), 1);
    let season_node = &series.graph.related[0];
    assert_eq!(season_node.kind, MediaKind::Season);
    assert_eq!(season_node.metadata.title.as_deref(), Some("Season 1"));
    assert_eq!(
        season_node.metadata.release_date.as_deref(),
        Some("2002-09-20")
    );
    let season_subject = season_node.subject.as_ref().unwrap();
    assert_eq!(season_subject.provider, ExternalProvider::Tmdb);
    assert_eq!(season_subject.subject_kind, ProviderSubjectKind::Season);
    assert_eq!(season_subject.subject_key, "1437/1");
    assert_eq!(season_subject.title.as_deref(), Some("Season 1"));
    assert_eq!(season_subject.release_year, Some(2002));
    assert_eq!(series.graph.relationships.len(), 1);
    let relationship = &series.graph.relationships[0];
    assert_eq!(
        relationship.kind,
        MetadataCandidateRelationshipKind::Contains
    );
    assert_eq!(relationship.parent_subject, series_subject.clone());
    assert_eq!(relationship.child_subject, season_subject.clone());
    assert_eq!(season.provider_key, "1437/1");
    assert_eq!(season.metadata().title, "Season 1");
    let fetched_season_subject = season.graph.root.subject.as_ref().unwrap();
    assert_eq!(
        fetched_season_subject.subject_kind,
        ProviderSubjectKind::Season
    );
    assert_eq!(fetched_season_subject.subject_key, "1437/1");
    assert_eq!(season.graph.related.len(), 1);
    let episode_node = &season.graph.related[0];
    assert_eq!(episode_node.kind, MediaKind::Episode);
    assert_eq!(
        episode_node.metadata.title.as_deref(),
        Some("The Train Job")
    );
    assert_eq!(
        episode_node.metadata.release_date.as_deref(),
        Some("2002-09-20")
    );
    assert_eq!(episode_node.metadata.runtime_minutes, Some(45));
    let episode_subject = episode_node.subject.as_ref().unwrap();
    assert_eq!(episode_subject.provider, ExternalProvider::Tmdb);
    assert_eq!(episode_subject.subject_kind, ProviderSubjectKind::Episode);
    assert_eq!(episode_subject.subject_key, "1437/1/2");
    assert_eq!(episode_subject.title.as_deref(), Some("The Train Job"));
    assert_eq!(episode_subject.release_year, Some(2002));
    assert_eq!(season.graph.relationships.len(), 1);
    let relationship = &season.graph.relationships[0];
    assert_eq!(
        relationship.kind,
        MetadataCandidateRelationshipKind::Contains
    );
    assert_eq!(relationship.parent_subject, fetched_season_subject.clone());
    assert_eq!(relationship.child_subject, episode_subject.clone());
    assert_eq!(episode.provider_key, "1437/1/2");
    assert_eq!(episode.metadata().title, "The Train Job");
    assert_eq!(
        episode.metadata().release_date,
        Some("2002-09-20".to_owned())
    );
    assert!(
        server
            .uris()
            .iter()
            .any(|uri| uri.contains("/tmdb/search/tv"))
    );
    assert!(
        server
            .uris()
            .iter()
            .any(|uri| uri.contains("/tmdb/tv/1437/season/1/episode/2"))
    );
}

#[test]
fn provider_configs_redact_resolved_secrets_in_debug_output() {
    let tmdb = TmdbProviderConfig::new(fixtures::TMDB_TOKEN);
    let bangumi = BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.into()),
        ..BangumiProviderConfig::default()
    };
    let douban = DoubanProviderConfig {
        api_key: Some(fixtures::DOUBAN_API_KEY.into()),
        headers: vec![("X-Douban-Secret".to_owned(), "header-secret".into())],
        ..DoubanProviderConfig::default()
    };

    let debug = format!("{tmdb:?}\n{bangumi:?}\n{douban:?}");

    assert!(!debug.contains(fixtures::TMDB_TOKEN));
    assert!(!debug.contains(fixtures::BANGUMI_TOKEN));
    assert!(!debug.contains(fixtures::DOUBAN_API_KEY));
    assert!(!debug.contains("header-secret"));
    assert!(debug.contains("<redacted>"));
}

#[tokio::test]
async fn bangumi_provider_uses_runtime_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = BangumiMetadataProvider::new(BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.into()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "nako-bangumi-test".to_owned(),
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
    assert_eq!(fetched.metadata().title, "星际牛仔");
    let series_subject = fetched.graph.root_provider_subject().unwrap().clone();
    assert_eq!(series_subject.subject_kind, ProviderSubjectKind::Series);
    assert_eq!(series_subject.subject_key, "8");
    assert_eq!(fetched.graph.related.len(), 1);
    let episode_node = &fetched.graph.related[0];
    assert_eq!(episode_node.kind, MediaKind::Episode);
    assert_eq!(episode_node.metadata.title.as_deref(), Some("阿斯特罗蓝调"));
    assert_eq!(
        episode_node.metadata.original_title.as_deref(),
        Some("Asteroid Blues")
    );
    assert_eq!(
        episode_node.metadata.release_date.as_deref(),
        Some("1998-04-03")
    );
    assert_eq!(episode_node.metadata.runtime_minutes, Some(24));
    assert!(
        episode_node
            .metadata
            .external_ids
            .iter()
            .any(
                |external_id| external_id.provider == ExternalProvider::Bangumi
                    && external_id.value == "101"
            )
    );
    let episode_subject = episode_node.subject.as_ref().unwrap();
    assert_eq!(episode_subject.provider, ExternalProvider::Bangumi);
    assert_eq!(episode_subject.subject_kind, ProviderSubjectKind::Episode);
    assert_eq!(episode_subject.subject_key, "101");
    assert_eq!(episode_subject.title.as_deref(), Some("阿斯特罗蓝调"));
    assert_eq!(episode_subject.release_year, Some(1998));
    assert_eq!(fetched.graph.relationships.len(), 1);
    let relationship = &fetched.graph.relationships[0];
    assert_eq!(
        relationship.kind,
        MetadataCandidateRelationshipKind::Contains
    );
    assert_eq!(relationship.parent_subject, series_subject);
    assert_eq!(relationship.child_subject, episode_subject.clone());
    assert!(fetched.raw_json.contains(r#""episodes""#));
    assert!(
        server
            .authorizations()
            .iter()
            .any(|value| value == &format!("Bearer {}", fixtures::BANGUMI_TOKEN))
    );
    assert!(server.uris().iter().any(|uri| uri.contains("/v0/episodes")
        && uri.contains("subject_id=8")
        && uri.contains("type=0")));
}

#[tokio::test]
async fn bangumi_provider_rejects_season_episode_until_endpoint_backed() {
    let server = MockMetadataServer::start().await;
    let provider = BangumiMetadataProvider::new(BangumiProviderConfig {
        access_token: Some(fixtures::BANGUMI_TOKEN.into()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            user_agent: "nako-bangumi-test".to_owned(),
            ..MetadataHttpRuntimeConfig::default()
        },
        ..BangumiProviderConfig::default()
    })
    .unwrap();

    let search_err = provider
        .search(MetadataLookup {
            kind: Some(MediaKind::Episode),
            title: "Asteroid Blues".to_owned(),
            year: Some(1998),
            language: Some("zh-CN".to_owned()),
            external_ids: Vec::new(),
        })
        .await
        .unwrap_err();
    let fetch_err = provider
        .fetch(MetadataFetchRequest {
            kind: MediaKind::Episode,
            provider_key: "8/1".to_owned(),
            language: Some("zh-CN".to_owned()),
        })
        .await
        .unwrap_err();

    assert!(matches!(search_err, NakoError::Unsupported(_)));
    assert!(matches!(fetch_err, NakoError::Unsupported(_)));
    assert!(server.uris().is_empty());
}

#[tokio::test]
async fn douban_provider_uses_api_key_and_maps_http_response() {
    let server = MockMetadataServer::start().await;
    let provider = DoubanMetadataProvider::new(DoubanProviderConfig {
        api_key: Some(fixtures::DOUBAN_API_KEY.into()),
        api_base_url: server.base_url(),
        runtime: MetadataHttpRuntimeConfig {
            min_interval_ms: 0,
            ..MetadataHttpRuntimeConfig::default()
        },
        headers: vec![("X-Douban-Test".to_owned(), "ok".into())],
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
    assert_eq!(fetched.metadata().title, "肖申克的救赎");
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
    store: &NakoDatabase,
    title: &str,
    release_date: Option<String>,
    external_ids: Vec<ExternalId>,
) -> MediaItem {
    seed_media_item(
        store,
        LibraryPreset::Movies,
        MediaKind::Movie,
        title,
        release_date,
        external_ids,
    )
    .await
}

async fn seed_media_item(
    store: &NakoDatabase,
    preset: LibraryPreset,
    kind: MediaKind,
    title: &str,
    release_date: Option<String>,
    external_ids: Vec<ExternalId>,
) -> MediaItem {
    let library = Library {
        id: LibraryId::new(),
        name: format!("{preset:?}"),
        roots: vec![format!("local:///{preset:?}")],
        options: LibraryOptions::from_preset(preset),
    };
    let item = MediaItem {
        id: MediaItemId::new(),
        kind,
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
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: true,
        })
        .await
        .unwrap();
    item
}

async fn seed_metadata_job(store: &NakoDatabase, item: &MediaItem) -> JobId {
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
    scored_candidate(provider, provider_key, title, 0.95)
}

fn scored_candidate(
    provider: ExternalProvider,
    provider_key: &str,
    title: &str,
    score: f32,
) -> MetadataCandidate {
    scored_candidate_with_release_date(provider, provider_key, title, None, score)
}

fn scored_candidate_with_release_date(
    provider: ExternalProvider,
    provider_key: &str,
    title: &str,
    release_date: Option<&str>,
    score: f32,
) -> MetadataCandidate {
    MetadataCandidate {
        provider: provider.clone(),
        provider_key: provider_key.to_owned(),
        score,
        graph: MetadataCandidateGraph::from_canonical(
            MetadataCandidateSource::Provider(provider),
            MediaKind::Movie,
            CanonicalMetadata {
                title: title.to_owned(),
                release_date: release_date.map(str::to_owned),
                ..CanonicalMetadata::default()
            },
        ),
    }
}

fn mock_fetch_result(
    provider: ExternalProvider,
    provider_key: &str,
    metadata: CanonicalMetadata,
) -> MetadataFetchResult {
    MetadataFetchResult {
        provider: provider.clone(),
        provider_key: provider_key.to_owned(),
        graph: MetadataCandidateGraph::from_canonical(
            MetadataCandidateSource::Provider(provider),
            MediaKind::Movie,
            metadata,
        ),
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
            .route("/bad-request", get(mock_bad_request))
            .route("/flaky", get(mock_flaky))
            .route("/tmdb/search/movie", get(mock_tmdb_search_movie))
            .route("/tmdb/search/tv", get(mock_tmdb_search_tv))
            .route("/tmdb/movie/{id}", get(mock_tmdb_movie_details))
            .route("/tmdb/tv/{id}", get(mock_tmdb_tv_details))
            .route(
                "/tmdb/tv/{id}/season/{season}",
                get(mock_tmdb_season_details),
            )
            .route(
                "/tmdb/tv/{id}/season/{season}/episode/{episode}",
                get(mock_tmdb_episode_details),
            )
            .route("/v0/search/subjects", post(mock_bangumi_search))
            .route("/v0/subjects/{id}", get(mock_bangumi_subject))
            .route("/v0/episodes", get(mock_bangumi_episodes))
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

async fn mock_bad_request(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Response {
    record_request(&state, &headers, &uri);
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": "bad request"})),
    )
        .into_response()
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

async fn mock_tmdb_search_tv(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "results": [{
            "id": 1437,
            "name": "Firefly",
            "original_name": "Firefly",
            "overview": "A crew aboard a small transport ship.",
            "first_air_date": "2002-09-20",
            "poster_path": "/firefly.jpg",
            "popularity": 42.0
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

async fn mock_tmdb_tv_details(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": 1437,
        "name": "Firefly",
        "original_name": "Firefly",
        "overview": "A crew aboard a small transport ship.",
        "first_air_date": "2002-09-20",
        "episode_run_time": [45],
        "tagline": "You can't take the sky from me.",
        "genres": [{"name": "Sci-Fi & Fantasy"}],
        "poster_path": "/firefly.jpg",
        "backdrop_path": "/firefly-backdrop.jpg",
        "credits": {"cast": [], "crew": []},
        "images": {"posters": [], "backdrops": []},
        "external_ids": {"imdb_id": "tt0303461"},
        "seasons": [{
            "id": 3624,
            "name": "Season 1",
            "overview": "The first season.",
            "air_date": "2002-09-20",
            "season_number": 1,
            "poster_path": "/firefly-s1.jpg"
        }]
    }))
}

async fn mock_tmdb_season_details(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": 3624,
        "name": "Season 1",
        "overview": "The first season.",
        "air_date": "2002-09-20",
        "season_number": 1,
        "poster_path": "/firefly-s1.jpg",
        "credits": {"cast": [], "crew": []},
        "images": {"posters": []},
        "episodes": [{
            "id": 12345,
            "name": "The Train Job",
            "overview": "The crew takes a train heist job.",
            "air_date": "2002-09-20",
            "season_number": 1,
            "episode_number": 2,
            "runtime": 45,
            "still_path": "/train-job.jpg"
        }]
    }))
}

async fn mock_tmdb_episode_details(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "id": 12345,
        "name": "The Train Job",
        "overview": "The crew takes a train heist job.",
        "air_date": "2002-09-20",
        "season_number": 1,
        "episode_number": 2,
        "runtime": 45,
        "still_path": "/train-job.jpg",
        "credits": {"cast": [], "crew": []},
        "images": {"backdrops": []}
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

async fn mock_bangumi_episodes(
    State(state): State<MockMetadataState>,
    headers: AxumHeaderMap,
    uri: Uri,
) -> Json<serde_json::Value> {
    record_request(&state, &headers, &uri);
    Json(json!({
        "data": [{
            "id": 101,
            "type": 0,
            "name": "Asteroid Blues",
            "name_cn": "阿斯特罗蓝调",
            "sort": 1.0,
            "ep": 1.0,
            "airdate": "1998-04-03",
            "comment": 0,
            "duration": "24m",
            "duration_seconds": 1440,
            "desc": "Spike and Jet chase a bounty head."
        }],
        "total": 1,
        "limit": 100,
        "offset": 0
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
