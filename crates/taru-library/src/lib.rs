use async_trait::async_trait;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use taru_core::{
    CanonicalMetadata, JobId, Library, LibraryId, LibraryRepository, MediaItem, MediaItemId,
    MediaProbeRepository, MediaRepository, MediaSource, MediaSourceId, PageRequest, Result,
};
use taru_media_probe::{MediaProbe, MediaProbeRequest};
use taru_naming::{DefaultNameParser, NameParser, ParsedName};
use taru_vfs::{ByteRange, ObjectKind, ObjectMetadata, StorageBackend, StorageUri};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanRequest {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub root: StorageUri,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryScanSummary {
    pub job_id: JobId,
    pub discovered_files: u64,
    pub changed_files: u64,
    pub removed_files: u64,
    pub media_sources: Vec<DiscoveredMediaSource>,
}

#[async_trait]
pub trait LibraryScanner: Send + Sync {
    async fn scan(&self, request: LibraryScanRequest) -> Result<LibraryScanSummary>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryIndexRequest {
    pub job_id: JobId,
    pub library: Library,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryIndexSummary {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub scanned_roots: u64,
    pub discovered_files: u64,
    pub inserted_sources: u64,
    pub updated_sources: u64,
}

#[derive(Debug)]
pub struct LibraryIndexService<S, R> {
    scanner: S,
    repository: R,
}

impl<S, R> LibraryIndexService<S, R> {
    pub fn new(scanner: S, repository: R) -> Self {
        Self {
            scanner,
            repository,
        }
    }

    #[must_use]
    pub fn scanner(&self) -> &S {
        &self.scanner
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeRequest {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeSummary {
    pub job_id: JobId,
    pub library_id: LibraryId,
    pub total_sources: u64,
    pub probed_sources: u64,
    pub skipped_sources: u64,
    pub failed_sources: u64,
    pub failures: Vec<LibraryProbeFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LibraryProbeFailure {
    pub locator: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryProbeOptions {
    pub max_concurrent_probes: usize,
}

impl Default for LibraryProbeOptions {
    fn default() -> Self {
        Self {
            max_concurrent_probes: 2,
        }
    }
}

#[derive(Debug)]
pub struct LibraryProbeService<B, P, R> {
    backend: B,
    probe: P,
    repository: R,
    options: LibraryProbeOptions,
}

impl<B, P, R> LibraryProbeService<B, P, R> {
    pub fn new(backend: B, probe: P, repository: R) -> Self {
        Self {
            backend,
            probe,
            repository,
            options: LibraryProbeOptions::default(),
        }
    }

    pub fn with_options(backend: B, probe: P, repository: R, options: LibraryProbeOptions) -> Self {
        Self {
            backend,
            probe,
            repository,
            options,
        }
    }

    #[must_use]
    pub fn options(&self) -> &LibraryProbeOptions {
        &self.options
    }
}

impl<B, P, R> LibraryProbeService<B, P, R>
where
    B: StorageBackend,
    P: MediaProbe,
    R: MediaRepository + MediaProbeRepository,
{
    pub async fn probe_library(&self, request: LibraryProbeRequest) -> Result<LibraryProbeSummary> {
        let sources = self.list_all_media_sources(request.library_id).await?;
        let total_sources = sources.len() as u64;
        let max_concurrent = self.options.max_concurrent_probes.max(1);
        let outcomes = stream::iter(sources)
            .map(|source| async move { self.probe_source(source, request.force).await })
            .buffer_unordered(max_concurrent)
            .collect::<Vec<_>>()
            .await;

        let mut summary = LibraryProbeSummary {
            job_id: request.job_id,
            library_id: request.library_id,
            total_sources,
            probed_sources: 0,
            skipped_sources: 0,
            failed_sources: 0,
            failures: Vec::new(),
        };

        for outcome in outcomes {
            match outcome {
                ProbeSourceOutcome::Probed => summary.probed_sources += 1,
                ProbeSourceOutcome::Skipped => summary.skipped_sources += 1,
                ProbeSourceOutcome::Failed(failure) => {
                    summary.failed_sources += 1;
                    summary.failures.push(failure);
                }
            }
        }

        summary
            .failures
            .sort_by(|left, right| left.locator.cmp(&right.locator));

        Ok(summary)
    }

    async fn list_all_media_sources(&self, library_id: LibraryId) -> Result<Vec<MediaSource>> {
        let mut offset = 0;
        let mut sources = Vec::new();

        loop {
            let page = self
                .repository
                .list_media_sources(
                    library_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = page.len();
            sources.extend(page);

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(sources)
    }

    async fn probe_source(&self, source: MediaSource, force: bool) -> ProbeSourceOutcome {
        if !force {
            match self.repository.get_media_probe(source.id).await {
                Ok(Some(_existing)) => return ProbeSourceOutcome::Skipped,
                Ok(None) => {}
                Err(err) => return probe_failure(source.locator, err),
            }
        }

        let uri = match StorageUri::parse(&source.locator) {
            Ok(uri) => uri,
            Err(err) => return probe_failure(source.locator, err),
        };
        let virtual_file = match self
            .backend
            .open_range(
                &uri,
                Some(ByteRange {
                    offset: 0,
                    length: None,
                }),
            )
            .await
        {
            Ok(virtual_file) => virtual_file,
            Err(err) => return probe_failure(source.locator, err),
        };
        let probe_result = match self
            .probe
            .probe(MediaProbeRequest {
                source: uri,
                local_path_hint: virtual_file.local_path_hint,
            })
            .await
        {
            Ok(result) => result,
            Err(err) => return probe_failure(source.locator, err),
        };

        match self
            .repository
            .upsert_media_probe(source.id, &probe_result)
            .await
        {
            Ok(()) => ProbeSourceOutcome::Probed,
            Err(err) => probe_failure(source.locator, err),
        }
    }
}

enum ProbeSourceOutcome {
    Probed,
    Skipped,
    Failed(LibraryProbeFailure),
}

fn probe_failure(locator: String, err: impl ToString) -> ProbeSourceOutcome {
    ProbeSourceOutcome::Failed(LibraryProbeFailure {
        locator,
        message: err.to_string(),
    })
}

impl<S, R> LibraryIndexService<S, R>
where
    S: LibraryScanner,
    R: LibraryRepository + MediaRepository,
{
    pub async fn index_library(&self, request: LibraryIndexRequest) -> Result<LibraryIndexSummary> {
        self.repository.upsert_library(&request.library).await?;

        let mut summary = LibraryIndexSummary {
            job_id: request.job_id,
            library_id: request.library.id,
            scanned_roots: 0,
            discovered_files: 0,
            inserted_sources: 0,
            updated_sources: 0,
        };

        for root in &request.library.roots {
            let root = StorageUri::parse(root)?;
            let scan = self
                .scanner
                .scan(LibraryScanRequest {
                    job_id: request.job_id,
                    library_id: request.library.id,
                    root,
                    force: request.force,
                })
                .await?;

            summary.scanned_roots += 1;
            summary.discovered_files += scan.discovered_files;

            for discovered in scan.media_sources {
                let locator = discovered.uri.as_str().to_owned();
                let existing = self
                    .repository
                    .get_media_source_by_locator(&locator)
                    .await?;

                let item_id = existing
                    .as_ref()
                    .map(|source| source.item_id)
                    .unwrap_or_else(MediaItemId::new);
                let source_id = existing
                    .as_ref()
                    .map(|source| source.id)
                    .unwrap_or_else(MediaSourceId::new);

                let item = media_item_from_discovered(item_id, &discovered);
                let source = media_source_from_discovered(source_id, item_id, discovered);

                self.repository.upsert_media_item(&item).await?;
                self.repository
                    .upsert_media_source(request.library.id, &source)
                    .await?;

                if existing.is_some() {
                    summary.updated_sources += 1;
                } else {
                    summary.inserted_sources += 1;
                }
            }
        }

        Ok(summary)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoveredMediaSource {
    pub uri: StorageUri,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub parsed_name: ParsedName,
}

fn media_item_from_discovered(id: MediaItemId, discovered: &DiscoveredMediaSource) -> MediaItem {
    MediaItem {
        id,
        kind: discovered.parsed_name.kind_hint,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: discovered.parsed_name.title.clone(),
            original_title: None,
            sort_title: None,
            overview: None,
            release_date: discovered.parsed_name.year.map(|year| year.to_string()),
            external_ids: Vec::new(),
            ..CanonicalMetadata::default()
        },
    }
}

fn media_source_from_discovered(
    id: MediaSourceId,
    item_id: MediaItemId,
    discovered: DiscoveredMediaSource,
) -> MediaSource {
    MediaSource {
        id,
        item_id,
        locator: discovered.uri.as_str().to_owned(),
        file_name: discovered.file_name,
        size_bytes: discovered.size_bytes,
        fingerprint: discovered.fingerprint,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryScannerOptions {
    pub media_extensions: Vec<String>,
    pub max_depth: usize,
}

impl Default for LibraryScannerOptions {
    fn default() -> Self {
        Self {
            media_extensions: DEFAULT_MEDIA_EXTENSIONS
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            max_depth: 32,
        }
    }
}

#[derive(Debug)]
pub struct VfsLibraryScanner<B> {
    backend: B,
    options: LibraryScannerOptions,
    name_parser: DefaultNameParser,
}

impl<B> VfsLibraryScanner<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            options: LibraryScannerOptions::default(),
            name_parser: DefaultNameParser,
        }
    }

    pub fn with_options(backend: B, options: LibraryScannerOptions) -> Self {
        Self {
            backend,
            options,
            name_parser: DefaultNameParser,
        }
    }

    #[must_use]
    pub fn options(&self) -> &LibraryScannerOptions {
        &self.options
    }
}

#[async_trait]
impl<B> LibraryScanner for VfsLibraryScanner<B>
where
    B: StorageBackend,
{
    async fn scan(&self, request: LibraryScanRequest) -> Result<LibraryScanSummary> {
        let mut media_sources = Vec::new();
        let mut stack = vec![(request.root.clone(), 0_usize)];

        while let Some((uri, depth)) = stack.pop() {
            if depth > self.options.max_depth {
                continue;
            }

            let metadata = self.backend.stat(&uri).await?;

            match metadata.kind {
                ObjectKind::Directory => {
                    let mut entries = self.backend.list(&uri).await?;
                    entries.sort_by(|left, right| right.uri.as_str().cmp(left.uri.as_str()));

                    for entry in entries {
                        stack.push((entry.uri, depth + 1));
                    }
                }
                ObjectKind::File | ObjectKind::Symlink if self.is_supported_media(&metadata) => {
                    media_sources.push(self.to_media_source(metadata));
                }
                ObjectKind::File | ObjectKind::Symlink | ObjectKind::Other => {}
            }
        }

        media_sources.sort_by(|left, right| left.uri.as_str().cmp(right.uri.as_str()));

        Ok(LibraryScanSummary {
            job_id: request.job_id,
            discovered_files: media_sources.len() as u64,
            changed_files: 0,
            removed_files: 0,
            media_sources,
        })
    }
}

impl<B> VfsLibraryScanner<B> {
    fn is_supported_media(&self, metadata: &ObjectMetadata) -> bool {
        extension(metadata.uri.as_str()).is_some_and(|extension| {
            self.options
                .media_extensions
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(extension))
        })
    }
    fn to_media_source(&self, metadata: ObjectMetadata) -> DiscoveredMediaSource {
        let file_name = metadata
            .uri
            .path_part()
            .rsplit_once('/')
            .map(|(_parent, file_name)| file_name)
            .unwrap_or_else(|| metadata.uri.path_part())
            .to_owned();
        let parsed_name = self.name_parser.parse_path(metadata.uri.path_part());

        DiscoveredMediaSource {
            uri: metadata.uri,
            file_name,
            size_bytes: metadata.len,
            fingerprint: metadata.fingerprint,
            parsed_name,
        }
    }
}

fn extension(path: &str) -> Option<&str> {
    let file_name = path.rsplit('/').next()?;
    let (_stem, extension) = file_name.rsplit_once('.')?;

    if extension.is_empty() {
        None
    } else {
        Some(extension)
    }
}

const DEFAULT_MEDIA_EXTENSIONS: &[&str] = &[
    "3gp", "avi", "flv", "m2ts", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "mts", "ts", "webm",
    "wmv",
];

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use taru_core::{
        LibraryOptions, LibraryPreset, MediaProbeRepository, MediaProbeResult, MediaRepository,
        MediaStreamInfo, MediaStreamKind, TaruError, TransactionManager,
    };
    use taru_db::SqliteStore;
    use taru_vfs::LocalFsBackend;
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
            assert_eq!(summary.media_sources[0].parsed_name.title, "demo");
            assert_eq!(
                summary.media_sources[0].uri.as_str(),
                "local:///Movies/Demo Movie/demo.MKV"
            );
        });
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
        let sources = store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap();
        let item = store
            .get_media_item(sources[0].item_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(first_summary.discovered_files, 1);
        assert_eq!(first_summary.inserted_sources, 1);
        assert_eq!(first_summary.updated_sources, 0);
        assert_eq!(second_summary.discovered_files, 1);
        assert_eq!(second_summary.inserted_sources, 0);
        assert_eq!(second_summary.updated_sources, 1);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].file_name, "The Matrix (1999).mkv");
        assert_eq!(item.metadata.title, "The Matrix");
        assert_eq!(item.metadata.release_date, Some("1999".to_owned()));
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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

        for source in store
            .list_media_sources(library.id, PageRequest::first_page())
            .await
            .unwrap()
        {
            assert!(store.get_media_probe(source.id).await.unwrap().is_some());
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
    }

    #[derive(Clone, Default)]
    struct RecordingProbe {
        active: Arc<AtomicUsize>,
        max_seen: Arc<AtomicUsize>,
        fail_locator_fragment: Option<String>,
    }

    #[async_trait::async_trait]
    impl MediaProbe for RecordingProbe {
        async fn probe(&self, request: MediaProbeRequest) -> Result<MediaProbeResult> {
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            update_max(&self.max_seen, active);

            sleep(Duration::from_millis(25)).await;

            self.active.fetch_sub(1, Ordering::SeqCst);

            if self
                .fail_locator_fragment
                .as_ref()
                .is_some_and(|fragment| request.source.as_str().contains(fragment))
            {
                return Err(TaruError::Provider {
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
                }],
            })
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
}
