use std::path::PathBuf;

use async_trait::async_trait;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use taru_core::{
    CanonicalMetadata, CatalogRepository, DirectorySnapshot, JobId, Library, LibraryId,
    LibraryRepository, MediaItem, MediaItemId, MediaProbeRepository, MediaRepository, MediaSource,
    MediaSourceId, PageRequest, Result, ScanRepository, ScanSnapshotId, ScanStatus, SourceState,
};
use taru_media_probe::{MediaProbe, MediaProbeRequest};
use taru_naming::{DefaultNameParser, NameParser, ParsedName};
use taru_search::{SearchDocument, SearchIndex};
use taru_vfs::{
    ByteRange, ObjectCacheState, ObjectKind, ObjectMetadata, StorageBackend, StorageUri,
};

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
    pub used_stale_cache: bool,
    pub media_sources: Vec<DiscoveredMediaSource>,
    pub directories: Vec<ScannedDirectory>,
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
    pub scan_id: ScanSnapshotId,
    pub scanned_roots: u64,
    pub discovered_files: u64,
    pub inserted_sources: u64,
    pub updated_sources: u64,
    pub tombstoned_sources: u64,
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
    pub staging_root: Option<PathBuf>,
}

impl Default for LibraryProbeOptions {
    fn default() -> Self {
        Self {
            max_concurrent_probes: 2,
            staging_root: None,
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
        let local_path_hint = match virtual_file.local_path_hint {
            Some(path) => Some(path),
            None => match &self.options.staging_root {
                Some(root) => match self
                    .backend
                    .stage(taru_vfs::StageRequest::new(uri.clone(), root.clone()))
                    .await
                {
                    Ok(staged) => Some(staged.path),
                    Err(err) => return probe_failure(source.locator, err),
                },
                None => None,
            },
        };
        let probe_result = match self
            .probe
            .probe(MediaProbeRequest {
                source: uri,
                local_path_hint,
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
    R: CatalogRepository + LibraryRepository + MediaRepository + ScanRepository + SearchIndex,
{
    pub async fn index_library(&self, request: LibraryIndexRequest) -> Result<LibraryIndexSummary> {
        self.repository.upsert_library(&request.library).await?;
        let scan_id = ScanSnapshotId::new();

        let mut summary = LibraryIndexSummary {
            job_id: request.job_id,
            library_id: request.library.id,
            scan_id,
            scanned_roots: 0,
            discovered_files: 0,
            inserted_sources: 0,
            updated_sources: 0,
            tombstoned_sources: 0,
        };

        let first_root = request
            .library
            .roots
            .first()
            .map(String::as_str)
            .unwrap_or("local:///");
        self.repository
            .begin_scan_snapshot(scan_id, request.library.id, first_root)
            .await?;

        let result = self.index_roots(&request, scan_id, &mut summary).await;

        match result {
            Ok(scan) => {
                if scan.complete {
                    self.mark_missing_sources_tombstoned(request.library.id, scan_id, &mut summary)
                        .await?;
                }
                self.repository
                    .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
                    .await?;
                Ok(summary)
            }
            Err(err) => {
                self.repository
                    .complete_scan_snapshot(scan_id, ScanStatus::Failed, Some(err.to_string()))
                    .await?;
                Err(err)
            }
        }
    }

    async fn index_roots(
        &self,
        request: &LibraryIndexRequest,
        scan_id: ScanSnapshotId,
        summary: &mut LibraryIndexSummary,
    ) -> Result<IndexRootsOutcome> {
        let mut complete = true;

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
            if scan.used_stale_cache {
                complete = false;
            }

            for directory in scan.directories {
                self.repository
                    .upsert_directory_snapshot(&DirectorySnapshot {
                        scan_id,
                        uri: directory.uri.as_str().to_owned(),
                        etag: directory.etag,
                        modified_at: directory.modified_at,
                        child_count: directory.child_count,
                    })
                    .await?;
            }

            for discovered in scan.media_sources {
                let locator = discovered.uri.as_str().to_owned();
                let existing = self
                    .repository
                    .get_media_source_by_locator(request.library.id, &locator)
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
                let state = source_state_from_discovered(
                    request.library.id,
                    source_id,
                    scan_id,
                    &discovered,
                );
                let source = media_source_from_discovered(
                    source_id,
                    request.library.id,
                    item_id,
                    discovered,
                );

                self.repository.upsert_media_item(&item).await?;
                self.repository.upsert_media_source(&source).await?;
                self.repository.upsert_source_state(&state).await?;
                self.rebuild_search_document(item, source).await?;

                if existing.is_some() {
                    summary.updated_sources += 1;
                } else {
                    summary.inserted_sources += 1;
                }
            }
        }

        Ok(IndexRootsOutcome { complete })
    }

    async fn rebuild_search_document(&self, item: MediaItem, source: MediaSource) -> Result<()> {
        let item_credits = self.repository.list_item_credits(item.id).await?;
        let item_genres = self.repository.list_item_genres(item.id).await?;
        let item_tags = self.repository.list_item_tags(item.id).await?;
        let item_studios = self.repository.list_item_studios(item.id).await?;
        let mut body_parts = Vec::new();
        let mut facets = vec![
            format!("kind:{}", item.kind.as_str()),
            format!("source:{}", source.file_name),
        ];

        if let Some(value) = &item.metadata.original_title {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.overview {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.tagline {
            body_parts.push(value.clone());
        }
        if let Some(value) = &item.metadata.release_date {
            facets.push(format!("release_date:{value}"));
        }

        for genre in item_genres {
            if let Some(genre) = self.repository.get_genre(genre.genre_id).await? {
                body_parts.push(genre.name.clone());
                facets.push(format!("genre:{}", genre.name));
            }
        }

        for tag in item_tags {
            if let Some(tag) = self.repository.get_tag(tag.tag_id).await? {
                body_parts.push(tag.name.clone());
                facets.push(format!("tag:{}", tag.name));
            }
        }

        for studio in item_studios {
            if let Some(studio) = self.repository.get_studio(studio.studio_id).await? {
                body_parts.push(studio.name.clone());
                facets.push(format!("studio:{}", studio.name));
            }
        }

        for credit in item_credits {
            if let Some(person) = self.repository.get_person(credit.person_id).await? {
                body_parts.push(person.name.clone());
                facets.push(format!("credit:{}", person.name));
            }
        }

        self.repository
            .upsert(SearchDocument {
                item_id: item.id,
                title: item.metadata.title,
                body: body_parts.join(" "),
                facets,
            })
            .await
    }

    async fn mark_missing_sources_tombstoned(
        &self,
        library_id: LibraryId,
        scan_id: ScanSnapshotId,
        summary: &mut LibraryIndexSummary,
    ) -> Result<()> {
        let mut offset = 0;

        loop {
            let states = self
                .repository
                .list_source_states(
                    library_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = states.len();

            for mut state in states {
                if state.last_seen_scan_id != scan_id && !state.tombstoned {
                    state.tombstoned = true;
                    self.repository.upsert_source_state(&state).await?;
                    summary.tombstoned_sources += 1;
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                break;
            }

            offset += u64::from(PageRequest::MAX_LIMIT);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IndexRootsOutcome {
    complete: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoveredMediaSource {
    pub uri: StorageUri,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub parsed_name: ParsedName,
    pub stale: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScannedDirectory {
    pub uri: StorageUri,
    pub etag: Option<String>,
    pub modified_at: Option<String>,
    pub child_count: u64,
    pub stale: bool,
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

fn source_state_from_discovered(
    library_id: LibraryId,
    source_id: MediaSourceId,
    scan_id: ScanSnapshotId,
    discovered: &DiscoveredMediaSource,
) -> SourceState {
    SourceState {
        library_id,
        source_id: Some(source_id),
        uri: discovered.uri.as_str().to_owned(),
        size_bytes: discovered.size_bytes,
        modified_at: discovered.modified_at.clone(),
        etag: discovered.etag.clone(),
        fingerprint: discovered.fingerprint.clone(),
        last_seen_scan_id: scan_id,
        tombstoned: false,
    }
}

fn media_source_from_discovered(
    id: MediaSourceId,
    library_id: LibraryId,
    item_id: MediaItemId,
    discovered: DiscoveredMediaSource,
) -> MediaSource {
    MediaSource {
        id,
        library_id,
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
        let mut directories = Vec::new();
        let mut stack = vec![(request.root.clone(), 0_usize)];

        while let Some((uri, depth)) = stack.pop() {
            if depth > self.options.max_depth {
                continue;
            }

            let metadata = self.backend.stat(&uri).await?;
            let metadata_stale = metadata_is_stale(&metadata);

            match metadata.kind {
                ObjectKind::Directory => {
                    let listing = self.backend.list_with_status(&uri).await?;
                    let listing_stale = listing_is_stale(&listing);
                    let mut entries = listing.entries;
                    directories.push(ScannedDirectory {
                        uri: metadata.uri,
                        etag: metadata.etag,
                        modified_at: metadata.modified_at,
                        child_count: entries.len() as u64,
                        stale: metadata_stale || listing_stale,
                    });
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
            used_stale_cache: used_stale_cache(&media_sources, &directories),
            media_sources,
            directories,
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
        let stale = metadata_is_stale(&metadata);
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
            modified_at: metadata.modified_at,
            etag: metadata.etag,
            fingerprint: metadata.fingerprint,
            parsed_name,
            stale,
        }
    }
}

fn used_stale_cache(
    media_sources: &[DiscoveredMediaSource],
    directories: &[ScannedDirectory],
) -> bool {
    directories.iter().any(|directory| directory.stale)
        || media_sources.iter().any(|source| source.stale)
}

fn metadata_is_stale(metadata: &ObjectMetadata) -> bool {
    metadata
        .cache
        .as_ref()
        .is_some_and(|cache| cache.state == ObjectCacheState::StaleFallback)
}

fn listing_is_stale(listing: &taru_vfs::ObjectListing) -> bool {
    listing
        .cache
        .as_ref()
        .is_some_and(|cache| cache.state == ObjectCacheState::StaleFallback)
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
    use taru_core::{
        LibraryOptions, LibraryPreset, MediaKind, MediaProbeRepository, MediaProbeResult,
        MediaRepository, MediaStreamInfo, MediaStreamKind, ScanRepository, TaruError,
        TransactionManager,
    };
    use taru_db::SqliteStore;
    use taru_search::{SearchIndex, SearchQuery};
    use taru_vfs::{
        CachedStorageBackend, LocalFsBackend, StorageCapabilities, VfsCacheOptions, WebDavBackend,
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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
        assert!(store.get_media_probe(source.id).await.unwrap().is_some());
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
        let hits = store
            .search(SearchQuery {
                query: "matrix".to_owned(),
                facets: Vec::new(),
                limit: 10,
                offset: 0,
            })
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
        assert_eq!(item.metadata.title, "The Matrix");
        assert_eq!(item.metadata.release_date, Some("1999".to_owned()));
        assert_eq!(hits[0].item_id, item.id);
    }

    #[tokio::test]
    async fn index_and_probe_keep_identical_local_locators_isolated_by_library() {
        let first_root = tempfile::tempdir().unwrap();
        let second_root = tempfile::tempdir().unwrap();
        fs::write(first_root.path().join("Movie.mkv"), b"first").unwrap();
        fs::write(second_root.path().join("Movie.mkv"), b"second").unwrap();

        let store = SqliteStore::connect_in_memory().await.unwrap();
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

        let first_sources = store
            .list_media_sources(first_library.id, PageRequest::first_page())
            .await
            .unwrap();
        let second_sources = store
            .list_media_sources(second_library.id, PageRequest::first_page())
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
            store
                .get_media_probe(first_sources[0].id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .get_media_probe(second_sources[0].id)
                .await
                .unwrap()
                .is_some()
        );

        let hits = store
            .search(SearchQuery {
                query: "movie".to_owned(),
                facets: Vec::new(),
                limit: 10,
                offset: 0,
            })
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
        let store = SqliteStore::connect_in_memory().await.unwrap();
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

    #[async_trait]
    impl StorageBackend for FlakyRemoteBackend {
        fn scheme(&self) -> &'static str {
            "remote"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Ok(Self::metadata(uri.clone()))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            if self.fail_list.load(Ordering::SeqCst) {
                return Err(TaruError::Storage {
                    uri: uri.to_string(),
                    message: "remote listing timed out".to_owned(),
                });
            }

            Ok(vec![Self::metadata(
                StorageUri::from_parts("remote", "Movies/Remote Movie.mkv").unwrap(),
            )])
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            range: Option<ByteRange>,
        ) -> Result<taru_vfs::VirtualFile> {
            Ok(taru_vfs::VirtualFile {
                uri: uri.clone(),
                range,
                local_path_hint: None,
            })
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(TaruError::Unsupported("flaky remote does not read text"))
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Err(TaruError::Unsupported("flaky remote does not write text"))
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
}
