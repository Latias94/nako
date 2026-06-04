mod failure;
mod index;
mod ingestion;
mod intake;
mod local_inference;
mod probe;
mod scan;
mod source_hash;
mod summary;

pub use index::*;
pub use ingestion::*;
pub use intake::*;
pub use probe::*;
pub use scan::*;
pub use source_hash::*;
pub use summary::*;

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use axum::{
        Router,
        http::{StatusCode, header},
        response::{IntoResponse, Response},
        routing::any,
    };
    use nako_core::{
        CanonicalMetadata, DatabaseLifecycle, IngestionFailureClass, IngestionFailureFilter,
        IngestionFailurePhase, IngestionFailureRepository, IngestionFailureStatus, JobId, Library,
        LibraryId, LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryPreset,
        LibraryRepository, LocalInferenceEvidenceSource, LocalInferenceRepository, MediaItem,
        MediaItemId, MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository,
        MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind, NakoError, PageRequest,
        Result, ScanRepository, ScanStatus, SourceDuplicateEvidenceKind,
        SourceDuplicateRelationshipStatus, SourceDuplicateRepository, SourceState,
    };
    use nako_db::NakoDatabase;
    use nako_media_probe::{MediaProbe, MediaProbeRequest};
    use nako_search::{SearchIndex, SearchQuery};
    use nako_vfs::{
        ByteRange, CachedStorageBackend, LocalFsBackend, ObjectKind, ObjectListing, ObjectMetadata,
        StorageBackend, StorageCapabilities, StorageUri, VfsCacheOptions, WebDavBackend,
        WebDavBackendConfig,
    };
    use tokio::time::sleep;

    use super::*;

    #[test]
    fn vfs_scanner_discovers_supported_media_recursively() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir_all(temp.path().join("Movies").join("Demo Movie")).unwrap();
            fs::write(
                temp.path()
                    .join("Movies")
                    .join("Demo Movie")
                    .join("demo.MKV"),
                b"demo",
            )
            .unwrap();
            fs::write(
                temp.path()
                    .join("Movies")
                    .join("Demo Movie")
                    .join("poster.jpg"),
                b"image",
            )
            .unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let scanner = VfsLibraryScanner::new(backend);
            let summary = scanner
                .scan(LibraryScanRequest {
                    job_id: JobId::new(),
                    library_id: LibraryId::new(),
                    root: StorageUri::from_parts("local", "Movies").unwrap(),
                    force: false,
                })
                .await
                .unwrap();

            assert_eq!(summary.discovered_files, 1);
            assert_eq!(summary.media_sources.len(), 1);
            assert_eq!(summary.media_sources[0].file_name, "demo.MKV");
            assert_eq!(
                summary.media_sources[0].uri.as_str(),
                "local:///Movies/Demo Movie/demo.MKV"
            );
        });
    }

    #[tokio::test]
    async fn source_identity_scan_derives_redacted_fingerprint_without_full_file_hash() {
        let opened_ranges = Arc::new(AtomicUsize::new(0));
        let scanner = VfsLibraryScanner::new(SourceIdentityEvidenceBackend {
            opened_ranges: opened_ranges.clone(),
        });

        let summary = scanner
            .scan(LibraryScanRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                root: StorageUri::from_parts("evidence", "Movies").unwrap(),
                force: false,
            })
            .await
            .unwrap();

        let source = &summary.media_sources[0];
        let fingerprint = source.fingerprint.as_deref().unwrap();

        assert_eq!(summary.discovered_files, 1);
        assert!(fingerprint.starts_with("source:v1:size_etag:sha256:"));
        assert!(!fingerprint.contains("secret-etag"));
        assert_eq!(opened_ranges.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn source_identity_scan_does_not_merge_distinct_locators_with_same_fingerprint() {
        let include_duplicate = Arc::new(AtomicBool::new(false));
        let scanner = VfsLibraryScanner::new(SourceIdentityDuplicateBackend {
            include_duplicate: include_duplicate.clone(),
        });
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["duplicate:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        include_duplicate.store(true, Ordering::SeqCst);
        let second_summary = service.index_library(request).await.unwrap();
        let mut sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        sources.sort_by(|left, right| left.locator.cmp(&right.locator));

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(second_summary.inserted_sources, 1);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].locator, "duplicate:///Movies/Copy.mkv");
        assert_eq!(sources[1].locator, "duplicate:///Movies/Original.mkv");
        assert_ne!(sources[0].id, sources[1].id);
        assert_eq!(sources[0].fingerprint, sources[1].fingerprint);
        assert!(
            sources[0]
                .fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("source:v1:size_etag:sha256:"))
        );
        let relationships = SourceDuplicateRepository::list_source_duplicate_relationships(
            &store,
            sources[0].id,
            PageRequest::first_page(),
        )
        .await
        .unwrap();

        assert_eq!(relationships.len(), 1);
        assert!(
            relationships[0].source_id == sources[0].id
                || relationships[0].source_id == sources[1].id
        );
        assert!(
            relationships[0].duplicate_source_id == sources[0].id
                || relationships[0].duplicate_source_id == sources[1].id
        );
        assert_ne!(
            relationships[0].source_id,
            relationships[0].duplicate_source_id
        );
        assert_eq!(
            relationships[0].evidence_kind,
            SourceDuplicateEvidenceKind::SizeAndEtag
        );
        assert_eq!(
            relationships[0].status,
            SourceDuplicateRelationshipStatus::Suggested
        );
        assert_eq!(relationships[0].confidence_milli, Some(800));
        assert!(
            relationships[0]
                .evidence_value
                .as_deref()
                .is_some_and(|value| value.starts_with("source:v1:size_etag:sha256:"))
        );
    }

    #[tokio::test]
    async fn source_identity_scan_does_not_merge_strong_duplicate_present_in_same_scan() {
        let include_duplicate = Arc::new(AtomicBool::new(false));
        let scanner = VfsLibraryScanner::new(SourceIdentityStrongDuplicateBackend {
            include_duplicate: include_duplicate.clone(),
        });
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["strongdup:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        include_duplicate.store(true, Ordering::SeqCst);
        let second_summary = service.index_library(request).await.unwrap();
        let mut sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        sources.sort_by(|left, right| left.locator.cmp(&right.locator));

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(second_summary.inserted_sources, 1);
        assert_eq!(second_summary.tombstoned_sources, 0);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].locator, "strongdup:///Movies/Copy.mkv");
        assert_eq!(sources[1].locator, "strongdup:///Movies/Original.mkv");
        assert_ne!(sources[0].id, sources[1].id);
        assert_eq!(sources[0].fingerprint, sources[1].fingerprint);
        assert!(
            sources[0]
                .fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("source:v1:content_hash:sha256:"))
        );

        let relationships = SourceDuplicateRepository::list_source_duplicate_relationships(
            &store,
            sources[0].id,
            PageRequest::first_page(),
        )
        .await
        .unwrap();

        assert_eq!(relationships.len(), 1);
        assert!(
            relationships[0].source_id == sources[0].id
                || relationships[0].source_id == sources[1].id
        );
        assert!(
            relationships[0].duplicate_source_id == sources[0].id
                || relationships[0].duplicate_source_id == sources[1].id
        );
        assert_ne!(
            relationships[0].source_id,
            relationships[0].duplicate_source_id
        );
        assert_eq!(
            relationships[0].evidence_kind,
            SourceDuplicateEvidenceKind::StrongFingerprint
        );
        assert_eq!(
            relationships[0].status,
            SourceDuplicateRelationshipStatus::Suggested
        );
        assert_eq!(relationships[0].confidence_milli, Some(1_000));
        assert!(
            relationships[0]
                .evidence_value
                .as_deref()
                .is_some_and(|value| value.starts_with("source:v1:content_hash:sha256:"))
        );
    }

    #[tokio::test]
    async fn source_identity_scan_keeps_locator_only_evidence_without_fingerprint() {
        let scanner = VfsLibraryScanner::new(SourceIdentityWeakBackend);

        let summary = scanner
            .scan(LibraryScanRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                root: StorageUri::from_parts("weak", "Movies").unwrap(),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.discovered_files, 1);
        assert_eq!(
            summary.media_sources[0].uri.as_str(),
            "weak:///Movies/LocatorOnly.mkv"
        );
        assert_eq!(summary.media_sources[0].fingerprint, None);
    }

    #[tokio::test]
    async fn rename_reconciliation_preserves_media_source_for_strong_fingerprint_move() {
        let renamed = Arc::new(AtomicBool::new(false));
        let scanner = VfsLibraryScanner::new(RenameReconciliationBackend {
            renamed: renamed.clone(),
        });
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["rename:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        let first_source =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap()
                .pop()
                .unwrap();
        let mut confirmed_item = store
            .get_media_item(first_source.item_id)
            .await
            .unwrap()
            .unwrap();
        confirmed_item.metadata.title = "Curated Identity".to_owned();
        store.upsert_media_item(&confirmed_item).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id: library.id,
                item_id: confirmed_item.id,
                provisional: false,
            })
            .await
            .unwrap();

        renamed.store(true, Ordering::SeqCst);
        let second_summary = service.index_library(request).await.unwrap();
        let sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        let moved_source = &sources[0];
        let old_state = store
            .get_source_state(library.id, "rename:///Movies/Original.mkv")
            .await
            .unwrap()
            .unwrap();
        let new_state = store
            .get_source_state(library.id, "rename:///Movies/Renamed.mkv")
            .await
            .unwrap()
            .unwrap();
        let loaded_item = store
            .get_media_item(confirmed_item.id)
            .await
            .unwrap()
            .unwrap();
        let item_state = store
            .get_library_item_state(library.id, confirmed_item.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.inserted_sources, 0);
        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(second_summary.tombstoned_sources, 1);
        assert_eq!(sources.len(), 1);
        assert_eq!(moved_source.id, first_source.id);
        assert_eq!(moved_source.item_id, confirmed_item.id);
        assert_eq!(moved_source.locator, "rename:///Movies/Renamed.mkv");
        assert_eq!(loaded_item.metadata.title, "Curated Identity");
        assert!(!item_state.provisional);
        assert!(old_state.tombstoned);
        assert_eq!(new_state.source_id, Some(first_source.id));
        assert!(!new_state.tombstoned);
    }

    #[test]
    fn vfs_scanner_respects_custom_extensions() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::write(temp.path().join("playlist.strm"), b"http://example.test").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let scanner = VfsLibraryScanner::with_options(
                backend,
                LibraryScannerOptions {
                    media_extensions: vec!["strm".to_owned()],
                    max_depth: 1,
                },
            );
            let summary = scanner
                .scan(LibraryScanRequest {
                    job_id: JobId::new(),
                    library_id: LibraryId::new(),
                    root: StorageUri::from_parts("local", "").unwrap(),
                    force: false,
                })
                .await
                .unwrap();

            assert_eq!(summary.discovered_files, 1);
            assert_eq!(summary.media_sources[0].file_name, "playlist.strm");
        });
    }

    #[tokio::test]
    async fn vfs_scanner_discovers_webdav_media_without_credentials_in_locator() {
        let server = MockWebDavServer::start().await;
        let backend = WebDavBackend::new(WebDavBackendConfig {
            base_url: server.base_url(),
            username: None,
            password_env: None,
            timeout_ms: 5_000,
            max_attempts: 2,
        })
        .unwrap();
        let scanner = VfsLibraryScanner::new(backend);

        let summary = scanner
            .scan(LibraryScanRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                root: StorageUri::from_parts("webdav", "Movies").unwrap(),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.discovered_files, 1);
        assert_eq!(summary.media_sources[0].file_name, "Remote Movie.mkv");
        assert_eq!(
            summary.media_sources[0].uri.as_str(),
            "webdav:///Movies/Remote Movie.mkv"
        );
        assert!(!summary.media_sources[0].uri.as_str().contains('@'));
        assert_eq!(summary.directories.len(), 1);
        assert_eq!(summary.directories[0].uri.as_str(), "webdav:///Movies/");
    }

    #[tokio::test]
    async fn webdav_scan_records_partial_directory_failures() {
        let server = PartialFailureWebDavServer::start().await;
        let backend = WebDavBackend::new(WebDavBackendConfig {
            base_url: server.base_url(),
            username: None,
            password_env: None,
            timeout_ms: 5_000,
            max_attempts: 1,
        })
        .unwrap();
        let scanner = VfsLibraryScanner::new(backend);

        let summary = scanner
            .scan(LibraryScanRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                root: StorageUri::from_parts("webdav", "Movies").unwrap(),
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.discovered_files, 1);
        assert_eq!(
            summary.media_sources[0].uri.as_str(),
            "webdav:///Movies/Good.mkv"
        );
        assert_eq!(summary.failures.len(), 1);
        assert_eq!(summary.failures[0].uri.as_str(), "webdav:///Movies/Broken/");
        assert_eq!(
            summary.failures[0].failure_class,
            IngestionFailureClass::Storage
        );
        assert_eq!(summary.failures[0].message, "storage backend unavailable");
        assert!(!summary.failures[0].message.contains("Broken"));
        assert!(summary.failures[0].retryable);
    }

    #[tokio::test]
    async fn probe_service_stages_webdav_source_before_probe() {
        let server = MockWebDavServer::start().await;
        let backend = WebDavBackend::new(WebDavBackendConfig {
            base_url: server.base_url(),
            username: None,
            password_env: None,
            timeout_ms: 5_000,
            max_attempts: 2,
        })
        .unwrap();
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let staging_root = tempfile::tempdir().unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["webdav:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Remote Movie".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "webdav:///Movies/Remote Movie.mkv".to_owned(),
            file_name: "Remote Movie.mkv".to_owned(),
            size_bytes: Some(12),
            fingerprint: None,
        };
        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();

        let probe = RecordingProbe::default();
        let observed_paths = probe.observed_paths.clone();
        let service = LibraryProbeService::with_options(
            backend,
            probe,
            store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: 1,
                staging_root: Some(staging_root.path().to_path_buf()),
            },
        );

        let summary = service
            .probe_library(LibraryProbeRequest {
                job_id: JobId::new(),
                library_id: library.id,
                force: false,
            })
            .await
            .unwrap();
        let observed_path = observed_paths.lock().unwrap()[0].clone().unwrap();

        assert_eq!(summary.probed_sources, 1);
        assert!(observed_path.starts_with(staging_root.path()));
        assert_eq!(fs::read(&observed_path).unwrap(), b"remote movie");
        assert!(
            MediaProbeRepository::get_media_probe(&store, source.id)
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn index_service_persists_scan_results_idempotently() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(
            temp.path().join("Movies").join("The Matrix (1999).mkv"),
            b"movie",
        )
        .unwrap();
        fs::write(temp.path().join("Movies").join("poster.jpg"), b"image").unwrap();

        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let scanner = VfsLibraryScanner::new(backend);
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        let second_summary = service.index_library(request).await.unwrap();
        let sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        let item = store
            .get_media_item(sources[0].item_id)
            .await
            .unwrap()
            .unwrap();
        let scan = store
            .get_scan_snapshot(second_summary.scan_id)
            .await
            .unwrap()
            .unwrap();
        let directories = store
            .list_directory_snapshots(second_summary.scan_id)
            .await
            .unwrap();
        let state = store
            .get_source_state(library.id, "local:///Movies/The Matrix (1999).mkv")
            .await
            .unwrap()
            .unwrap();
        let evidence = store
            .list_local_inference_evidence_for_source(sources[0].id, PageRequest::first_page())
            .await
            .unwrap();
        let hits = store
            .search(SearchQuery::from_facet_labels("matrix", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap();

        assert_eq!(first_summary.discovered_files, 1);
        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(first_summary.updated_sources, 0);
        assert_eq!(first_summary.tombstoned_sources, 0);
        assert_eq!(second_summary.discovered_files, 1);
        assert_eq!(second_summary.inserted_sources, 0);
        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(second_summary.tombstoned_sources, 0);
        assert_eq!(scan.status, ScanStatus::Succeeded);
        assert!(!directories.is_empty());
        assert!(!state.tombstoned);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].file_name, "The Matrix (1999).mkv");
        assert_eq!(evidence.len(), 1);
        assert_eq!(item.metadata.title, "The Matrix");
        assert_eq!(item.metadata.release_date, Some("1999".to_owned()));
        assert_eq!(hits[0].item_id, item.id);
    }

    #[tokio::test]
    async fn index_service_preserves_confirmed_canonical_metadata_on_rescan() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(
            temp.path().join("Movies").join("The Matrix (1999).mkv"),
            b"movie",
        )
        .unwrap();

        let scanner = VfsLibraryScanner::new(LocalFsBackend::new(temp.path()).unwrap());
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        service.index_library(request.clone()).await.unwrap();
        let source =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap()
                .pop()
                .unwrap();
        let mut confirmed_item = store.get_media_item(source.item_id).await.unwrap().unwrap();
        confirmed_item.metadata.title = "Curated Matrix Title".to_owned();
        confirmed_item.metadata.release_date = Some("1999-03-31".to_owned());
        confirmed_item.metadata.overview = Some("Confirmed user metadata.".to_owned());
        store.upsert_media_item(&confirmed_item).await.unwrap();
        store
            .upsert_library_item_state(&LibraryItemState {
                library_id: library.id,
                item_id: confirmed_item.id,
                provisional: false,
            })
            .await
            .unwrap();

        let second_summary = service.index_library(request).await.unwrap();
        let loaded_item = store
            .get_media_item(confirmed_item.id)
            .await
            .unwrap()
            .unwrap();
        let state = store
            .get_library_item_state(library.id, confirmed_item.id)
            .await
            .unwrap()
            .unwrap();
        let evidence = store
            .list_local_inference_evidence_for_source(source.id, PageRequest::first_page())
            .await
            .unwrap();

        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(loaded_item, confirmed_item);
        assert!(!state.provisional);
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].inferred_title, Some("The Matrix".to_owned()));
    }

    #[tokio::test]
    async fn index_service_creates_provisional_hierarchy_and_local_inference_evidence() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("TV").join("Firefly").join("Season 01")).unwrap();
        fs::write(
            temp.path()
                .join("TV")
                .join("Firefly")
                .join("Season 01")
                .join("Firefly.S01E02.The Train Job.mkv"),
            b"episode",
        )
        .unwrap();

        let scanner = VfsLibraryScanner::new(LocalFsBackend::new(temp.path()).unwrap());
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "TV".to_owned(),
            roots: vec!["local:///TV".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Tv),
        };
        let service = LibraryIndexService::new(scanner, store.clone());

        let summary = service
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();
        let sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        let episode = store
            .get_media_item(sources[0].item_id)
            .await
            .unwrap()
            .unwrap();
        let season = store
            .get_media_item(episode.parent_id.unwrap())
            .await
            .unwrap()
            .unwrap();
        let series = store
            .get_media_item(season.parent_id.unwrap())
            .await
            .unwrap()
            .unwrap();
        let evidence = store
            .list_local_inference_evidence_for_source(sources[0].id, PageRequest::first_page())
            .await
            .unwrap();

        assert_eq!(summary.inserted_sources, 1);
        assert_eq!(series.kind, MediaKind::Series);
        assert_eq!(series.metadata.title, "Firefly");
        assert_eq!(season.kind, MediaKind::Season);
        assert_eq!(season.metadata.title, "Season 1");
        assert_eq!(episode.kind, MediaKind::Episode);
        assert_eq!(episode.parent_id, Some(season.id));
        assert_eq!(
            store
                .get_library_item_state(library.id, series.id)
                .await
                .unwrap()
                .unwrap()
                .provisional,
            true
        );
        assert_eq!(
            store
                .get_library_item_state(library.id, season.id)
                .await
                .unwrap()
                .unwrap()
                .provisional,
            true
        );
        assert_eq!(
            store
                .get_library_item_state(library.id, episode.id)
                .await
                .unwrap()
                .unwrap()
                .provisional,
            true
        );
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].inferred_kind, MediaKind::Episode);
        assert_eq!(evidence[0].inferred_title, Some("Firefly".to_owned()));
        assert_eq!(evidence[0].inferred_season, Some(1));
        assert_eq!(evidence[0].inferred_episode, Some(2));
        assert_eq!(evidence[0].confidence_milli, Some(900));
        assert_eq!(
            evidence[0].evidence_source,
            LocalInferenceEvidenceSource::FileName
        );
        assert_eq!(
            evidence[0].inference_version,
            nako_naming::DEFAULT_PARSER_VERSION
        );
    }

    #[tokio::test]
    async fn index_service_uses_unknown_item_for_weak_local_inference() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Uploads")).unwrap();
        fs::write(temp.path().join("Uploads").join("random.clip.mkv"), b"clip").unwrap();

        let scanner = VfsLibraryScanner::new(LocalFsBackend::new(temp.path()).unwrap());
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Uploads".to_owned(),
            roots: vec!["local:///Uploads".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::MixedVideo),
        };
        let service = LibraryIndexService::new(scanner, store.clone());

        service
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();
        let source =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap()
                .pop()
                .unwrap();
        let item = store.get_media_item(source.item_id).await.unwrap().unwrap();
        let evidence = store
            .list_local_inference_evidence_for_source(source.id, PageRequest::first_page())
            .await
            .unwrap();

        assert_eq!(item.kind, MediaKind::Unknown);
        assert_eq!(item.parent_id, None);
        assert_eq!(item.metadata.title, "random clip");
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].inferred_kind, MediaKind::Unknown);
        assert_eq!(evidence[0].confidence_milli, Some(350));
    }

    #[tokio::test]
    async fn index_and_probe_keep_identical_local_locators_isolated_by_library() {
        let first_root = tempfile::tempdir().unwrap();
        let second_root = tempfile::tempdir().unwrap();
        fs::write(first_root.path().join("Movie.mkv"), b"first").unwrap();
        fs::write(second_root.path().join("Movie.mkv"), b"second").unwrap();

        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let first_library = Library {
            id: LibraryId::new(),
            name: "First Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let second_library = Library {
            id: LibraryId::new(),
            name: "Second Movies".to_owned(),
            roots: vec!["local:///".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&first_library).await.unwrap();
        store.upsert_library(&second_library).await.unwrap();

        let first_summary = LibraryIndexService::new(
            VfsLibraryScanner::new(LocalFsBackend::new(first_root.path()).unwrap()),
            store.clone(),
        )
        .index_library(LibraryIndexRequest {
            job_id: JobId::new(),
            library: first_library.clone(),
            force: false,
        })
        .await
        .unwrap();
        let second_summary = LibraryIndexService::new(
            VfsLibraryScanner::new(LocalFsBackend::new(second_root.path()).unwrap()),
            store.clone(),
        )
        .index_library(LibraryIndexRequest {
            job_id: JobId::new(),
            library: second_library.clone(),
            force: false,
        })
        .await
        .unwrap();

        let first_sources = MediaRepository::list_media_sources(
            &store,
            first_library.id,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
        let second_sources = MediaRepository::list_media_sources(
            &store,
            second_library.id,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
        let first_state = store
            .get_source_state(first_library.id, "local:///Movie.mkv")
            .await
            .unwrap()
            .unwrap();
        let second_state = store
            .get_source_state(second_library.id, "local:///Movie.mkv")
            .await
            .unwrap()
            .unwrap();
        let first_item = store
            .get_media_item(first_sources[0].item_id)
            .await
            .unwrap()
            .unwrap();
        let second_item = store
            .get_media_item(second_sources[0].item_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.inserted_sources, 1);
        assert_eq!(first_sources.len(), 1);
        assert_eq!(second_sources.len(), 1);
        assert_eq!(first_sources[0].locator, "local:///Movie.mkv");
        assert_eq!(second_sources[0].locator, "local:///Movie.mkv");
        assert_eq!(first_sources[0].library_id, first_library.id);
        assert_eq!(second_sources[0].library_id, second_library.id);
        assert_ne!(first_sources[0].id, second_sources[0].id);
        assert_ne!(first_sources[0].item_id, second_sources[0].item_id);
        assert_eq!(first_state.source_id, Some(first_sources[0].id));
        assert_eq!(second_state.source_id, Some(second_sources[0].id));
        assert_eq!(first_item.metadata.title, "Movie");
        assert_eq!(second_item.metadata.title, "Movie");

        let first_probe = RecordingProbe::default();
        let first_observed_paths = first_probe.observed_paths.clone();
        LibraryProbeService::with_options(
            LocalFsBackend::new(first_root.path()).unwrap(),
            first_probe,
            store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: 1,
                staging_root: None,
            },
        )
        .probe_library(LibraryProbeRequest {
            job_id: JobId::new(),
            library_id: first_library.id,
            force: false,
        })
        .await
        .unwrap();

        let second_probe = RecordingProbe::default();
        let second_observed_paths = second_probe.observed_paths.clone();
        LibraryProbeService::with_options(
            LocalFsBackend::new(second_root.path()).unwrap(),
            second_probe,
            store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: 1,
                staging_root: None,
            },
        )
        .probe_library(LibraryProbeRequest {
            job_id: JobId::new(),
            library_id: second_library.id,
            force: false,
        })
        .await
        .unwrap();

        let first_observed_path = first_observed_paths.lock().unwrap()[0].clone().unwrap();
        let second_observed_path = second_observed_paths.lock().unwrap()[0].clone().unwrap();
        assert_eq!(fs::read(first_observed_path).unwrap(), b"first");
        assert_eq!(fs::read(second_observed_path).unwrap(), b"second");
        assert!(
            MediaProbeRepository::get_media_probe(&store, first_sources[0].id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            MediaProbeRepository::get_media_probe(&store, second_sources[0].id)
                .await
                .unwrap()
                .is_some()
        );

        let hits = store
            .search(SearchQuery::from_facet_labels("movie", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap();
        let hit_item_ids = hits.into_iter().map(|hit| hit.item_id).collect::<Vec<_>>();
        assert!(hit_item_ids.contains(&first_sources[0].item_id));
        assert!(hit_item_ids.contains(&second_sources[0].item_id));
    }

    #[tokio::test]
    async fn index_service_tombstones_sources_missing_from_rescan() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        let movie_path = temp.path().join("Movies").join("Gone Movie.mkv");
        fs::write(&movie_path, b"movie").unwrap();

        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let scanner = VfsLibraryScanner::new(backend);
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        fs::remove_file(movie_path).unwrap();
        let second_summary = service.index_library(request).await.unwrap();
        let state = store
            .get_source_state(library.id, "local:///Movies/Gone Movie.mkv")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.discovered_files, 0);
        assert_eq!(second_summary.tombstoned_sources, 1);
        assert!(state.tombstoned);
    }

    #[tokio::test]
    async fn index_service_does_not_tombstone_when_scan_uses_stale_vfs_cache() {
        let backend = FlakyRemoteBackend::new();
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let cached_backend = CachedStorageBackend::with_options(
            backend.clone(),
            store.clone(),
            VfsCacheOptions {
                stat_ttl_ms: 0,
                list_ttl_ms: 0,
                serve_stale_on_error: true,
                cache_local: true,
            },
        );
        let scanner = VfsLibraryScanner::new(cached_backend);

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["remote:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());
        let request = LibraryIndexRequest {
            job_id: JobId::new(),
            library: library.clone(),
            force: false,
        };

        let first_summary = service.index_library(request.clone()).await.unwrap();
        let missing_source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: MediaItemId::new(),
            locator: "remote:///Movies/Missing During Outage.mkv".to_owned(),
            file_name: "Missing During Outage.mkv".to_owned(),
            size_bytes: Some(9),
            fingerprint: Some("remote:missing".to_owned()),
        };
        store
            .upsert_media_item(&MediaItem {
                id: missing_source.item_id,
                kind: MediaKind::Movie,
                parent_id: None,
                metadata: CanonicalMetadata {
                    title: "Missing During Outage".to_owned(),
                    ..CanonicalMetadata::default()
                },
            })
            .await
            .unwrap();
        store.upsert_media_source(&missing_source).await.unwrap();
        store
            .upsert_source_state(&SourceState {
                library_id: library.id,
                source_id: Some(missing_source.id),
                uri: missing_source.locator.clone(),
                size_bytes: missing_source.size_bytes,
                modified_at: None,
                etag: None,
                fingerprint: missing_source.fingerprint.clone(),
                last_seen_scan_id: first_summary.scan_id,
                tombstoned: false,
            })
            .await
            .unwrap();

        backend.fail_list.store(true, Ordering::SeqCst);
        let second_summary = service.index_library(request).await.unwrap();
        let missing_state = store
            .get_source_state(library.id, &missing_source.locator)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(second_summary.discovered_files, 1);
        assert_eq!(second_summary.tombstoned_sources, 0);
        assert!(!missing_state.tombstoned);
    }

    #[tokio::test]
    async fn index_service_records_scan_failures_without_blocking_good_sources() {
        let backend = PartiallyFailingScanBackend;
        let scanner = VfsLibraryScanner::new(backend);
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["fixture:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let service = LibraryIndexService::new(scanner, store.clone());

        let summary = service
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();

        let sources =
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap();
        let failures = store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library.id),
                    phase: Some(IngestionFailurePhase::Scan),
                    status: Some(IngestionFailureStatus::Open),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap();

        assert_eq!(summary.discovered_files, 1);
        assert_eq!(summary.inserted_sources, 1);
        assert_eq!(summary.failed_entries, 1);
        assert_eq!(summary.tombstoned_sources, 0);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].locator, "fixture:///Movies/Good.mkv");
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].target_uri, "fixture:///Movies/Broken/");
        assert_eq!(failures[0].phase, IngestionFailurePhase::Scan);
        assert_eq!(failures[0].status, IngestionFailureStatus::Open);
        assert_eq!(failures[0].message, "storage backend unavailable");
        assert!(!failures[0].message.contains("Broken"));
        assert!(failures[0].retryable);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn probe_service_uses_bounded_concurrency_and_persists_results() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();

        for index in 0..4 {
            fs::write(
                temp.path()
                    .join("Movies")
                    .join(format!("Movie {index}.mkv")),
                b"movie",
            )
            .unwrap();
        }

        let index_backend = LocalFsBackend::new(temp.path()).unwrap();
        let probe_backend = LocalFsBackend::new(temp.path()).unwrap();
        let scanner = VfsLibraryScanner::new(index_backend);
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let index_service = LibraryIndexService::new(scanner, store.clone());
        index_service
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();

        let probe = RecordingProbe::default();
        let max_seen = probe.max_seen.clone();
        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            probe,
            store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: 2,
                staging_root: None,
            },
        );
        let summary = probe_service
            .probe_library(LibraryProbeRequest {
                job_id: JobId::new(),
                library_id: library.id,
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.total_sources, 4);
        assert_eq!(summary.probed_sources, 4);
        assert_eq!(summary.skipped_sources, 0);
        assert_eq!(summary.failed_sources, 0);
        assert!(max_seen.load(Ordering::SeqCst) <= 2);

        for source in
            MediaRepository::list_media_sources(&store, library.id, PageRequest::first_page())
                .await
                .unwrap()
        {
            assert!(
                MediaProbeRepository::get_media_probe(&store, source.id)
                    .await
                    .unwrap()
                    .is_some()
            );
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn probe_service_isolates_failures_and_skips_existing_results() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("Good Movie.mkv"), b"good").unwrap();
        fs::write(temp.path().join("Movies").join("Bad Movie.mkv"), b"bad").unwrap();

        let index_backend = LocalFsBackend::new(temp.path()).unwrap();
        let probe_backend = LocalFsBackend::new(temp.path()).unwrap();
        let scanner = VfsLibraryScanner::new(index_backend);
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        LibraryIndexService::new(scanner, store.clone())
            .index_library(LibraryIndexRequest {
                job_id: JobId::new(),
                library: library.clone(),
                force: false,
            })
            .await
            .unwrap();

        let probe_service = LibraryProbeService::with_options(
            probe_backend,
            RecordingProbe {
                fail_locator_fragment: Some("Bad Movie".to_owned()),
                ..RecordingProbe::default()
            },
            store.clone(),
            LibraryProbeOptions {
                max_concurrent_probes: 2,
                staging_root: None,
            },
        );

        let first_summary = probe_service
            .probe_library(LibraryProbeRequest {
                job_id: JobId::new(),
                library_id: library.id,
                force: false,
            })
            .await
            .unwrap();
        let second_summary = probe_service
            .probe_library(LibraryProbeRequest {
                job_id: JobId::new(),
                library_id: library.id,
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(first_summary.total_sources, 2);
        assert_eq!(first_summary.probed_sources, 1);
        assert_eq!(first_summary.skipped_sources, 0);
        assert_eq!(first_summary.failed_sources, 1);
        assert_eq!(second_summary.total_sources, 2);
        assert_eq!(second_summary.probed_sources, 0);
        assert_eq!(second_summary.skipped_sources, 1);
        assert_eq!(second_summary.failed_sources, 1);

        let failures = store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library.id),
                    phase: Some(IngestionFailurePhase::Probe),
                    status: Some(IngestionFailureStatus::Open),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].attempts, 2);
        assert!(failures[0].target_uri.contains("Bad Movie"));
    }

    #[derive(Clone, Default)]
    struct RecordingProbe {
        active: Arc<AtomicUsize>,
        max_seen: Arc<AtomicUsize>,
        observed_paths: Arc<std::sync::Mutex<Vec<Option<PathBuf>>>>,
        fail_locator_fragment: Option<String>,
    }

    #[async_trait::async_trait]
    impl MediaProbe for RecordingProbe {
        async fn probe(&self, request: MediaProbeRequest) -> Result<MediaProbeResult> {
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            update_max(&self.max_seen, active);
            self.observed_paths
                .lock()
                .unwrap()
                .push(request.local_path_hint.clone());

            sleep(Duration::from_millis(25)).await;

            self.active.fetch_sub(1, Ordering::SeqCst);

            if self
                .fail_locator_fragment
                .as_ref()
                .is_some_and(|fragment| request.source.as_str().contains(fragment))
            {
                return Err(NakoError::Provider {
                    provider: "recording-probe".to_owned(),
                    message: format!("probe failed for {}", request.source),
                });
            }

            Ok(MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: Some(1_000_000),
                streams: vec![MediaStreamInfo {
                    index: 0,
                    kind: MediaStreamKind::Video,
                    codec: Some("h264".to_owned()),
                    language: None,
                    duration_ms: Some(1_000),
                    bit_rate: Some(1_000_000),
                    width: Some(1920),
                    height: Some(1080),
                    channels: None,
                    sample_rate: None,
                    technical: Default::default(),
                }],
            })
        }
    }

    #[derive(Clone)]
    struct RenameReconciliationBackend {
        renamed: Arc<AtomicBool>,
    }

    impl RenameReconciliationBackend {
        fn current_path(&self) -> &'static str {
            if self.renamed.load(Ordering::SeqCst) {
                "Movies/Renamed.mkv"
            } else {
                "Movies/Original.mkv"
            }
        }

        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: (kind == ObjectKind::File).then_some(42),
                modified_at: Some("2026-05-29T00:00:00Z".to_owned()),
                etag: None,
                fingerprint: (kind == ObjectKind::File).then_some(
                    "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        .to_owned(),
                ),
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for RenameReconciliationBackend {
        fn scheme(&self) -> &'static str {
            "rename"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() == "rename:///Movies" {
                return Ok(ObjectListing {
                    entries: vec![Self::metadata(
                        StorageUri::from_parts("rename", self.current_path()).unwrap(),
                    )],
                    cache: None,
                });
            }

            Ok(ObjectListing {
                entries: Vec::new(),
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "rename reconciliation fixture does not read text",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "rename reconciliation fixture does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct SourceIdentityWeakBackend;

    impl SourceIdentityWeakBackend {
        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: None,
                modified_at: None,
                etag: None,
                fingerprint: None,
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for SourceIdentityWeakBackend {
        fn scheme(&self) -> &'static str {
            "weak"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() == "weak:///Movies" {
                return Ok(ObjectListing {
                    entries: vec![Self::metadata(
                        StorageUri::from_parts("weak", "Movies/LocatorOnly.mkv").unwrap(),
                    )],
                    cache: None,
                });
            }

            Ok(ObjectListing {
                entries: Vec::new(),
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "weak source identity fixture does not read text",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "weak source identity fixture does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct SourceIdentityDuplicateBackend {
        include_duplicate: Arc<AtomicBool>,
    }

    impl SourceIdentityDuplicateBackend {
        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: (kind == ObjectKind::File).then_some(42),
                modified_at: Some("2026-05-29T00:00:00Z".to_owned()),
                etag: (kind == ObjectKind::File).then_some("same-remote-etag".to_owned()),
                fingerprint: None,
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for SourceIdentityDuplicateBackend {
        fn scheme(&self) -> &'static str {
            "duplicate"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() != "duplicate:///Movies" {
                return Ok(ObjectListing {
                    entries: Vec::new(),
                    cache: None,
                });
            }

            let mut entries = vec![Self::metadata(
                StorageUri::from_parts("duplicate", "Movies/Original.mkv").unwrap(),
            )];
            if self.include_duplicate.load(Ordering::SeqCst) {
                entries.push(Self::metadata(
                    StorageUri::from_parts("duplicate", "Movies/Copy.mkv").unwrap(),
                ));
            }

            Ok(ObjectListing {
                entries,
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "source duplicate fixture does not read text",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "source duplicate fixture does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct SourceIdentityStrongDuplicateBackend {
        include_duplicate: Arc<AtomicBool>,
    }

    impl SourceIdentityStrongDuplicateBackend {
        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: (kind == ObjectKind::File).then_some(42),
                modified_at: Some("2026-05-29T00:00:00Z".to_owned()),
                etag: None,
                fingerprint: (kind == ObjectKind::File).then_some(
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_owned(),
                ),
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for SourceIdentityStrongDuplicateBackend {
        fn scheme(&self) -> &'static str {
            "strongdup"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() != "strongdup:///Movies" {
                return Ok(ObjectListing {
                    entries: Vec::new(),
                    cache: None,
                });
            }

            let mut entries = vec![Self::metadata(
                StorageUri::from_parts("strongdup", "Movies/Original.mkv").unwrap(),
            )];
            if self.include_duplicate.load(Ordering::SeqCst) {
                entries.push(Self::metadata(
                    StorageUri::from_parts("strongdup", "Movies/Copy.mkv").unwrap(),
                ));
            }

            Ok(ObjectListing {
                entries,
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "strong source duplicate fixture does not read text",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "strong source duplicate fixture does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct SourceIdentityEvidenceBackend {
        opened_ranges: Arc<AtomicUsize>,
    }

    impl SourceIdentityEvidenceBackend {
        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: (kind == ObjectKind::File).then_some(42),
                modified_at: Some("2026-05-29T00:00:00Z".to_owned()),
                etag: (kind == ObjectKind::File).then_some("secret-etag-for-source".to_owned()),
                fingerprint: None,
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for SourceIdentityEvidenceBackend {
        fn scheme(&self) -> &'static str {
            "evidence"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() == "evidence:///Movies" {
                return Ok(ObjectListing {
                    entries: vec![Self::metadata(
                        StorageUri::from_parts("evidence", "Movies/NoFullHash.mkv").unwrap(),
                    )],
                    cache: None,
                });
            }

            Ok(ObjectListing {
                entries: Vec::new(),
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            self.opened_ranges.fetch_add(1, Ordering::SeqCst);
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported(
                "source identity fixture does not read text",
            ))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "source identity fixture does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct PartiallyFailingScanBackend;

    impl PartiallyFailingScanBackend {
        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri,
                kind,
                len: (kind == ObjectKind::File).then_some(4),
                modified_at: Some("100".to_owned()),
                etag: None,
                fingerprint: Some("fixture:fingerprint".to_owned()),
                capabilities: StorageCapabilities::SEEKABLE | StorageCapabilities::RANGE_READABLE,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for PartiallyFailingScanBackend {
        fn scheme(&self) -> &'static str {
            "fixture"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(self.list_with_status(uri).await?.entries)
        }

        async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
            if uri.as_str() == "fixture:///Movies/Broken/" {
                return Err(NakoError::storage_io(
                    uri.to_string(),
                    "fixture directory is unreadable",
                ));
            }

            if uri.as_str() == "fixture:///Movies" {
                return Ok(ObjectListing {
                    entries: vec![
                        Self::metadata(StorageUri::parse("fixture:///Movies/Good.mkv").unwrap()),
                        Self::metadata(StorageUri::parse("fixture:///Movies/Broken/").unwrap()),
                    ],
                    cache: None,
                });
            }

            Ok(ObjectListing {
                entries: Vec::new(),
                cache: None,
            })
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported("fixture backend does not read text"))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported(
                "fixture backend does not write text",
            ))
        }
    }

    #[derive(Clone)]
    struct FlakyRemoteBackend {
        fail_list: Arc<AtomicBool>,
    }

    impl FlakyRemoteBackend {
        fn new() -> Self {
            Self {
                fail_list: Arc::new(AtomicBool::new(false)),
            }
        }

        fn metadata(uri: StorageUri) -> ObjectMetadata {
            let kind = if uri.as_str().ends_with(".mkv") {
                ObjectKind::File
            } else {
                ObjectKind::Directory
            };

            ObjectMetadata {
                uri: uri.clone(),
                kind,
                len: (kind == ObjectKind::File).then_some(4),
                modified_at: Some("100".to_owned()),
                etag: Some(format!("etag:{}", uri.as_str())),
                fingerprint: Some(format!("remote:{}", uri.as_str())),
                capabilities: StorageCapabilities::SEEKABLE
                    | StorageCapabilities::RANGE_READABLE
                    | StorageCapabilities::REMOTE_LATENCY
                    | StorageCapabilities::EXPENSIVE_LISTING,
                cache: None,
            }
        }
    }

    #[async_trait::async_trait]
    impl StorageBackend for FlakyRemoteBackend {
        fn scheme(&self) -> &'static str {
            "remote"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            if self.fail_list.load(Ordering::SeqCst) {
                return Err(NakoError::storage_timeout(
                    uri.to_string(),
                    "remote listing timed out",
                ));
            }

            Ok(vec![Self::metadata(
                StorageUri::from_parts("remote", "Movies/Remote Movie.mkv").unwrap(),
            )])
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<nako_vfs::VirtualFile> {
            Ok(nako_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::Unsupported("flaky remote does not read text"))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::Unsupported("flaky remote does not write text"))
        }
    }

    fn update_max(max_seen: &AtomicUsize, active: usize) {
        let mut current = max_seen.load(Ordering::SeqCst);

        while active > current {
            match max_seen.compare_exchange(current, active, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => break,
                Err(observed) => current = observed,
            }
        }
    }

    struct MockWebDavServer {
        addr: std::net::SocketAddr,
    }

    impl MockWebDavServer {
        async fn start() -> Self {
            let router = Router::new().route("/{*path}", any(webdav_handler));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });

            Self { addr }
        }

        fn base_url(&self) -> String {
            format!("http://{}/dav", self.addr)
        }
    }

    async fn webdav_handler(method: axum::http::Method, uri: axum::http::Uri) -> Response {
        let path = uri.path();
        if method.as_str() == "GET" && path.ends_with("/Movies/Remote%20Movie.mkv") {
            return "remote movie".into_response();
        }

        if method.as_str() != "PROPFIND" {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }

        if path.ends_with("/Movies/") || path.ends_with("/Movies") {
            return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/Movies/</D:href>
    <D:propstat><D:prop><D:resourcetype><D:collection/></D:resourcetype><D:getetag>"movies"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
  <D:response>
    <D:href>/dav/Movies/Remote Movie.mkv</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>12</D:getcontentlength><D:getetag>"remote-movie"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
  <D:response>
    <D:href>/dav/Movies/poster.jpg</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>5</D:getcontentlength><D:getetag>"poster"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
</D:multistatus>"#,
            )
                .into_response();
        }

        if path.ends_with("/Movies/Remote%20Movie.mkv")
            || path.ends_with("/Movies/Remote Movie.mkv")
        {
            return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/Movies/Remote Movie.mkv</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>12</D:getcontentlength><D:getetag>"remote-movie"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
</D:multistatus>"#,
            )
                .into_response();
        }

        if path.ends_with("/Movies/poster.jpg") {
            return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/Movies/poster.jpg</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>5</D:getcontentlength><D:getetag>"poster"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
</D:multistatus>"#,
            )
                .into_response();
        }

        StatusCode::NOT_FOUND.into_response()
    }

    struct PartialFailureWebDavServer {
        addr: std::net::SocketAddr,
    }

    impl PartialFailureWebDavServer {
        async fn start() -> Self {
            let router = Router::new().route("/{*path}", any(partial_failure_webdav_handler));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, router).await.unwrap();
            });

            Self { addr }
        }

        fn base_url(&self) -> String {
            format!("http://{}/dav", self.addr)
        }
    }

    async fn partial_failure_webdav_handler(
        method: axum::http::Method,
        uri: axum::http::Uri,
    ) -> Response {
        let path = uri.path();

        if method.as_str() != "PROPFIND" {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }

        if path.ends_with("/Movies/") || path.ends_with("/Movies") {
            return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/Movies/</D:href>
    <D:propstat><D:prop><D:resourcetype><D:collection/></D:resourcetype><D:getetag>"movies"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
  <D:response>
    <D:href>/dav/Movies/Good.mkv</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>4</D:getcontentlength><D:getetag>"good"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
  <D:response>
    <D:href>/dav/Movies/Broken/</D:href>
    <D:propstat><D:prop><D:resourcetype><D:collection/></D:resourcetype><D:getetag>"broken"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
</D:multistatus>"#,
            )
                .into_response();
        }

        if path.ends_with("/Movies/Good.mkv") {
            return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/Movies/Good.mkv</D:href>
    <D:propstat><D:prop><D:resourcetype/><D:getcontentlength>4</D:getcontentlength><D:getetag>"good"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat>
  </D:response>
</D:multistatus>"#,
            )
                .into_response();
        }

        if path.ends_with("/Movies/Broken/") || path.ends_with("/Movies/Broken") {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::NOT_FOUND.into_response()
    }
}
