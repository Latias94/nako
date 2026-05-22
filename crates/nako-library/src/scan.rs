use async_trait::async_trait;
use nako_core::{NakoError, Result};
use nako_vfs::{ObjectCacheState, ObjectKind, ObjectMetadata, StorageBackend, StorageUri};
use serde::{Deserialize, Serialize};

use super::{
    failure::{ingestion_failure_class, ingestion_failure_is_retryable},
    summary::{LibraryScanFailure, LibraryScanRequest, LibraryScanSummary},
};

#[async_trait]
pub trait LibraryScanner: Send + Sync {
    async fn scan(&self, request: LibraryScanRequest) -> Result<LibraryScanSummary>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiscoveredMediaSource {
    pub uri: StorageUri,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
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
}

impl<B> VfsLibraryScanner<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            options: LibraryScannerOptions::default(),
        }
    }

    pub fn with_options(backend: B, options: LibraryScannerOptions) -> Self {
        Self { backend, options }
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
        let mut failures = Vec::new();
        let mut stack = vec![(request.root.clone(), 0_usize)];

        while let Some((uri, depth)) = stack.pop() {
            if depth > self.options.max_depth {
                continue;
            }

            let metadata = match self.backend.stat(&uri).await {
                Ok(metadata) => metadata,
                Err(err) => {
                    failures.push(scan_failure(uri, "entry", err));
                    continue;
                }
            };
            let metadata_stale = metadata_is_stale(&metadata);

            match metadata.kind {
                ObjectKind::Directory => {
                    let listing = match self.backend.list_with_status(&uri).await {
                        Ok(listing) => listing,
                        Err(err) => {
                            failures.push(scan_failure(uri, "directory", err));
                            continue;
                        }
                    };
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
            failures,
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

        DiscoveredMediaSource {
            uri: metadata.uri,
            file_name,
            size_bytes: metadata.len,
            modified_at: metadata.modified_at,
            etag: metadata.etag,
            fingerprint: metadata.fingerprint,
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

fn listing_is_stale(listing: &nako_vfs::ObjectListing) -> bool {
    listing
        .cache
        .as_ref()
        .is_some_and(|cache| cache.state == ObjectCacheState::StaleFallback)
}

fn scan_failure(uri: StorageUri, target_kind: &str, err: NakoError) -> LibraryScanFailure {
    LibraryScanFailure {
        uri,
        target_kind: target_kind.to_owned(),
        failure_class: ingestion_failure_class(&err),
        message: err.to_string(),
        retryable: ingestion_failure_is_retryable(&err),
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
