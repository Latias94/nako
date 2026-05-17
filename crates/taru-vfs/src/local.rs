use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use async_trait::async_trait;
use taru_core::{Result, StorageErrorKind, TaruError};

use crate::{
    ByteRange, ObjectKind, ObjectMetadata, ReadRange, StageRequest, StagedFile, StorageBackend,
    StorageCapabilities, StorageUri, VirtualFile,
};

#[derive(Clone, Debug)]
pub struct LocalFsBackend {
    root: PathBuf,
}

impl LocalFsBackend {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let root = root.canonicalize().map_err(|err| {
            TaruError::storage_io(
                root.display().to_string(),
                format!("failed to canonicalize local root: {err}"),
            )
        })?;

        if !root.is_dir() {
            return Err(TaruError::InvalidInput {
                message: format!("local root must be a directory: {}", root.display()),
            });
        }

        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, uri: &StorageUri) -> Result<PathBuf> {
        self.ensure_local_scheme(uri)?;

        let relative = relative_path(uri)?;
        let candidate = self.root.join(relative);
        let canonical = candidate.canonicalize().map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                TaruError::NotFound {
                    entity: "storage_object",
                    id: uri.to_string(),
                }
            } else {
                TaruError::storage_io(
                    uri.to_string(),
                    format!("failed to resolve local path: {err}"),
                )
            }
        })?;

        if !canonical.starts_with(&self.root) {
            return Err(TaruError::storage(
                uri.to_string(),
                StorageErrorKind::SecurityViolation,
                "resolved local path escaped backend root",
            ));
        }

        Ok(candidate)
    }

    fn writable_path_for(&self, uri: &StorageUri) -> Result<PathBuf> {
        self.ensure_local_scheme(uri)?;

        let relative = relative_path(uri)?;
        let candidate = self.root.join(relative);
        let parent = candidate.parent().ok_or_else(|| {
            TaruError::storage(
                uri.to_string(),
                StorageErrorKind::SecurityViolation,
                "local write target has no parent directory",
            )
        })?;
        let canonical_parent = parent.canonicalize().map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to resolve local write parent: {err}"),
            )
        })?;

        if !canonical_parent.starts_with(&self.root) {
            return Err(TaruError::storage(
                uri.to_string(),
                StorageErrorKind::SecurityViolation,
                "resolved local write path escaped backend root",
            ));
        }

        Ok(candidate)
    }

    fn metadata_for(&self, path: &Path, uri: StorageUri) -> Result<ObjectMetadata> {
        let metadata = fs::symlink_metadata(path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to read local metadata: {err}"),
            )
        })?;

        let file_type = metadata.file_type();
        let kind = if file_type.is_file() {
            ObjectKind::File
        } else if file_type.is_dir() {
            ObjectKind::Directory
        } else if file_type.is_symlink() {
            ObjectKind::Symlink
        } else {
            ObjectKind::Other
        };

        let len = metadata.is_file().then_some(metadata.len());
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| value.as_millis().to_string());
        let fingerprint = len
            .zip(modified_at.as_ref())
            .map(|(len, modified_at)| format!("local:size={len}:mtime={modified_at}"));

        Ok(ObjectMetadata {
            uri,
            kind,
            len,
            modified_at,
            etag: None,
            fingerprint,
            capabilities: local_capabilities(kind),
            cache: None,
        })
    }

    fn uri_for_path(&self, path: &Path) -> Result<StorageUri> {
        let relative = path.strip_prefix(&self.root).map_err(|err| {
            TaruError::storage(
                path.display().to_string(),
                StorageErrorKind::SecurityViolation,
                format!("failed to build local uri: {err}"),
            )
        })?;

        let relative = relative.to_string_lossy().replace('\\', "/");
        StorageUri::from_parts("local", &relative)
    }

    fn ensure_local_scheme(&self, uri: &StorageUri) -> Result<()> {
        if uri.scheme() != self.scheme() {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "local backend only accepts '{}' uris, got '{}'",
                    self.scheme(),
                    uri.scheme()
                ),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl StorageBackend for LocalFsBackend {
    fn scheme(&self) -> &'static str {
        "local"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        let path = self.path_for(uri)?;
        self.metadata_for(&path, uri.clone())
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        let path = self.path_for(uri)?;
        let stat = self.metadata_for(&path, uri.clone())?;

        if stat.kind != ObjectKind::Directory {
            return Err(TaruError::InvalidInput {
                message: format!("cannot list non-directory local uri: {uri}"),
            });
        }

        let mut entries = Vec::new();

        for entry in fs::read_dir(&path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to list local directory: {err}"),
            )
        })? {
            let entry = entry.map_err(|err| {
                TaruError::storage_io(
                    uri.to_string(),
                    format!("failed to read local directory entry: {err}"),
                )
            })?;
            let entry_uri = self.uri_for_path(&entry.path())?;
            entries.push(self.metadata_for(&entry.path(), entry_uri)?);
        }

        entries.sort_by(|left, right| left.uri.as_str().cmp(right.uri.as_str()));
        Ok(entries)
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        let path = self.path_for(uri)?;
        let metadata = fs::metadata(&path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to read local file metadata: {err}"),
            )
        })?;

        if !metadata.is_file() {
            return Err(TaruError::InvalidInput {
                message: format!("cannot open non-file local uri: {uri}"),
            });
        }

        if let Some(range) = range {
            validate_range(uri, range, metadata.len())?;
        }

        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: Some(path),
        })
    }

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        let path = self.path_for(uri)?;
        let metadata = fs::metadata(&path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to read local file metadata: {err}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(TaruError::InvalidInput {
                message: format!("cannot read non-file local uri: {uri}"),
            });
        }

        let bytes = fs::read(&path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to read local file range: {err}"),
            )
        })?;
        let bytes = match range {
            Some(range) => {
                validate_range(uri, range, metadata.len())?;
                let start = usize::try_from(range.offset).map_err(|err| {
                    TaruError::storage(
                        uri.to_string(),
                        StorageErrorKind::Unknown,
                        format!("range offset does not fit memory index: {err}"),
                    )
                })?;
                let end = match range.length {
                    Some(length) => {
                        let end = range.offset.checked_add(length).ok_or_else(|| {
                            TaruError::InvalidInput {
                                message: format!("range overflows file length: {uri}"),
                            }
                        })?;
                        usize::try_from(end).map_err(|err| {
                            TaruError::storage(
                                uri.to_string(),
                                StorageErrorKind::Unknown,
                                format!("range end does not fit memory index: {err}"),
                            )
                        })?
                    }
                    None => bytes.len(),
                };
                bytes[start..end].to_vec()
            }
            None => bytes,
        };

        Ok(ReadRange {
            uri: uri.clone(),
            range,
            bytes,
        })
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        let path = self.path_for(uri)?;
        fs::read_to_string(&path).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to read local text file: {err}"),
            )
        })
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        let path = self.writable_path_for(uri)?;
        fs::write(&path, content).map_err(|err| {
            TaruError::storage_io(
                uri.to_string(),
                format!("failed to write local text file: {err}"),
            )
        })
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let metadata = self.stat(&request.uri).await?;
        let file = self.open_range(&request.uri, None).await?;
        let Some(path) = file.local_path_hint else {
            return Err(TaruError::storage(
                request.uri.to_string(),
                StorageErrorKind::Unknown,
                "local backend did not return a local path hint",
            ));
        };

        Ok(StagedFile {
            uri: request.uri,
            path,
            len: metadata.len,
            etag: metadata.etag,
            fingerprint: metadata.fingerprint,
            reused: true,
        })
    }
}

fn local_capabilities(kind: ObjectKind) -> StorageCapabilities {
    let base = StorageCapabilities::SEEKABLE
        | StorageCapabilities::RANGE_READABLE
        | StorageCapabilities::WATCHABLE
        | StorageCapabilities::LINKABLE
        | StorageCapabilities::WRITABLE;

    match kind {
        ObjectKind::File | ObjectKind::Symlink => base,
        ObjectKind::Directory => base,
        ObjectKind::Other => StorageCapabilities::empty(),
    }
}

fn relative_path(uri: &StorageUri) -> Result<PathBuf> {
    let raw = uri.path_part().trim_start_matches(['/', '\\']);
    let normalized = raw.replace('\\', "/");
    let mut relative = PathBuf::new();

    if normalized.is_empty() {
        return Ok(relative);
    }

    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => relative.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(TaruError::InvalidInput {
                    message: format!("local uri path is not allowed to escape root: {uri}"),
                });
            }
        }
    }

    Ok(relative)
}

fn validate_range(uri: &StorageUri, range: ByteRange, len: u64) -> Result<()> {
    if range.offset > len {
        return Err(TaruError::InvalidInput {
            message: format!(
                "range offset {} exceeds file length {len}: {uri}",
                range.offset
            ),
        });
    }

    if let Some(length) = range.length {
        if length == 0 {
            return Err(TaruError::InvalidInput {
                message: format!("range length must be greater than zero: {uri}"),
            });
        }

        let Some(end) = range.offset.checked_add(length) else {
            return Err(TaruError::InvalidInput {
                message: format!("range overflows file length: {uri}"),
            });
        };

        if end > len {
            return Err(TaruError::InvalidInput {
                message: format!("range end {end} exceeds file length {len}: {uri}"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn local_backend_lists_and_stats_files_under_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), b"taru").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let root_uri = StorageUri::from_parts("local", "movies").unwrap();
            let entries = backend.list(&root_uri).await.unwrap();

            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].uri.as_str(), "local:///movies/demo.mkv");
            assert_eq!(entries[0].kind, ObjectKind::File);
            assert_eq!(entries[0].len, Some(4));

            let metadata = backend.stat(&entries[0].uri).await.unwrap();
            assert_eq!(metadata.kind, ObjectKind::File);
        });
    }

    #[test]
    fn local_backend_returns_local_path_hint_for_ranges() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::write(temp.path().join("demo.mkv"), b"taru").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "demo.mkv").unwrap();
            let file = backend
                .open_range(
                    &uri,
                    Some(ByteRange {
                        offset: 1,
                        length: Some(2),
                    }),
                )
                .await
                .unwrap();

            assert_eq!(file.uri, uri);
            assert_eq!(
                file.local_path_hint,
                Some(temp.path().join("demo.mkv").canonicalize().unwrap())
            );
        });
    }

    #[test]
    fn local_backend_rejects_path_traversal() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::parse("local:///../outside.mkv").unwrap();

            assert!(backend.stat(&uri).await.is_err());
        });
    }

    #[test]
    fn local_backend_rejects_out_of_bounds_ranges() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::write(temp.path().join("demo.mkv"), b"taru").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "demo.mkv").unwrap();
            let result = backend
                .open_range(
                    &uri,
                    Some(ByteRange {
                        offset: 3,
                        length: Some(2),
                    }),
                )
                .await;

            assert!(result.is_err());
        });
    }

    #[test]
    fn local_backend_reads_and_writes_text_files_under_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            backend.write_string(&uri, "<movie />").await.unwrap();
            let content = backend.read_to_string(&uri).await.unwrap();

            assert_eq!(content, "<movie />");
        });
    }

    #[test]
    fn local_backend_rejects_text_writes_outside_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::parse("local:///../outside.nfo").unwrap();

            assert!(backend.write_string(&uri, "bad").await.is_err());
        });
    }
}
