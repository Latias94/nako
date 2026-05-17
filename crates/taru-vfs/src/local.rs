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
    StorageBackupMode, StorageBackupPolicy, StorageBackupPruneFailure, StorageBackupReport,
    StorageCapabilities, StorageUri, StorageWriteMode, StorageWriteReport, StorageWriteRequest,
    VirtualFile,
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

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        match request.mode {
            StorageWriteMode::Direct => {
                let backup = self.backup_for_request(&request.uri, &request.backup)?;
                self.write_string(&request.uri, &request.content).await?;
                Ok(StorageWriteReport {
                    uri: request.uri,
                    mode: StorageWriteMode::Direct,
                    atomic: false,
                    backup,
                })
            }
            StorageWriteMode::AtomicReplace => {
                let (atomic, backup) = self.write_string_atomic_replace(
                    &request.uri,
                    &request.content,
                    &request.backup,
                )?;
                Ok(StorageWriteReport {
                    uri: request.uri,
                    mode: StorageWriteMode::AtomicReplace,
                    atomic,
                    backup,
                })
            }
        }
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

impl LocalFsBackend {
    fn write_string_atomic_replace(
        &self,
        uri: &StorageUri,
        content: &str,
        backup: &StorageBackupPolicy,
    ) -> Result<(bool, Option<StorageBackupReport>)> {
        let path = self.writable_path_for(uri)?;
        let parent = path.parent().ok_or_else(|| {
            TaruError::storage(
                uri.to_string(),
                StorageErrorKind::SecurityViolation,
                "local atomic write target has no parent directory",
            )
        })?;
        let temp_path = atomic_temp_path(&path);
        let write_result = (|| -> Result<(bool, Option<StorageBackupReport>)> {
            let backup = self.backup_for_path(uri, &path, backup)?;
            {
                let mut file = fs::File::create(&temp_path).map_err(|err| {
                    TaruError::storage_io(
                        temp_path.display().to_string(),
                        format!("failed to create local atomic temp file: {err}"),
                    )
                })?;
                use std::io::Write as _;
                file.write_all(content.as_bytes()).map_err(|err| {
                    TaruError::storage_io(
                        temp_path.display().to_string(),
                        format!("failed to write local atomic temp file: {err}"),
                    )
                })?;
                file.sync_all().map_err(|err| {
                    TaruError::storage_io(
                        temp_path.display().to_string(),
                        format!("failed to sync local atomic temp file: {err}"),
                    )
                })?;
            }

            let atomic = replace_temp_file(uri, &temp_path, &path)?;

            sync_directory_if_possible(parent);

            Ok((atomic, backup))
        })();

        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }

        write_result
    }

    fn backup_for_request(
        &self,
        uri: &StorageUri,
        backup: &StorageBackupPolicy,
    ) -> Result<Option<StorageBackupReport>> {
        let path = self.writable_path_for(uri)?;
        self.backup_for_path(uri, &path, backup)
    }

    fn backup_for_path(
        &self,
        uri: &StorageUri,
        path: &Path,
        backup: &StorageBackupPolicy,
    ) -> Result<Option<StorageBackupReport>> {
        match backup.mode {
            StorageBackupMode::None => Ok(None),
            StorageBackupMode::ExistingFile => {
                if !path.exists() {
                    return Ok(None);
                }
                let backup_path = local_backup_path(path);
                fs::copy(path, &backup_path).map_err(|err| {
                    TaruError::storage_backup(
                        uri.to_string(),
                        format!("failed to create local backup before write: {err}"),
                    )
                })?;
                sync_file_if_possible(&backup_path);
                if let Some(parent) = backup_path.parent() {
                    sync_directory_if_possible(parent);
                }
                let (pruned_backups, prune_failures) =
                    self.prune_backups_for_path(path, backup.retention.keep_latest)?;
                Ok(Some(StorageBackupReport {
                    original_uri: uri.clone(),
                    backup_uri: self.uri_for_local_path(&backup_path)?,
                    pruned_backups,
                    prune_failures,
                }))
            }
        }
    }

    fn prune_backups_for_path(
        &self,
        path: &Path,
        keep_latest: Option<usize>,
    ) -> Result<(Vec<StorageUri>, Vec<StorageBackupPruneFailure>)> {
        let Some(keep_latest) = keep_latest else {
            return Ok((Vec::new(), Vec::new()));
        };
        let Some(parent) = path.parent() else {
            return Ok((Vec::new(), Vec::new()));
        };
        let prefix = local_backup_file_prefix(path);
        let mut candidates = Vec::new();

        for entry in fs::read_dir(parent).map_err(|err| {
            TaruError::storage_backup(
                path.display().to_string(),
                format!("failed to list local backup directory for pruning: {err}"),
            )
        })? {
            let entry = entry.map_err(|err| {
                TaruError::storage_backup(
                    path.display().to_string(),
                    format!("failed to read local backup directory entry for pruning: {err}"),
                )
            })?;
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with(&prefix) {
                candidates.push(entry.path());
            }
        }

        candidates.sort_by(|left, right| {
            let left_name = left
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            let right_name = right
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            right_name.cmp(left_name)
        });

        let mut pruned = Vec::new();
        let mut failures = Vec::new();
        for candidate in candidates.into_iter().skip(keep_latest) {
            let uri = self.uri_for_local_path(&candidate)?;
            match fs::remove_file(&candidate) {
                Ok(()) => pruned.push(uri),
                Err(err) => failures.push(StorageBackupPruneFailure {
                    uri,
                    message: format!("failed to prune local backup: {err}"),
                }),
            }
        }

        if !pruned.is_empty() || !failures.is_empty() {
            sync_directory_if_possible(parent);
        }

        Ok((pruned, failures))
    }

    fn uri_for_local_path(&self, path: &Path) -> Result<StorageUri> {
        let relative = path.strip_prefix(&self.root).map_err(|err| {
            TaruError::storage_security_violation(
                path.display().to_string(),
                format!("local backup path escaped backend root: {err}"),
            )
        })?;
        let relative = relative.to_string_lossy().replace('\\', "/");
        StorageUri::from_parts("local", &relative)
    }
}

#[cfg(not(windows))]
fn replace_temp_file(uri: &StorageUri, temp_path: &Path, path: &Path) -> Result<bool> {
    fs::rename(temp_path, path).map_err(|err| {
        TaruError::storage_io(
            uri.to_string(),
            format!("failed to replace local file atomically: {err}"),
        )
    })?;
    Ok(true)
}

#[cfg(windows)]
fn replace_temp_file(uri: &StorageUri, temp_path: &Path, path: &Path) -> Result<bool> {
    match fs::rename(temp_path, path) {
        Ok(()) => Ok(true),
        Err(rename_err) if path.exists() => {
            fs::remove_file(path).map_err(|err| {
                TaruError::storage_io(
                    uri.to_string(),
                    format!(
                        "failed to remove existing local file after atomic replace was unavailable: {err}"
                    ),
                )
            })?;
            fs::rename(temp_path, path).map_err(|err| {
                TaruError::storage_io(
                    uri.to_string(),
                    format!(
                        "failed to replace local file after atomic replace was unavailable: {err}"
                    ),
                )
            })?;
            let _ = rename_err;
            Ok(false)
        }
        Err(err) => Err(TaruError::storage_io(
            uri.to_string(),
            format!("failed to replace local file atomically: {err}"),
        )),
    }
}

fn atomic_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("taru-write");
    let nonce = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let process_id = std::process::id();
    path.with_file_name(format!(".{file_name}.taru-{process_id}-{nonce}.tmp"))
}

fn local_backup_path(path: &Path) -> PathBuf {
    let file_name = local_sidecar_file_name(path);
    let nonce = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    path.with_file_name(format!("{file_name}.taru-backup-{nonce}"))
}

fn local_backup_file_prefix(path: &Path) -> String {
    format!("{}.taru-backup-", local_sidecar_file_name(path))
}

fn local_sidecar_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("taru-sidecar")
        .to_owned()
}

fn sync_file_if_possible(path: &Path) {
    if let Ok(file) = fs::File::open(path) {
        let _ = file.sync_all();
    }
}

fn sync_directory_if_possible(path: &Path) {
    if let Ok(directory) = fs::File::open(path) {
        let _ = directory.sync_all();
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
    fn local_backend_atomic_replace_creates_text_file_under_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(StorageWriteRequest::atomic_replace(
                    uri.clone(),
                    "<movie><title>Created</title></movie>",
                ))
                .await
                .unwrap();
            let content = backend.read_to_string(&uri).await.unwrap();

            assert_eq!(report.uri, uri);
            assert_eq!(report.mode, StorageWriteMode::AtomicReplace);
            assert!(report.atomic);
            assert_eq!(report.backup, None);
            assert_eq!(content, "<movie><title>Created</title></movie>");
            assert_no_atomic_temp_files(temp.path().join("movies").as_path());
        });
    }

    #[test]
    fn local_backend_atomic_replace_updates_existing_text_file_without_temp_leftovers() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.nfo"), "old").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(StorageWriteRequest::atomic_replace(uri.clone(), "new"))
                .await
                .unwrap();
            let content = backend.read_to_string(&uri).await.unwrap();

            assert_eq!(report.uri, uri);
            assert_eq!(report.mode, StorageWriteMode::AtomicReplace);
            assert!(report.atomic || cfg!(windows));
            assert_eq!(report.backup, None);
            assert_eq!(content, "new");
            assert_no_atomic_temp_files(temp.path().join("movies").as_path());
        });
    }

    #[test]
    fn local_backend_backup_atomic_replace_copies_existing_file_before_overwrite() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.nfo"), "old").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(
                    StorageWriteRequest::atomic_replace(uri.clone(), "new")
                        .with_backup(StorageBackupMode::ExistingFile),
                )
                .await
                .unwrap();
            let content = backend.read_to_string(&uri).await.unwrap();
            let backup = report.backup.unwrap();
            let backup_content = backend.read_to_string(&backup.backup_uri).await.unwrap();

            assert_eq!(backup.original_uri, uri);
            assert!(
                backup
                    .backup_uri
                    .as_str()
                    .starts_with("local:///movies/demo.nfo.taru-backup-")
            );
            assert_eq!(content, "new");
            assert_eq!(backup_content, "old");
            assert_no_atomic_temp_files(temp.path().join("movies").as_path());
        });
    }

    #[test]
    fn local_backend_backup_atomic_replace_skips_backup_for_new_file() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(
                    StorageWriteRequest::atomic_replace(uri.clone(), "new")
                        .with_backup(StorageBackupMode::ExistingFile),
                )
                .await
                .unwrap();

            assert_eq!(report.backup, None);
            assert_eq!(backend.read_to_string(&uri).await.unwrap(), "new");
            assert_eq!(
                backup_files(temp.path().join("movies").as_path()),
                Vec::<String>::new()
            );
        });
    }

    #[test]
    fn local_backend_backup_retention_prunes_old_taru_backups_for_same_sidecar() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "current").unwrap();
            fs::write(movies.join("demo.nfo.taru-backup-0001"), "oldest").unwrap();
            fs::write(movies.join("demo.nfo.taru-backup-0002"), "middle").unwrap();
            fs::write(movies.join("demo.nfo.taru-backup-0003"), "newest").unwrap();
            fs::write(movies.join("other.nfo.taru-backup-0001"), "other").unwrap();
            fs::write(movies.join("demo.nfo.manual-backup"), "manual").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(
                    StorageWriteRequest::atomic_replace(uri.clone(), "replacement")
                        .with_backup_policy(StorageBackupPolicy::existing_file().keep_latest(2)),
                )
                .await
                .unwrap();
            let backup = report.backup.unwrap();
            let pruned = backup
                .pruned_backups
                .iter()
                .map(StorageUri::as_str)
                .collect::<Vec<_>>();

            assert_eq!(backup.prune_failures, Vec::new());
            assert_eq!(
                pruned,
                vec![
                    "local:///movies/demo.nfo.taru-backup-0002",
                    "local:///movies/demo.nfo.taru-backup-0001"
                ]
            );
            assert_eq!(backend.read_to_string(&uri).await.unwrap(), "replacement");
            assert!(movies.join("demo.nfo.taru-backup-0003").exists());
            assert!(movies.join("other.nfo.taru-backup-0001").exists());
            assert!(movies.join("demo.nfo.manual-backup").exists());
            assert!(
                backup
                    .backup_uri
                    .as_str()
                    .starts_with("local:///movies/demo.nfo.taru-backup-")
            );
        });
    }

    #[test]
    fn local_backend_backup_retention_zero_prunes_all_taru_backups_after_write() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "current").unwrap();
            fs::write(movies.join("demo.nfo.taru-backup-0001"), "old").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(
                    StorageWriteRequest::atomic_replace(uri, "replacement")
                        .with_backup_policy(StorageBackupPolicy::existing_file().keep_latest(0)),
                )
                .await
                .unwrap();
            let backup = report.backup.unwrap();

            assert_eq!(backup.prune_failures, Vec::new());
            assert_eq!(backup.pruned_backups.len(), 2);
            assert_eq!(backup_files(&movies), Vec::<String>::new());
        });
    }

    #[test]
    fn local_backend_backup_retention_reports_prune_failures() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "current").unwrap();
            fs::create_dir(movies.join("demo.nfo.taru-backup-0000")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .write(
                    StorageWriteRequest::atomic_replace(uri, "replacement")
                        .with_backup_policy(StorageBackupPolicy::existing_file().keep_latest(0)),
                )
                .await
                .unwrap();
            let backup = report.backup.unwrap();

            assert_eq!(backup.pruned_backups.len(), 1);
            assert_eq!(backup.prune_failures.len(), 1);
            assert_eq!(
                backup.prune_failures[0].uri.as_str(),
                "local:///movies/demo.nfo.taru-backup-0000"
            );
            assert!(movies.join("demo.nfo.taru-backup-0000").exists());
        });
    }

    #[test]
    fn default_backend_rejects_backup_writes_explicitly() {
        pollster::block_on(async {
            let backend = DirectOnlyBackend;
            let uri = StorageUri::from_parts("memory", "demo.nfo").unwrap();

            let err = backend
                .write(
                    StorageWriteRequest::direct(uri, "new")
                        .with_backup(StorageBackupMode::ExistingFile),
                )
                .await
                .unwrap_err();

            assert_eq!(
                err,
                TaruError::Unsupported("storage backend does not support backup writes")
            );
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

    #[test]
    fn local_backend_rejects_atomic_text_writes_outside_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let uri = StorageUri::parse("local:///../outside.nfo").unwrap();

            assert!(
                backend
                    .write(StorageWriteRequest::atomic_replace(uri, "bad"))
                    .await
                    .is_err()
            );
        });
    }

    fn assert_no_atomic_temp_files(path: &Path) {
        let leftovers = fs::read_dir(path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".taru-") && name.ends_with(".tmp"))
            .collect::<Vec<_>>();

        assert_eq!(leftovers, Vec::<String>::new());
    }

    fn backup_files(path: &Path) -> Vec<String> {
        fs::read_dir(path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".taru-backup-"))
            .collect()
    }

    struct DirectOnlyBackend;

    #[async_trait::async_trait]
    impl StorageBackend for DirectOnlyBackend {
        fn scheme(&self) -> &'static str {
            "memory"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Err(TaruError::NotFound {
                entity: "storage_object",
                id: uri.to_string(),
            })
        }

        async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Ok(Vec::new())
        }

        async fn open_range(
            &self,
            _uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(TaruError::Unsupported(
                "direct-only backend does not support opening files",
            ))
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(TaruError::NotFound {
                entity: "storage_object",
                id: "memory".to_owned(),
            })
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Ok(())
        }
    }
}
