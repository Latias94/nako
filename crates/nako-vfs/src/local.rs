use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use async_trait::async_trait;
use nako_core::{NakoError, Result, StorageErrorKind};

use crate::{
    ByteRange, ObjectKind, ObjectMetadata, ReadRange, StageRequest, StagedFile, StorageApplyReport,
    StorageApplyRequest, StorageBackend, StorageBackupPolicy, StorageBackupReport,
    StorageCapabilities, StorageCleanupReport, StorageCleanupRequest, StorageLinkPlan,
    StorageLinkPlanRequest, StorageRestoreReport, StorageRestoreRequest, StorageUri,
    StorageWriteMode, StorageWriteReport, StorageWriteRequest, VirtualFile,
};

mod apply_plan;
mod lifecycle;
mod path_authority;
mod write_transaction;

#[derive(Clone, Debug)]
pub struct LocalFsBackend {
    root: PathBuf,
}

impl LocalFsBackend {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = path_authority::canonicalize_root(root.into())?;

        Ok(Self { root })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, uri: &StorageUri) -> Result<PathBuf> {
        path_authority::existing_path_for(&self.root, uri, self.scheme())
    }

    fn writable_path_for(&self, uri: &StorageUri) -> Result<PathBuf> {
        path_authority::writable_path_for(&self.root, uri, self.scheme())
    }

    fn cleanup_path_for(&self, uri: &StorageUri) -> Result<PathBuf> {
        path_authority::cleanup_path_for(&self.root, uri, self.scheme())
    }

    fn metadata_for(&self, path: &Path, uri: StorageUri) -> Result<ObjectMetadata> {
        let metadata = fs::symlink_metadata(path).map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                NakoError::NotFound {
                    entity: "storage_object",
                    id: uri.to_string(),
                }
            } else {
                NakoError::storage_io(
                    uri.to_string(),
                    format!("failed to read local metadata: {err}"),
                )
            }
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
        path_authority::uri_for_path(&self.root, path)
    }

    fn ensure_local_scheme(&self, uri: &StorageUri) -> Result<()> {
        path_authority::ensure_local_scheme(uri, self.scheme())
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
            return Err(NakoError::InvalidInput {
                message: format!("cannot list non-directory local uri: {uri}"),
            });
        }

        let mut entries = Vec::new();

        for entry in fs::read_dir(&path).map_err(|err| {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to list local directory: {err}"),
            )
        })? {
            let entry = entry.map_err(|err| {
                NakoError::storage_io(
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
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to read local file metadata: {err}"),
            )
        })?;

        if !metadata.is_file() {
            return Err(NakoError::InvalidInput {
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
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to read local file metadata: {err}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(NakoError::InvalidInput {
                message: format!("cannot read non-file local uri: {uri}"),
            });
        }

        let bytes = fs::read(&path).map_err(|err| {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to read local file range: {err}"),
            )
        })?;
        let bytes = match range {
            Some(range) => {
                validate_range(uri, range, metadata.len())?;
                let start = usize::try_from(range.offset).map_err(|err| {
                    NakoError::storage(
                        uri.to_string(),
                        StorageErrorKind::Unknown,
                        format!("range offset does not fit memory index: {err}"),
                    )
                })?;
                let end = match range.length {
                    Some(length) => {
                        let end = range.offset.checked_add(length).ok_or_else(|| {
                            NakoError::InvalidInput {
                                message: format!("range overflows file length: {uri}"),
                            }
                        })?;
                        usize::try_from(end).map_err(|err| {
                            NakoError::storage(
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
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to read local text file: {err}"),
            )
        })
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        let path = self.writable_path_for(uri)?;
        fs::write(&path, content).map_err(|err| {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to write local text file: {err}"),
            )
        })
    }

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        match request.mode {
            StorageWriteMode::Direct => {
                let path = self.writable_path_for(&request.uri)?;
                let backup = write_transaction::backup_for_path(
                    &self.root,
                    &request.uri,
                    &path,
                    &request.backup,
                )?;
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

    async fn plan_link(&self, request: StorageLinkPlanRequest) -> Result<StorageLinkPlan> {
        apply_plan::plan_local_link(self, request)
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<StorageApplyReport> {
        apply_plan::apply_local(self, request)
    }

    async fn cleanup(&self, request: StorageCleanupRequest) -> Result<StorageCleanupReport> {
        lifecycle::cleanup_local(self, request)
    }

    async fn restore(&self, request: StorageRestoreRequest) -> Result<StorageRestoreReport> {
        lifecycle::restore_local(self, request)
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let metadata = self.stat(&request.uri).await?;
        let file = self.open_range(&request.uri, None).await?;
        let Some(path) = file.local_path_hint else {
            return Err(NakoError::storage(
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
        write_transaction::write_string_atomic_replace(&self.root, uri, &path, content, backup)
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

fn validate_range(uri: &StorageUri, range: ByteRange, len: u64) -> Result<()> {
    if range.offset > len {
        return Err(NakoError::InvalidInput {
            message: format!(
                "range offset {} exceeds file length {len}: {uri}",
                range.offset
            ),
        });
    }

    if let Some(length) = range.length {
        if length == 0 {
            return Err(NakoError::InvalidInput {
                message: format!("range length must be greater than zero: {uri}"),
            });
        }

        let Some(end) = range.offset.checked_add(length) else {
            return Err(NakoError::InvalidInput {
                message: format!("range overflows file length: {uri}"),
            });
        };

        if end > len {
            return Err(NakoError::InvalidInput {
                message: format!("range end {end} exceeds file length {len}: {uri}"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        StorageApplyKind, StorageApplyStatus, StorageBackupMode, StorageCleanupStatus,
        StorageLinkPlanStatus, StorageRestoreStatus,
    };

    use super::*;

    #[test]
    fn local_backend_lists_and_stats_files_under_root() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), b"nako").unwrap();

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
            fs::write(temp.path().join("demo.mkv"), b"nako").unwrap();

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
            fs::write(temp.path().join("demo.mkv"), b"nako").unwrap();

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
                    .starts_with("local:///movies/demo.nfo.nako-backup-")
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
    fn local_backend_backup_retention_prunes_old_nako_backups_for_same_sidecar() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "current").unwrap();
            fs::write(movies.join("demo.nfo.nako-backup-0001"), "oldest").unwrap();
            fs::write(movies.join("demo.nfo.nako-backup-0002"), "middle").unwrap();
            fs::write(movies.join("demo.nfo.nako-backup-0003"), "newest").unwrap();
            fs::write(movies.join("other.nfo.nako-backup-0001"), "other").unwrap();
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
                    "local:///movies/demo.nfo.nako-backup-0002",
                    "local:///movies/demo.nfo.nako-backup-0001"
                ]
            );
            assert_eq!(backend.read_to_string(&uri).await.unwrap(), "replacement");
            assert!(movies.join("demo.nfo.nako-backup-0003").exists());
            assert!(movies.join("other.nfo.nako-backup-0001").exists());
            assert!(movies.join("demo.nfo.manual-backup").exists());
            assert!(
                backup
                    .backup_uri
                    .as_str()
                    .starts_with("local:///movies/demo.nfo.nako-backup-")
            );
        });
    }

    #[test]
    fn local_backend_backup_retention_zero_prunes_all_nako_backups_after_write() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "current").unwrap();
            fs::write(movies.join("demo.nfo.nako-backup-0001"), "old").unwrap();

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
            fs::create_dir(movies.join("demo.nfo.nako-backup-0000")).unwrap();

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
                "local:///movies/demo.nfo.nako-backup-0000"
            );
            assert!(movies.join("demo.nfo.nako-backup-0000").exists());
        });
    }

    #[test]
    fn local_backend_restore_replaces_target_from_backup_atomically() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "new-but-uncommitted").unwrap();
            fs::write(movies.join("demo.nfo.nako-backup-0001"), "old").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let backup_uri =
                StorageUri::from_parts("local", "movies/demo.nfo.nako-backup-0001").unwrap();
            let target_uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();

            let report = backend
                .restore(StorageRestoreRequest::new(
                    backup_uri.clone(),
                    target_uri.clone(),
                ))
                .await
                .unwrap();

            assert_eq!(report.backup_uri, backup_uri);
            assert_eq!(report.target_uri, target_uri);
            assert_eq!(report.status, StorageRestoreStatus::Restored);
            assert!(report.restored);
            assert_eq!(backend.read_to_string(&target_uri).await.unwrap(), "old");
            assert_eq!(
                backend.read_to_string(&report.backup_uri).await.unwrap(),
                "old"
            );
            assert_no_atomic_temp_files(&movies);
        });
    }

    #[test]
    fn local_backend_restore_reports_missing_backup_without_touching_target() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            let movies = temp.path().join("movies");
            fs::create_dir(&movies).unwrap();
            fs::write(movies.join("demo.nfo"), "new-but-uncommitted").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let target_uri = StorageUri::from_parts("local", "movies/demo.nfo").unwrap();
            let report = backend
                .restore(StorageRestoreRequest::new(
                    StorageUri::from_parts("local", "movies/missing.nfo.nako-backup-0001").unwrap(),
                    target_uri.clone(),
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageRestoreStatus::BackupMissing);
            assert!(!report.restored);
            assert_eq!(
                backend.read_to_string(&target_uri).await.unwrap(),
                "new-but-uncommitted"
            );
            assert_no_atomic_temp_files(&movies);
        });
    }

    #[test]
    fn local_backend_plans_hard_link_without_creating_target() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let source = StorageUri::from_parts("local", "movies/demo.mkv").unwrap();
            let target = StorageUri::from_parts("local", "movies/demo-copy.mkv").unwrap();

            let plan = backend
                .plan_link(StorageLinkPlanRequest::new(
                    source.clone(),
                    target.clone(),
                    crate::StorageLinkKind::Hard,
                ))
                .await
                .unwrap();

            assert_eq!(plan.source_uri, source);
            assert_eq!(plan.target_uri, target);
            assert_eq!(plan.status, StorageLinkPlanStatus::Ready);
            assert!(plan.can_apply);
            assert_eq!(plan.source.unwrap().kind, ObjectKind::File);
            assert_eq!(plan.target, None);
            assert!(!temp.path().join("movies").join("demo-copy.mkv").exists());
        });
    }

    #[test]
    fn local_backend_applies_copy_without_exposing_os_paths() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let source = StorageUri::from_parts("local", "movies/demo.mkv").unwrap();
            let target = StorageUri::from_parts("local", "movies/demo-copy.mkv").unwrap();

            let report = backend
                .apply(StorageApplyRequest::new(
                    source.clone(),
                    target.clone(),
                    StorageApplyKind::Copy,
                ))
                .await
                .unwrap();

            assert_eq!(report.source_uri, source);
            assert_eq!(report.target_uri, target);
            assert_eq!(report.kind, StorageApplyKind::Copy);
            assert_eq!(report.status, StorageApplyStatus::Applied);
            assert!(report.applied);
            assert_eq!(report.source.unwrap().kind, ObjectKind::File);
            assert_eq!(report.target.unwrap().kind, ObjectKind::File);
            assert_eq!(
                fs::read_to_string(temp.path().join("movies").join("demo.mkv")).unwrap(),
                "media"
            );
            assert_eq!(
                fs::read_to_string(temp.path().join("movies").join("demo-copy.mkv")).unwrap(),
                "media"
            );
            assert!(
                !report
                    .message
                    .contains(temp.path().to_string_lossy().as_ref())
            );
        });
    }

    #[test]
    fn local_backend_applies_hard_link_through_ready_plan() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let source = StorageUri::from_parts("local", "movies/demo.mkv").unwrap();
            let target = StorageUri::from_parts("local", "movies/demo-hardlink.mkv").unwrap();

            let report = backend
                .apply(StorageApplyRequest::new(
                    source.clone(),
                    target.clone(),
                    StorageApplyKind::Hardlink,
                ))
                .await
                .unwrap();

            assert_eq!(report.source_uri, source);
            assert_eq!(report.target_uri, target);
            assert_eq!(report.kind, StorageApplyKind::Hardlink);
            assert_eq!(report.status, StorageApplyStatus::Applied);
            assert!(report.applied);
            assert_eq!(
                fs::read_to_string(temp.path().join("movies").join("demo-hardlink.mkv")).unwrap(),
                "media"
            );
        });
    }

    #[test]
    fn local_backend_apply_reuses_plan_and_does_not_overwrite_targets() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();
            fs::write(temp.path().join("movies").join("demo-copy.mkv"), "existing").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let report = backend
                .apply(StorageApplyRequest::new(
                    StorageUri::from_parts("local", "movies/demo.mkv").unwrap(),
                    StorageUri::from_parts("local", "movies/demo-copy.mkv").unwrap(),
                    StorageApplyKind::Copy,
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageApplyStatus::TargetExists);
            assert!(!report.applied);
            assert_eq!(
                fs::read_to_string(temp.path().join("movies").join("demo-copy.mkv")).unwrap(),
                "existing"
            );
        });
    }

    #[test]
    fn local_backend_apply_reports_security_violation_without_mutation() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let report = backend
                .apply(StorageApplyRequest::new(
                    StorageUri::from_parts("local", "movies/demo.mkv").unwrap(),
                    StorageUri::parse("local:///../outside.mkv").unwrap(),
                    StorageApplyKind::Copy,
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageApplyStatus::SecurityViolation);
            assert!(!report.applied);
            assert!(!temp.path().join("outside.mkv").exists());
        });
    }

    #[test]
    fn local_backend_applies_symlink_when_platform_allows_it() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let report = backend
                .apply(StorageApplyRequest::new(
                    StorageUri::from_parts("local", "movies/demo.mkv").unwrap(),
                    StorageUri::from_parts("local", "movies/demo-symlink.mkv").unwrap(),
                    StorageApplyKind::Symlink,
                ))
                .await
                .unwrap();

            match report.status {
                StorageApplyStatus::Applied => {
                    assert!(report.applied);
                    assert_eq!(report.target.unwrap().kind, ObjectKind::Symlink);
                    assert_eq!(
                        fs::read_to_string(temp.path().join("movies").join("demo-symlink.mkv"))
                            .unwrap(),
                        "media"
                    );
                }
                StorageApplyStatus::ApplyFailed if cfg!(windows) => {
                    assert!(!report.applied);
                    assert!(!temp.path().join("movies").join("demo-symlink.mkv").exists());
                }
                status => panic!("unexpected symlink apply status: {status:?}"),
            }
        });
    }

    #[test]
    fn default_backend_apply_is_unsupported_without_mutation() {
        pollster::block_on(async {
            let backend = DirectOnlyBackend;
            let report = backend
                .apply(StorageApplyRequest::new(
                    StorageUri::from_parts("memory", "source.mkv").unwrap(),
                    StorageUri::from_parts("memory", "target.mkv").unwrap(),
                    StorageApplyKind::Copy,
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageApplyStatus::Unsupported);
            assert!(!report.applied);
            assert!(report.message.contains("does not support storage apply"));
        });
    }

    #[test]
    fn local_backend_cleanup_removes_file_without_exposing_os_paths() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo-copy.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let target = StorageUri::from_parts("local", "movies/demo-copy.mkv").unwrap();

            let report = backend
                .cleanup(StorageCleanupRequest::new(target.clone()))
                .await
                .unwrap();

            assert_eq!(report.target_uri, target);
            assert_eq!(report.status, StorageCleanupStatus::Cleaned);
            assert!(report.cleaned);
            assert_eq!(report.target.unwrap().kind, ObjectKind::File);
            assert!(!temp.path().join("movies").join("demo-copy.mkv").exists());
            assert!(
                !report
                    .message
                    .contains(temp.path().to_string_lossy().as_ref())
            );
        });
    }

    #[test]
    fn local_backend_cleanup_refuses_directories_without_mutation() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let report = backend
                .cleanup(StorageCleanupRequest::new(
                    StorageUri::from_parts("local", "movies").unwrap(),
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageCleanupStatus::TargetNotFile);
            assert!(!report.cleaned);
            assert_eq!(report.target.unwrap().kind, ObjectKind::Directory);
            assert!(temp.path().join("movies").exists());
        });
    }

    #[test]
    fn local_backend_cleanup_reports_security_violation_without_mutation() {
        pollster::block_on(async {
            let root = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            fs::write(outside.path().join("outside.mkv"), "outside").unwrap();

            let backend = LocalFsBackend::new(root.path()).unwrap();
            let report = backend
                .cleanup(StorageCleanupRequest::new(
                    StorageUri::parse("local:///../outside.mkv").unwrap(),
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageCleanupStatus::SecurityViolation);
            assert!(!report.cleaned);
            assert!(outside.path().join("outside.mkv").exists());
        });
    }

    #[test]
    fn local_backend_cleanup_reports_missing_target_without_mutation() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let report = backend
                .cleanup(StorageCleanupRequest::new(
                    StorageUri::from_parts("local", "movies/missing.mkv").unwrap(),
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageCleanupStatus::TargetMissing);
            assert!(!report.cleaned);
            assert!(temp.path().join("movies").exists());
        });
    }

    #[test]
    fn default_backend_cleanup_is_unsupported_without_mutation() {
        pollster::block_on(async {
            let backend = DirectOnlyBackend;
            let report = backend
                .cleanup(StorageCleanupRequest::new(
                    StorageUri::from_parts("memory", "target.mkv").unwrap(),
                ))
                .await
                .unwrap();

            assert_eq!(report.status, StorageCleanupStatus::Unsupported);
            assert!(!report.cleaned);
            assert!(report.message.contains("does not support storage cleanup"));
        });
    }

    #[test]
    fn local_backend_link_plan_reports_existing_target_without_overwrite() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();
            fs::write(temp.path().join("movies").join("demo-copy.mkv"), "existing").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let plan = backend
                .plan_link(StorageLinkPlanRequest::new(
                    StorageUri::from_parts("local", "movies/demo.mkv").unwrap(),
                    StorageUri::from_parts("local", "movies/demo-copy.mkv").unwrap(),
                    crate::StorageLinkKind::Soft,
                ))
                .await
                .unwrap();

            assert_eq!(plan.status, StorageLinkPlanStatus::TargetExists);
            assert!(!plan.can_apply);
            assert_eq!(plan.target.unwrap().kind, ObjectKind::File);
            assert_eq!(
                fs::read_to_string(temp.path().join("movies").join("demo-copy.mkv")).unwrap(),
                "existing"
            );
        });
    }

    #[test]
    fn local_backend_link_plan_reports_missing_source_and_parent() {
        pollster::block_on(async {
            let temp = tempfile::tempdir().unwrap();
            fs::create_dir(temp.path().join("movies")).unwrap();
            fs::write(temp.path().join("movies").join("demo.mkv"), "media").unwrap();

            let backend = LocalFsBackend::new(temp.path()).unwrap();
            let missing_source = backend
                .plan_link(StorageLinkPlanRequest::new(
                    StorageUri::from_parts("local", "movies/missing.mkv").unwrap(),
                    StorageUri::from_parts("local", "movies/copy.mkv").unwrap(),
                    crate::StorageLinkKind::Hard,
                ))
                .await
                .unwrap();
            let missing_parent = backend
                .plan_link(StorageLinkPlanRequest::new(
                    StorageUri::from_parts("local", "movies/demo.mkv").unwrap(),
                    StorageUri::from_parts("local", "missing/copy.mkv").unwrap(),
                    crate::StorageLinkKind::Hard,
                ))
                .await
                .unwrap();

            assert_eq!(missing_source.status, StorageLinkPlanStatus::SourceMissing);
            assert!(!missing_source.can_apply);
            assert_eq!(
                missing_parent.status,
                StorageLinkPlanStatus::TargetParentMissing
            );
            assert!(!missing_parent.can_apply);
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
                NakoError::Unsupported("storage backend does not support backup writes")
            );
        });
    }

    #[test]
    fn default_backend_link_plan_is_unsupported_without_mutation() {
        pollster::block_on(async {
            let backend = DirectOnlyBackend;
            let plan = backend
                .plan_link(StorageLinkPlanRequest::new(
                    StorageUri::from_parts("memory", "source.mkv").unwrap(),
                    StorageUri::from_parts("memory", "target.mkv").unwrap(),
                    crate::StorageLinkKind::Soft,
                ))
                .await
                .unwrap();

            assert_eq!(plan.status, StorageLinkPlanStatus::Unsupported);
            assert!(!plan.can_apply);
            assert!(plan.message.contains("does not support link planning"));
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
            .filter(|name| name.contains(".nako-") && name.ends_with(".tmp"))
            .collect::<Vec<_>>();

        assert_eq!(leftovers, Vec::<String>::new());
    }

    fn backup_files(path: &Path) -> Vec<String> {
        fs::read_dir(path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".nako-backup-"))
            .collect()
    }

    struct DirectOnlyBackend;

    #[async_trait::async_trait]
    impl StorageBackend for DirectOnlyBackend {
        fn scheme(&self) -> &'static str {
            "memory"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Err(NakoError::NotFound {
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
            Err(NakoError::Unsupported(
                "direct-only backend does not support opening files",
            ))
        }

        async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
            Err(NakoError::NotFound {
                entity: "storage_object",
                id: "memory".to_owned(),
            })
        }

        async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
            Ok(())
        }
    }
}
