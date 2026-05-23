use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use nako_core::{NakoError, Result, StorageErrorKind};

use super::path_authority;
use crate::{
    StorageBackupMode, StorageBackupPolicy, StorageBackupPruneFailure, StorageBackupReport,
    StorageUri,
};

pub(super) fn write_string_atomic_replace(
    root: &Path,
    uri: &StorageUri,
    path: &Path,
    content: &str,
    backup: &StorageBackupPolicy,
) -> Result<(bool, Option<StorageBackupReport>)> {
    let parent = path.parent().ok_or_else(|| {
        NakoError::storage(
            uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "local atomic write target has no parent directory",
        )
    })?;
    let temp_path = atomic_temp_path(path);
    let write_result = (|| -> Result<(bool, Option<StorageBackupReport>)> {
        let backup = backup_for_path(root, uri, path, backup)?;
        {
            let mut file = fs::File::create(&temp_path).map_err(|err| {
                NakoError::storage_io(
                    temp_path.display().to_string(),
                    format!("failed to create local atomic temp file: {err}"),
                )
            })?;
            use std::io::Write as _;
            file.write_all(content.as_bytes()).map_err(|err| {
                NakoError::storage_io(
                    temp_path.display().to_string(),
                    format!("failed to write local atomic temp file: {err}"),
                )
            })?;
            file.sync_all().map_err(|err| {
                NakoError::storage_io(
                    temp_path.display().to_string(),
                    format!("failed to sync local atomic temp file: {err}"),
                )
            })?;
        }

        let atomic = replace_temp_file(uri, &temp_path, path)?;

        sync_directory_if_possible(parent);

        Ok((atomic, backup))
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    write_result
}

pub(super) fn backup_for_path(
    root: &Path,
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
                NakoError::storage_backup(
                    uri.to_string(),
                    format!("failed to create local backup before write: {err}"),
                )
            })?;
            sync_file_if_possible(&backup_path);
            if let Some(parent) = backup_path.parent() {
                sync_directory_if_possible(parent);
            }
            let (pruned_backups, prune_failures) =
                prune_backups_for_path(root, path, backup.retention.keep_latest)?;
            Ok(Some(StorageBackupReport {
                original_uri: uri.clone(),
                backup_uri: path_authority::backup_uri_for_path(root, &backup_path)?,
                pruned_backups,
                prune_failures,
            }))
        }
    }
}

fn prune_backups_for_path(
    root: &Path,
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
        NakoError::storage_backup(
            path.display().to_string(),
            format!("failed to list local backup directory for pruning: {err}"),
        )
    })? {
        let entry = entry.map_err(|err| {
            NakoError::storage_backup(
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
        let uri = path_authority::backup_uri_for_path(root, &candidate)?;
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

pub(super) fn restore_file_atomically(
    uri: &StorageUri,
    backup_path: &Path,
    target_path: &Path,
) -> Result<bool> {
    let temp_path = atomic_temp_path(target_path);
    let restore_result = (|| -> Result<bool> {
        fs::copy(backup_path, &temp_path).map_err(|err| {
            NakoError::storage_io(
                uri.to_string(),
                format!("failed to copy local backup into restore temp file: {err}"),
            )
        })?;
        sync_file_if_possible(&temp_path);
        replace_temp_file(uri, &temp_path, target_path)
    })();

    if restore_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    restore_result
}

#[cfg(not(windows))]
fn replace_temp_file(uri: &StorageUri, temp_path: &Path, path: &Path) -> Result<bool> {
    fs::rename(temp_path, path).map_err(|err| {
        NakoError::storage_io(
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
                NakoError::storage_io(
                    uri.to_string(),
                    format!(
                        "failed to remove existing local file after atomic replace was unavailable: {err}"
                    ),
                )
            })?;
            fs::rename(temp_path, path).map_err(|err| {
                NakoError::storage_io(
                    uri.to_string(),
                    format!(
                        "failed to replace local file after atomic replace was unavailable: {err}"
                    ),
                )
            })?;
            let _ = rename_err;
            Ok(false)
        }
        Err(err) => Err(NakoError::storage_io(
            uri.to_string(),
            format!("failed to replace local file atomically: {err}"),
        )),
    }
}

fn atomic_temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("nako-write");
    let nonce = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let process_id = std::process::id();
    path.with_file_name(format!(".{file_name}.nako-{process_id}-{nonce}.tmp"))
}

fn local_backup_path(path: &Path) -> PathBuf {
    let file_name = local_sidecar_file_name(path);
    let nonce = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    path.with_file_name(format!("{file_name}.nako-backup-{nonce}"))
}

fn local_backup_file_prefix(path: &Path) -> String {
    format!("{}.nako-backup-", local_sidecar_file_name(path))
}

fn local_sidecar_file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("nako-sidecar")
        .to_owned()
}

fn sync_file_if_possible(path: &Path) {
    if let Ok(file) = fs::File::open(path) {
        let _ = file.sync_all();
    }
}

pub(super) fn sync_directory_if_possible(path: &Path) {
    if let Ok(directory) = fs::File::open(path) {
        let _ = directory.sync_all();
    }
}
