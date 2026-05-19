mod codec;
mod export;
mod import;
mod summary;
mod workflow;

pub use codec::*;
pub use summary::*;

#[derive(Debug)]
pub struct NfoService<B, R, C> {
    pub(crate) backend: B,
    pub(crate) repository: R,
    pub(crate) codec: C,
}
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs,
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use taru_core::{
        CanonicalMetadata, Credit, CreditRole, ImageKind, JobId, Library, LibraryId,
        LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryPreset,
        LocalMetadataPolicy, MediaItem, MediaItemId, MediaKind, MediaRepository, MediaSource,
        MediaSourceId, MetadataField, MetadataFieldLock, MetadataSource, PageRequest, TaruError,
        TransactionManager,
        repository::{CatalogRepository, LibraryRepository, MetadataRepository},
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchIndex, SearchQuery};
    use taru_vfs::{
        ByteRange, LocalFsBackend, ObjectKind, ObjectMetadata, StorageBackend, StorageBackupMode,
        StorageBackupReport, StorageCapabilities, StorageUri, StorageWriteMode, StorageWriteReport,
        StorageWriteRequest, VirtualFile,
    };

    use super::*;

    #[test]
    fn movie_nfo_round_trips_core_fields() {
        let document = NfoDocument {
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                original_title: Some("The Matrix".to_owned()),
                sort_title: Some("Matrix, The".to_owned()),
                overview: Some("A hacker discovers reality.".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                runtime_minutes: Some(136),
                tagline: Some("Welcome to the Real World".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                tags: vec!["cyberpunk".to_owned()],
                credits: vec![Credit {
                    name: "Keanu Reeves".to_owned(),
                    role: CreditRole::Actor,
                    character: Some("Neo".to_owned()),
                    order: Some(0),
                    external_ids: Vec::new(),
                }],
                ..CanonicalMetadata::default()
            },
            external_ids: Vec::new(),
            hierarchy: NfoHierarchy::default(),
        };
        let codec = MovieNfoCodec;

        let xml = codec.render(&document).unwrap();
        let parsed = codec.parse(&xml).unwrap();

        assert_eq!(parsed.metadata.title, "The Matrix");
        assert_eq!(parsed.metadata.sort_title, Some("Matrix, The".to_owned()));
        assert_eq!(parsed.metadata.runtime_minutes, Some(136));
        assert_eq!(
            parsed.metadata.genres,
            vec!["Action".to_owned(), "Science Fiction".to_owned()]
        );
        assert_eq!(parsed.metadata.tags, vec!["cyberpunk".to_owned()]);
        assert_eq!(parsed.metadata.credits[0].name, "Keanu Reeves");
    }

    #[test]
    fn movie_nfo_parser_handles_attributes_entities_and_nested_fanart() {
        let codec = MovieNfoCodec;
        let parsed = codec
            .parse(
                r#"<movie>
  <title sort="ignored">Tom &amp; Jerry</title>
  <plot>Uses &lt;escaped&gt; text</plot>
  <fanart>
    <thumb>local:///fanart-a.jpg</thumb>
    <thumb>local:///fanart-b.jpg</thumb>
  </fanart>
  <actor>
    <name>Lead Actor</name>
    <role>Hero &amp; Guide</role>
  </actor>
</movie>"#,
            )
            .unwrap();

        assert_eq!(parsed.metadata.title, "Tom & Jerry");
        assert_eq!(
            parsed.metadata.overview,
            Some("Uses <escaped> text".to_owned())
        );
        assert_eq!(parsed.metadata.images.len(), 2);
        assert_eq!(parsed.metadata.images[0].kind, ImageKind::Backdrop);
        assert_eq!(parsed.metadata.images[0].uri, "local:///fanart-a.jpg");
        assert_eq!(
            parsed.metadata.credits[0].character.as_deref(),
            Some("Hero & Guide")
        );
    }

    #[test]
    fn movie_nfo_preservation_keeps_unknown_fields_and_updates_owned_fields() {
        let codec = MovieNfoCodec;
        let document = NfoDocument {
            metadata: CanonicalMetadata {
                title: "Canonical Title".to_owned(),
                overview: Some("Canonical overview".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                genres: vec!["Action".to_owned()],
                ..CanonicalMetadata::default()
            },
            external_ids: Vec::new(),
            hierarchy: NfoHierarchy::default(),
        };

        let rendered = codec
            .render_preserving(
                &document,
                r#"<movie>
  <title>Old Title</title>
  <plot>Old overview</plot>
  <customrating system="local">five stars</customrating>
  <!-- keep hand-authored notes -->
  <uniqueid type="imdb">tt0133093</uniqueid>
</movie>"#,
            )
            .unwrap();

        assert!(rendered.xml.contains("<title>Canonical Title</title>"));
        assert!(rendered.xml.contains("<plot>Canonical overview</plot>"));
        assert!(rendered.xml.contains("<genre>Action</genre>"));
        assert!(!rendered.xml.contains("<title>Old Title</title>"));
        assert!(!rendered.xml.contains("<plot>Old overview</plot>"));
        assert!(
            rendered
                .xml
                .contains(r#"<customrating system="local">five stars</customrating>"#)
        );
        assert!(rendered.xml.contains("<!-- keep hand-authored notes -->"));
        assert!(
            rendered
                .xml
                .contains(r#"<uniqueid type="imdb">tt0133093</uniqueid>"#)
        );
        assert_eq!(
            rendered.report.preserved_unknown_fields,
            vec![
                "customrating".to_owned(),
                "#comment".to_owned(),
                "uniqueid".to_owned()
            ]
        );
        assert!(
            rendered
                .report
                .updated_owned_fields
                .contains(&"title".to_owned())
        );
        assert!(
            rendered
                .report
                .updated_owned_fields
                .contains(&"plot".to_owned())
        );
    }

    #[test]
    fn movie_nfo_preservation_reports_duplicate_owned_and_alias_fields() {
        let codec = MovieNfoCodec;
        let document = NfoDocument {
            metadata: CanonicalMetadata {
                title: "Canonical Title".to_owned(),
                release_date: Some("1999-03-31".to_owned()),
                ..CanonicalMetadata::default()
            },
            external_ids: Vec::new(),
            hierarchy: NfoHierarchy::default(),
        };

        let rendered = codec
            .render_preserving(
                &document,
                r#"<movie>
  <title>Old Title</title>
  <title>Duplicate Title</title>
  <year>1999</year>
</movie>"#,
            )
            .unwrap();

        assert_eq!(rendered.report.conflicts.len(), 2);
        assert!(rendered.report.conflicts.iter().any(|conflict| {
            conflict.field == "title"
                && conflict.existing_value.as_deref() == Some("Duplicate Title")
                && conflict.replacement_value.as_deref() == Some("Canonical Title")
                && conflict.reason == NfoFieldConflictReason::DuplicateOwnedField
        }));
        assert!(rendered.report.conflicts.iter().any(|conflict| {
            conflict.field == "release_date"
                && conflict.existing_value.as_deref() == Some("1999")
                && conflict.replacement_value.as_deref() == Some("1999-03-31")
                && conflict.reason == NfoFieldConflictReason::OwnedFieldAlias
        }));
        assert!(rendered.xml.contains("<title>Canonical Title</title>"));
        assert!(
            rendered
                .xml
                .contains("<releasedate>1999-03-31</releasedate>")
        );
        assert!(!rendered.xml.contains("Duplicate Title"));
        assert!(!rendered.xml.contains("<year>1999</year>"));
    }

    #[tokio::test]
    async fn nfo_service_discovers_and_imports_movie_sidecar_with_locks() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies").join("Demo")).unwrap();
        fs::write(
            temp.path().join("Movies").join("Demo").join("demo.mkv"),
            b"media",
        )
        .unwrap();
        fs::write(
            temp.path().join("Movies").join("Demo").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
  <releasedate>1999-03-31</releasedate>
  <runtime>136</runtime>
  <genre>Action</genre>
  <tag>cyberpunk</tag>
  <actor>
    <name>Demo Actor</name>
    <role>Lead</role>
    <order>0</order>
  </actor>
  <director>Demo Director</director>
  <poster>local:///Movies/Demo/poster.jpg</poster>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/Demo/demo.mkv").await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let sidecars = service.discover_sidecars(library_id).await.unwrap();
        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        let people = store.list_people(PageRequest::first_page()).await.unwrap();
        let tags = store.list_tags(PageRequest::first_page()).await.unwrap();
        let images = store.list_item_images(item.id).await.unwrap();
        let hits = store
            .search(SearchQuery {
                query: "Demo Actor".to_owned(),
                facets: vec!["tag:cyberpunk".to_owned()],
                limit: 10,
                offset: 0,
            })
            .await
            .unwrap();

        assert_eq!(sidecars.len(), 1);
        assert_eq!(
            sidecars[0].nfo_uri.as_str(),
            "local:///Movies/Demo/demo.nfo"
        );
        assert_eq!(summary.scanned_sources, 1);
        assert_eq!(summary.discovered_nfo, 1);
        assert_eq!(summary.imported_items, 1);
        assert_eq!(loaded.metadata.title, "NFO Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert_eq!(loaded.metadata.tags, vec!["cyberpunk"]);
        assert!(people.iter().any(|person| person.name == "Demo Actor"));
        assert_eq!(tags[0].name, "cyberpunk");
        assert_eq!(images[0].source_uri, "local:///Movies/Demo/poster.jpg");
        assert_eq!(hits[0].item_id, item.id);
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
        }));
    }

    #[tokio::test]
    async fn nfo_service_remote_first_import_only_fills_missing_fields_without_locks() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
  <genre>Action</genre>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Remote Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::RemoteFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();

        assert_eq!(summary.imported_items, 1);
        assert_eq!(loaded.metadata.title, "Remote Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.is_empty());
    }

    #[tokio::test]
    async fn nfo_service_preserves_user_locked_fields_during_import() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/demo.mkv").await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::User,
            })
            .await
            .unwrap();
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        assert_eq!(loaded.metadata.title, "File Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::User
        }));

        service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();
        assert_eq!(loaded.metadata.title, "File Title");
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::User
        }));
    }

    #[tokio::test]
    async fn nfo_service_allows_nfo_authority_to_refresh_nfo_locked_fields() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>NFO Title</title>
  <plot>NFO overview</plot>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/demo.mkv").await;
        store
            .upsert_field_lock(&MetadataFieldLock {
                item_id: item.id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::Nfo,
            })
            .await
            .unwrap();
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        let loaded = store.get_media_item(item.id).await.unwrap().unwrap();
        let locks = store.list_field_locks(item.id).await.unwrap();

        assert_eq!(loaded.metadata.title, "NFO Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
        assert!(locks.iter().any(|lock| {
            lock.field == MetadataField::Title && lock.locked && lock.source == MetadataSource::Nfo
        }));
    }

    #[tokio::test]
    async fn nfo_service_confirms_provisional_episode_hierarchy_in_place() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("TV").join("Firefly")).unwrap();
        fs::write(
            temp.path()
                .join("TV")
                .join("Firefly")
                .join("Firefly.S01E02.mkv"),
            b"media",
        )
        .unwrap();
        fs::write(
            temp.path()
                .join("TV")
                .join("Firefly")
                .join("Firefly.S01E02.nfo"),
            r#"<episodedetails>
  <title>The Train Job</title>
  <showtitle>Firefly</showtitle>
  <season>1</season>
  <episode>2</episode>
  <aired>2002-09-20</aired>
  <plot>The crew takes a train heist job.</plot>
</episodedetails>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
                title: "Season 01".to_owned(),
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
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: episode.id,
            locator: "local:///TV/Firefly/Firefly.S01E02.mkv".to_owned(),
            file_name: "Firefly.S01E02.mkv".to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
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
        store.upsert_media_source(&source).await.unwrap();

        let service = NfoService::new(
            LocalFsBackend::new(temp.path()).unwrap(),
            store.clone(),
            MovieNfoCodec,
        );
        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id: library.id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();
        let loaded_series = store.get_media_item(series.id).await.unwrap().unwrap();
        let loaded_season = store.get_media_item(season.id).await.unwrap().unwrap();
        let loaded_episode = store.get_media_item(episode.id).await.unwrap().unwrap();

        assert_eq!(summary.discovered_nfo, 1);
        assert_eq!(summary.imported_items, 1);
        assert_eq!(loaded_series.id, series.id);
        assert_eq!(loaded_series.metadata.title, "Firefly");
        assert_eq!(loaded_season.id, season.id);
        assert_eq!(loaded_season.metadata.title, "Season 1");
        assert_eq!(loaded_episode.id, episode.id);
        assert_eq!(loaded_episode.parent_id, Some(season.id));
        assert_eq!(loaded_episode.metadata.title, "The Train Job");
        assert_eq!(
            loaded_episode.metadata.overview,
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
    }

    #[tokio::test]
    async fn nfo_service_exports_movie_sidecar_when_policy_allows_writing() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Exported Title".to_owned(),
                overview: Some("Exported overview".to_owned()),
                genres: vec!["Action".to_owned()],
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: false,
            })
            .await
            .unwrap();

        let xml = fs::read_to_string(temp.path().join("Movies").join("demo.nfo")).unwrap();
        assert_eq!(summary.exported_items, 1);
        assert!(xml.contains("<title>Exported Title</title>"));
        assert!(xml.contains("<genre>Action</genre>"));
    }

    #[tokio::test]
    async fn nfo_service_exports_movie_sidecar_with_atomic_replace_request() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Atomic Export Title".to_owned(),
                overview: Some("Atomic export overview".to_owned()),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = RecordingStorageBackend::default();
        let recorder = backend.clone();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: false,
            })
            .await
            .unwrap();

        let writes = recorder.writes();
        assert_eq!(summary.exported_items, 1);
        assert_eq!(summary.failed_items, 0);
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].mode, StorageWriteMode::AtomicReplace);
        assert_eq!(
            writes[0].uri,
            StorageUri::parse("local:///Movies/demo.nfo").unwrap()
        );
        assert!(
            writes[0]
                .content
                .contains("<title>Atomic Export Title</title>")
        );
    }

    #[tokio::test]
    async fn nfo_service_reports_storage_unsupported_when_atomic_write_is_unavailable() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Unsupported Atomic Export".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = RecordingStorageBackend::default().fail_atomic_replace();
        let recorder = backend.clone();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.exported_items, 0);
        assert_eq!(summary.failed_items, 1);
        assert_eq!(summary.failures[0].kind, NfoFailureKind::StorageUnsupported);
        assert_eq!(recorder.writes()[0].mode, StorageWriteMode::AtomicReplace);
    }

    #[tokio::test]
    async fn nfo_service_reports_backup_failure_before_replacing_sidecar() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Replacement Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let nfo_uri = StorageUri::parse("local:///Movies/demo.nfo").unwrap();
        let backend = RecordingStorageBackend::default()
            .with_file(nfo_uri.clone(), "<movie><title>Old Title</title></movie>")
            .fail_backup();
        let recorder = backend.clone();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        assert_eq!(summary.exported_items, 0);
        assert_eq!(summary.failed_items, 1);
        assert_eq!(summary.failures[0].kind, NfoFailureKind::StorageBackup);
        assert_eq!(
            recorder.read_to_string(&nfo_uri).await.unwrap(),
            "<movie><title>Old Title</title></movie>"
        );
        assert_eq!(
            recorder.writes()[0].backup.mode,
            StorageBackupMode::ExistingFile
        );
    }

    #[tokio::test]
    async fn nfo_service_reports_preservation_failure_when_forced_sidecar_is_invalid() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Forced Export Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = RecordingStorageBackend::default().with_file(
            StorageUri::parse("local:///Movies/demo.nfo").unwrap(),
            "<movie><title>broken",
        );
        let recorder = backend.clone();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        assert_eq!(summary.exported_items, 0);
        assert_eq!(summary.failed_items, 1);
        assert_eq!(summary.failures[0].kind, NfoFailureKind::NfoPreservation);
        assert!(summary.failures[0].message.contains("invalid NFO XML"));
        assert!(recorder.writes().is_empty());
    }

    #[tokio::test]
    async fn nfo_service_reports_parse_failure_when_import_sidecar_is_invalid() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item(&store, library_id, "local:///Movies/demo.mkv").await;
        let backend = RecordingStorageBackend::default().with_file(
            StorageUri::parse("local:///Movies/demo.nfo").unwrap(),
            "<movie><title>broken",
        );
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();

        assert_eq!(summary.imported_items, 0);
        assert_eq!(summary.failed_items, 1);
        assert_eq!(summary.failures[0].kind, NfoFailureKind::NfoParse);
    }

    #[tokio::test]
    async fn nfo_service_preserves_existing_sidecar_unknown_fields_when_forced() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>Old Sidecar Title</title>
  <plot>Old sidecar overview</plot>
  <customrating system="local">five stars</customrating>
  <uniqueid type="imdb">tt0133093</uniqueid>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Forced Export Title".to_owned(),
                overview: Some("Forced export overview".to_owned()),
                genres: vec!["Action".to_owned()],
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        let xml = fs::read_to_string(temp.path().join("Movies").join("demo.nfo")).unwrap();
        assert_eq!(summary.exported_items, 1);
        assert_eq!(summary.backed_up_items, 1);
        assert_eq!(summary.backups.len(), 1);
        assert_eq!(
            summary.backups[0].original_uri,
            StorageUri::parse("local:///Movies/demo.nfo").unwrap()
        );
        assert!(xml.contains("<title>Forced Export Title</title>"));
        assert!(xml.contains("<plot>Forced export overview</plot>"));
        assert!(xml.contains("<genre>Action</genre>"));
        assert!(!xml.contains("<title>Old Sidecar Title</title>"));
        assert!(!xml.contains("<plot>Old sidecar overview</plot>"));
        assert!(xml.contains(r#"<customrating system="local">five stars</customrating>"#));
        assert!(xml.contains(r#"<uniqueid type="imdb">tt0133093</uniqueid>"#));
        let backup_xml = fs::read_to_string(
            temp.path().join(
                summary.backups[0]
                    .backup_uri
                    .path_part()
                    .trim_start_matches('/'),
            ),
        )
        .unwrap();
        assert!(backup_xml.contains("<title>Old Sidecar Title</title>"));
    }

    #[tokio::test]
    async fn nfo_service_forced_export_reports_backup_retention_pruning() {
        let temp = tempfile::tempdir().unwrap();
        let movies = temp.path().join("Movies");
        fs::create_dir_all(&movies).unwrap();
        fs::write(movies.join("demo.mkv"), b"media").unwrap();
        fs::write(
            movies.join("demo.nfo"),
            r#"<movie><title>Old Sidecar Title</title></movie>"#,
        )
        .unwrap();
        for index in 0..5 {
            fs::write(
                movies.join(format!("demo.nfo.taru-backup-000{index}")),
                format!("old backup {index}"),
            )
            .unwrap();
        }
        fs::write(movies.join("other.nfo.taru-backup-0000"), "other").unwrap();
        fs::write(movies.join("demo.nfo.manual-backup"), "manual").unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Replacement Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        assert_eq!(summary.exported_items, 1);
        assert_eq!(summary.backed_up_items, 1);
        assert_eq!(summary.pruned_backup_items, 1);
        assert_eq!(summary.pruned_backups, 1);
        assert_eq!(summary.prune_failures, Vec::new());
        assert_eq!(summary.backups[0].pruned_backups.len(), 1);
        assert_eq!(
            summary.backups[0].pruned_backups[0].as_str(),
            "local:///Movies/demo.nfo.taru-backup-0000"
        );
        assert!(!movies.join("demo.nfo.taru-backup-0000").exists());
        assert!(movies.join("demo.nfo.taru-backup-0001").exists());
        assert!(movies.join("other.nfo.taru-backup-0000").exists());
        assert!(movies.join("demo.nfo.manual-backup").exists());
    }

    #[tokio::test]
    async fn nfo_service_does_not_backup_fresh_sidecar_export() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        seed_item_with_metadata(
            &store,
            library_id,
            "local:///Movies/demo.mkv",
            CanonicalMetadata {
                title: "Fresh Export Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        let summary = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        assert_eq!(summary.exported_items, 1);
        assert_eq!(summary.backed_up_items, 0);
        assert!(summary.backups.is_empty());
        assert!(
            fs::read_to_string(temp.path().join("Movies").join("demo.nfo"))
                .unwrap()
                .contains("<title>Fresh Export Title</title>")
        );
    }

    #[tokio::test]
    async fn nfo_service_import_then_forced_export_preserves_unknown_sidecar_fields() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Movies")).unwrap();
        fs::write(temp.path().join("Movies").join("demo.mkv"), b"media").unwrap();
        fs::write(
            temp.path().join("Movies").join("demo.nfo"),
            r#"<movie>
  <title>Imported Title</title>
  <plot>Imported overview</plot>
  <customrating system="local">five stars</customrating>
  <uniqueid type="imdb">tt0133093</uniqueid>
</movie>
"#,
        )
        .unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let library_id = LibraryId::new();
        let item = seed_item(&store, library_id, "local:///Movies/demo.mkv").await;
        let backend = LocalFsBackend::new(temp.path()).unwrap();
        let service = NfoService::new(backend, store.clone(), MovieNfoCodec);

        service
            .import_library(NfoImportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap();
        let imported = store.get_media_item(item.id).await.unwrap().unwrap();
        assert_eq!(imported.metadata.title, "Imported Title");
        store
            .upsert_media_item(&MediaItem {
                metadata: CanonicalMetadata {
                    title: "Post Import Title".to_owned(),
                    overview: Some("Post import overview".to_owned()),
                    genres: vec!["Action".to_owned()],
                    ..imported.metadata
                },
                ..imported
            })
            .await
            .unwrap();

        service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id,
                policy: LocalMetadataPolicy::WriteSidecar,
                force: true,
            })
            .await
            .unwrap();

        let xml = fs::read_to_string(temp.path().join("Movies").join("demo.nfo")).unwrap();
        assert!(xml.contains("<title>Post Import Title</title>"));
        assert!(xml.contains("<plot>Post import overview</plot>"));
        assert!(xml.contains("<genre>Action</genre>"));
        assert!(xml.contains(r#"<customrating system="local">five stars</customrating>"#));
        assert!(xml.contains(r#"<uniqueid type="imdb">tt0133093</uniqueid>"#));
    }

    #[tokio::test]
    async fn nfo_service_rejects_export_without_write_sidecar_policy() {
        let temp = tempfile::tempdir().unwrap();
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        let service = NfoService::new(
            LocalFsBackend::new(temp.path()).unwrap(),
            store,
            MovieNfoCodec,
        );

        let err = service
            .export_library(NfoExportRequest {
                job_id: JobId::new(),
                library_id: LibraryId::new(),
                policy: LocalMetadataPolicy::LocalFirst,
                force: false,
            })
            .await
            .unwrap_err();

        assert_eq!(
            err,
            TaruError::Unsupported("NFO export requires write-sidecar local metadata policy")
        );
    }

    #[derive(Clone, Default)]
    struct RecordingStorageBackend {
        files: Arc<Mutex<HashMap<StorageUri, String>>>,
        writes: Arc<Mutex<Vec<StorageWriteRequest>>>,
        fail_atomic_replace: bool,
        fail_backup: bool,
    }

    impl RecordingStorageBackend {
        fn with_file(self, uri: StorageUri, content: impl Into<String>) -> Self {
            {
                let mut files = self.files.lock().unwrap();
                files.insert(uri, content.into());
            }
            self
        }

        fn fail_atomic_replace(mut self) -> Self {
            self.fail_atomic_replace = true;
            self
        }

        fn fail_backup(mut self) -> Self {
            self.fail_backup = true;
            self
        }

        fn writes(&self) -> Vec<StorageWriteRequest> {
            self.writes.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl StorageBackend for RecordingStorageBackend {
        fn scheme(&self) -> &'static str {
            "local"
        }

        async fn stat(&self, uri: &StorageUri) -> taru_core::Result<ObjectMetadata> {
            let files = self.files.lock().unwrap();
            let Some(content) = files.get(uri) else {
                return Err(TaruError::NotFound {
                    entity: "storage_object",
                    id: uri.to_string(),
                });
            };

            Ok(ObjectMetadata {
                uri: uri.clone(),
                kind: ObjectKind::File,
                len: Some(content.len() as u64),
                modified_at: None,
                etag: None,
                fingerprint: None,
                capabilities: StorageCapabilities::WRITABLE,
                cache: None,
            })
        }

        async fn list(&self, _uri: &StorageUri) -> taru_core::Result<Vec<ObjectMetadata>> {
            Ok(Vec::new())
        }

        async fn open_range(
            &self,
            _uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> taru_core::Result<VirtualFile> {
            Err(TaruError::Unsupported(
                "recording storage backend does not support opening files",
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> taru_core::Result<String> {
            self.files
                .lock()
                .unwrap()
                .get(uri)
                .cloned()
                .ok_or_else(|| TaruError::NotFound {
                    entity: "storage_object",
                    id: uri.to_string(),
                })
        }

        async fn write_string(&self, uri: &StorageUri, content: &str) -> taru_core::Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(uri.clone(), content.to_owned());
            Ok(())
        }

        async fn write(
            &self,
            request: StorageWriteRequest,
        ) -> taru_core::Result<StorageWriteReport> {
            self.writes.lock().unwrap().push(request.clone());
            if self.fail_atomic_replace && request.mode == StorageWriteMode::AtomicReplace {
                return Err(TaruError::Unsupported(
                    "recording storage backend does not support atomic replace writes",
                ));
            }
            if self.fail_backup && request.backup.mode == StorageBackupMode::ExistingFile {
                return Err(TaruError::storage_backup(
                    request.uri.to_string(),
                    "recording storage backend failed to create backup",
                ));
            }

            let backup = if request.backup.mode == StorageBackupMode::ExistingFile
                && self.files.lock().unwrap().contains_key(&request.uri)
            {
                Some(StorageBackupReport {
                    original_uri: request.uri.clone(),
                    backup_uri: StorageUri::parse(format!("{}.taru-backup-test", request.uri))
                        .unwrap(),
                    pruned_backups: Vec::new(),
                    prune_failures: Vec::new(),
                })
            } else {
                None
            };

            self.files
                .lock()
                .unwrap()
                .insert(request.uri.clone(), request.content);
            Ok(StorageWriteReport {
                uri: request.uri,
                mode: request.mode,
                atomic: request.mode == StorageWriteMode::AtomicReplace,
                backup,
            })
        }
    }

    async fn seed_item(store: &SqliteStore, library_id: LibraryId, locator: &str) -> MediaItem {
        seed_item_with_metadata(
            store,
            library_id,
            locator,
            CanonicalMetadata {
                title: "File Title".to_owned(),
                ..CanonicalMetadata::default()
            },
        )
        .await
    }

    async fn seed_item_with_metadata(
        store: &SqliteStore,
        library_id: LibraryId,
        locator: &str,
        metadata: CanonicalMetadata,
    ) -> MediaItem {
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata,
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id: item.id,
            locator: locator.to_owned(),
            file_name: locator.rsplit('/').next().unwrap_or(locator).to_owned(),
            size_bytes: Some(1),
            fingerprint: None,
        };
        let library = Library {
            id: library_id,
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();
        item
    }
}
