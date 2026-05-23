use std::{fs, io::ErrorKind};

use nako_core::{NakoError, Result};

use super::{LocalFsBackend, path_authority, write_transaction};
use crate::{
    ObjectKind, ObjectMetadata, StorageApplyObject, StorageCleanupReport, StorageCleanupRequest,
    StorageCleanupStatus, StorageRestoreReport, StorageRestoreRequest, StorageRestoreStatus,
};

pub(super) fn cleanup_local(
    backend: &LocalFsBackend,
    request: StorageCleanupRequest,
) -> Result<StorageCleanupReport> {
    if let Err(err) = backend.ensure_local_scheme(&request.target_uri) {
        return Ok(cleanup_request_error_report(
            request,
            err,
            "cleanup target uses an unsupported storage scheme",
        ));
    }

    let target_path = match backend.cleanup_path_for(&request.target_uri) {
        Ok(path) => path,
        Err(NakoError::NotFound { .. }) => {
            return Ok(cleanup_report(
                request,
                StorageCleanupStatus::TargetMissing,
                false,
                None,
                "cleanup target is already missing",
            ));
        }
        Err(err) if path_authority::is_security_violation(&err) => {
            return Ok(cleanup_report(
                request,
                StorageCleanupStatus::SecurityViolation,
                false,
                None,
                "cleanup target escaped the local backend root",
            ));
        }
        Err(err) => return Err(err),
    };
    let target = match backend.metadata_for(&target_path, request.target_uri.clone()) {
        Ok(target) => target,
        Err(NakoError::NotFound { .. }) => {
            return Ok(cleanup_report(
                request,
                StorageCleanupStatus::TargetMissing,
                false,
                None,
                "cleanup target is already missing",
            ));
        }
        Err(err) => return Err(err),
    };

    if !matches!(target.kind, ObjectKind::File | ObjectKind::Symlink) {
        return Ok(cleanup_report(
            request,
            StorageCleanupStatus::TargetNotFile,
            false,
            Some(target),
            "cleanup target is not a file object",
        ));
    }

    match fs::remove_file(&target_path) {
        Ok(()) => {
            if let Some(parent) = target_path.parent() {
                write_transaction::sync_directory_if_possible(parent);
            }
            Ok(cleanup_report(
                request,
                StorageCleanupStatus::Cleaned,
                true,
                Some(target),
                "storage target cleanup completed by the local backend",
            ))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(cleanup_report(
            request,
            StorageCleanupStatus::TargetMissing,
            false,
            Some(target),
            "cleanup target is already missing",
        )),
        Err(_err) => Ok(cleanup_report(
            request,
            StorageCleanupStatus::CleanupFailed,
            false,
            Some(target),
            "storage target cleanup failed in the local backend",
        )),
    }
}

pub(super) fn restore_local(
    backend: &LocalFsBackend,
    request: StorageRestoreRequest,
) -> Result<StorageRestoreReport> {
    if let Err(err) = backend.ensure_local_scheme(&request.backup_uri) {
        return Ok(restore_request_error_report(
            request,
            err,
            "restore backup uses an unsupported storage scheme",
        ));
    }
    if let Err(err) = backend.ensure_local_scheme(&request.target_uri) {
        return Ok(restore_request_error_report(
            request,
            err,
            "restore target uses an unsupported storage scheme",
        ));
    }

    let backup_path = match backend.path_for(&request.backup_uri) {
        Ok(path) => path,
        Err(NakoError::NotFound { .. }) => {
            return Ok(restore_report(
                request,
                StorageRestoreStatus::BackupMissing,
                false,
                None,
                None,
                "restore backup is missing",
            ));
        }
        Err(err) if path_authority::is_security_violation(&err) => {
            return Ok(restore_request_error_report(
                request,
                err,
                "restore backup escaped the local backend root",
            ));
        }
        Err(err) => return Err(err),
    };
    let target_path = match backend.writable_path_for(&request.target_uri) {
        Ok(path) => path,
        Err(NakoError::NotFound { .. }) => {
            return Ok(restore_report(
                request,
                StorageRestoreStatus::TargetParentMissing,
                false,
                None,
                None,
                "restore target parent is missing",
            ));
        }
        Err(err) if path_authority::is_security_violation(&err) => {
            return Ok(restore_request_error_report(
                request,
                err,
                "restore target escaped the local backend root",
            ));
        }
        Err(err) => return Err(err),
    };
    let backup = match backend.metadata_for(&backup_path, request.backup_uri.clone()) {
        Ok(metadata) => metadata,
        Err(NakoError::NotFound { .. }) => {
            return Ok(restore_report(
                request,
                StorageRestoreStatus::BackupMissing,
                false,
                None,
                None,
                "restore backup is missing",
            ));
        }
        Err(err) => return Err(err),
    };
    if backup.kind != ObjectKind::File {
        return Ok(restore_report(
            request,
            StorageRestoreStatus::BackupNotFile,
            false,
            Some(StorageApplyObject::from_metadata(backup)),
            None,
            "restore backup is not a file object",
        ));
    }
    let Some(parent) = target_path.parent() else {
        return Ok(restore_report(
            request,
            StorageRestoreStatus::SecurityViolation,
            false,
            Some(StorageApplyObject::from_metadata(backup)),
            None,
            "restore target has no parent directory",
        ));
    };
    let parent_metadata = match fs::metadata(parent) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(restore_report(
                request,
                StorageRestoreStatus::TargetParentMissing,
                false,
                Some(StorageApplyObject::from_metadata(backup)),
                None,
                "restore target parent is missing",
            ));
        }
        Err(err) => {
            return Err(NakoError::storage_io(
                request.target_uri.to_string(),
                format!("failed to read local restore target parent metadata: {err}"),
            ));
        }
    };
    if !parent_metadata.is_dir() {
        return Ok(restore_report(
            request,
            StorageRestoreStatus::TargetParentNotDirectory,
            false,
            Some(StorageApplyObject::from_metadata(backup)),
            None,
            "restore target parent is not a directory",
        ));
    }

    let restore_result =
        write_transaction::restore_file_atomically(&request.target_uri, &backup_path, &target_path);
    match restore_result {
        Ok(_) => {
            write_transaction::sync_directory_if_possible(parent);
            let target = backend.metadata_for(&target_path, request.target_uri.clone())?;
            Ok(restore_report(
                request,
                StorageRestoreStatus::Restored,
                true,
                Some(StorageApplyObject::from_metadata(backup)),
                Some(StorageApplyObject::from_metadata(target)),
                "storage target restored from backup by the local backend",
            ))
        }
        Err(_err) => Ok(restore_report(
            request.clone(),
            StorageRestoreStatus::RestoreFailed,
            false,
            Some(StorageApplyObject::from_metadata(backup)),
            backend
                .metadata_for(&target_path, request.target_uri.clone())
                .map(StorageApplyObject::from_metadata)
                .ok(),
            "storage restore failed in the local backend",
        )),
    }
}

fn cleanup_report(
    request: StorageCleanupRequest,
    status: StorageCleanupStatus,
    cleaned: bool,
    target: Option<ObjectMetadata>,
    message: impl Into<String>,
) -> StorageCleanupReport {
    StorageCleanupReport {
        target_uri: request.target_uri,
        status,
        cleaned,
        target: target.map(StorageApplyObject::from_metadata),
        message: message.into(),
    }
}

fn cleanup_request_error_report(
    request: StorageCleanupRequest,
    err: NakoError,
    fallback_message: &'static str,
) -> StorageCleanupReport {
    match err {
        err if path_authority::is_security_violation(&err) => cleanup_report(
            request,
            StorageCleanupStatus::SecurityViolation,
            false,
            None,
            "storage cleanup request escaped the backend root",
        ),
        NakoError::InvalidInput { .. } => cleanup_report(
            request,
            StorageCleanupStatus::Unsupported,
            false,
            None,
            fallback_message,
        ),
        _ => cleanup_report(
            request,
            StorageCleanupStatus::CleanupFailed,
            false,
            None,
            "storage cleanup request could not be validated",
        ),
    }
}

fn restore_report(
    request: StorageRestoreRequest,
    status: StorageRestoreStatus,
    restored: bool,
    backup: Option<StorageApplyObject>,
    target: Option<StorageApplyObject>,
    message: impl Into<String>,
) -> StorageRestoreReport {
    StorageRestoreReport {
        backup_uri: request.backup_uri,
        target_uri: request.target_uri,
        status,
        restored,
        backup,
        target,
        message: message.into(),
    }
}

fn restore_request_error_report(
    request: StorageRestoreRequest,
    err: NakoError,
    fallback_message: &'static str,
) -> StorageRestoreReport {
    match err {
        err if path_authority::is_security_violation(&err) => restore_report(
            request,
            StorageRestoreStatus::SecurityViolation,
            false,
            None,
            None,
            "storage restore request escaped the backend root",
        ),
        NakoError::InvalidInput { .. } => restore_report(
            request,
            StorageRestoreStatus::Unsupported,
            false,
            None,
            None,
            fallback_message,
        ),
        _ => restore_report(
            request,
            StorageRestoreStatus::RestoreFailed,
            false,
            None,
            None,
            "storage restore request could not be validated",
        ),
    }
}
