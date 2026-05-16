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
    use std::fs;

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
    use taru_vfs::LocalFsBackend;

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
        assert_eq!(loaded.metadata.title, "File Title");
        assert_eq!(loaded.metadata.overview, Some("NFO overview".to_owned()));
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
