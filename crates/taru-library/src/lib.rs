use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{JobId, LibraryId, Result};
use taru_naming::{DefaultNameParser, NameParser, ParsedName};
use taru_vfs::{ObjectKind, ObjectMetadata, StorageBackend, StorageUri};

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
pub struct DiscoveredMediaSource {
    pub uri: StorageUri,
    pub file_name: String,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub parsed_name: ParsedName,
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
    use std::fs;

    use taru_vfs::LocalFsBackend;

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
}
