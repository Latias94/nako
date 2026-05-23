use std::{fs, io::ErrorKind, path::Path};

use nako_core::{NakoError, Result, StorageErrorKind};

use super::{LocalFsBackend, path_authority, write_transaction};
use crate::{
    ObjectKind, ObjectMetadata, StorageApplyKind, StorageApplyObject, StorageApplyReport,
    StorageApplyRequest, StorageApplyStatus, StorageLinkKind, StorageLinkPlan,
    StorageLinkPlanRequest, StorageLinkPlanStatus,
};

pub(super) fn plan_local_link(
    backend: &LocalFsBackend,
    request: StorageLinkPlanRequest,
) -> Result<StorageLinkPlan> {
    let source_path = match backend.path_for(&request.source_uri) {
        Ok(path) => path,
        Err(NakoError::NotFound { .. }) => {
            return Ok(link_plan(
                request,
                StorageLinkPlanStatus::SourceMissing,
                None,
                None,
                "link source does not exist",
            ));
        }
        Err(err) if path_authority::is_security_violation(&err) => {
            return Ok(link_plan(
                request,
                StorageLinkPlanStatus::SecurityViolation,
                None,
                None,
                "link source escaped the local backend root",
            ));
        }
        Err(err) => return Err(err),
    };
    let source = backend.metadata_for(&source_path, request.source_uri.clone())?;
    if source.kind != ObjectKind::File {
        return Ok(link_plan(
            request,
            StorageLinkPlanStatus::SourceNotFile,
            Some(source),
            None,
            "link source is not a regular file",
        ));
    }

    backend.ensure_local_scheme(&request.target_uri)?;
    let target_relative = match path_authority::relative_path(&request.target_uri) {
        Ok(relative) => relative,
        Err(err) if path_authority::is_security_violation(&err) => {
            return Ok(link_plan(
                request,
                StorageLinkPlanStatus::SecurityViolation,
                Some(source),
                None,
                "link target escaped the local backend root",
            ));
        }
        Err(err) => return Err(err),
    };
    let target_path = backend.root().join(target_relative);
    let parent = target_path.parent().ok_or_else(|| {
        NakoError::storage(
            request.target_uri.to_string(),
            StorageErrorKind::SecurityViolation,
            "local link target has no parent directory",
        )
    })?;
    if !parent.exists() {
        return Ok(link_plan(
            request,
            StorageLinkPlanStatus::TargetParentMissing,
            Some(source),
            None,
            "link target parent does not exist",
        ));
    }
    let canonical_parent = parent.canonicalize().map_err(|err| {
        NakoError::storage_io(
            request.target_uri.to_string(),
            format!("failed to resolve local link target parent: {err}"),
        )
    })?;
    if !canonical_parent.starts_with(backend.root()) {
        return Ok(link_plan(
            request,
            StorageLinkPlanStatus::SecurityViolation,
            Some(source),
            None,
            "link target escaped the local backend root",
        ));
    }
    if !parent.is_dir() {
        return Ok(link_plan(
            request,
            StorageLinkPlanStatus::TargetParentNotDirectory,
            Some(source),
            None,
            "link target parent is not a directory",
        ));
    }
    if target_path.exists() || fs::symlink_metadata(&target_path).is_ok() {
        let target = backend
            .metadata_for(&target_path, request.target_uri.clone())
            .ok();
        return Ok(link_plan(
            request,
            StorageLinkPlanStatus::TargetExists,
            Some(source),
            target,
            "link target already exists",
        ));
    }

    Ok(link_plan(
        request,
        StorageLinkPlanStatus::Ready,
        Some(source),
        None,
        "link can be applied by the local backend",
    ))
}

pub(super) fn apply_local(
    backend: &LocalFsBackend,
    request: StorageApplyRequest,
) -> Result<StorageApplyReport> {
    if let Err(err) = backend.ensure_local_scheme(&request.source_uri) {
        return Ok(apply_request_error_report(
            request,
            err,
            "apply source uses an unsupported storage scheme",
        ));
    }
    if let Err(err) = backend.ensure_local_scheme(&request.target_uri) {
        return Ok(apply_request_error_report(
            request,
            err,
            "apply target uses an unsupported storage scheme",
        ));
    }

    match request.kind {
        StorageApplyKind::Copy => apply_local_copy(backend, request),
        StorageApplyKind::Hardlink | StorageApplyKind::Symlink => {
            apply_local_link(backend, request)
        }
    }
}

fn apply_local_copy(
    backend: &LocalFsBackend,
    request: StorageApplyRequest,
) -> Result<StorageApplyReport> {
    let plan = plan_local_copy(backend, &request)?;
    if !plan.can_apply {
        return Ok(report_from_plan(
            request,
            apply_status_from_link_status(plan.status),
            false,
            false,
            plan.source,
            plan.target,
            plan.message,
        ));
    }

    let source = plan.source.clone();
    let source_path = backend.path_for(&request.source_uri)?;
    let target_path = backend
        .root()
        .join(path_authority::relative_path(&request.target_uri)?);
    match copy_file_create_new(&source_path, &target_path) {
        Ok(()) => {
            if let Some(parent) = target_path.parent() {
                write_transaction::sync_directory_if_possible(parent);
            }
            let target = backend.metadata_for(&target_path, request.target_uri.clone())?;
            Ok(report_from_plan(
                request,
                StorageApplyStatus::Applied,
                true,
                true,
                plan.source,
                Some(target),
                "copy applied by the local backend",
            ))
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            let target = backend
                .metadata_for(&target_path, request.target_uri.clone())
                .ok();
            Ok(report_from_plan(
                request,
                StorageApplyStatus::TargetExists,
                false,
                false,
                plan.source,
                target,
                "copy target already exists",
            ))
        }
        Err(_err) => Ok(report_from_plan(
            request,
            StorageApplyStatus::ApplyFailed,
            false,
            false,
            source,
            None,
            "copy failed in the local backend",
        )),
    }
}

fn apply_local_link(
    backend: &LocalFsBackend,
    request: StorageApplyRequest,
) -> Result<StorageApplyReport> {
    let link_kind = match request.kind {
        StorageApplyKind::Hardlink => StorageLinkKind::Hard,
        StorageApplyKind::Symlink => StorageLinkKind::Soft,
        StorageApplyKind::Copy => {
            return Ok(report_from_plan(
                request,
                StorageApplyStatus::Unsupported,
                false,
                false,
                None,
                None,
                "copy is not a link operation",
            ));
        }
    };
    let plan = plan_local_link(
        backend,
        StorageLinkPlanRequest::new(
            request.source_uri.clone(),
            request.target_uri.clone(),
            link_kind,
        ),
    )?;
    if !plan.can_apply {
        return Ok(report_from_plan(
            request,
            apply_status_from_link_status(plan.status),
            false,
            false,
            plan.source,
            plan.target,
            plan.message,
        ));
    }

    let source = plan.source.clone();
    let source_path = backend.path_for(&request.source_uri)?;
    let target_path = backend
        .root()
        .join(path_authority::relative_path(&request.target_uri)?);
    let apply_result = match link_kind {
        StorageLinkKind::Hard => fs::hard_link(&source_path, &target_path),
        StorageLinkKind::Soft => create_file_symlink(&source_path, &target_path),
    };

    match apply_result {
        Ok(()) => {
            if let Some(parent) = target_path.parent() {
                write_transaction::sync_directory_if_possible(parent);
            }
            let target = backend.metadata_for(&target_path, request.target_uri.clone())?;
            Ok(report_from_plan(
                request,
                StorageApplyStatus::Applied,
                true,
                true,
                plan.source,
                Some(target),
                "link applied by the local backend",
            ))
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            let target = backend
                .metadata_for(&target_path, request.target_uri.clone())
                .ok();
            Ok(report_from_plan(
                request,
                StorageApplyStatus::TargetExists,
                false,
                false,
                plan.source,
                target,
                "link target already exists",
            ))
        }
        Err(_err) => Ok(report_from_plan(
            request,
            StorageApplyStatus::ApplyFailed,
            false,
            false,
            source,
            None,
            "link failed in the local backend",
        )),
    }
}

fn plan_local_copy(
    backend: &LocalFsBackend,
    request: &StorageApplyRequest,
) -> Result<StorageLinkPlan> {
    plan_local_link(
        backend,
        StorageLinkPlanRequest::new(
            request.source_uri.clone(),
            request.target_uri.clone(),
            StorageLinkKind::Hard,
        ),
    )
    .map(|mut plan| {
        if plan.status == StorageLinkPlanStatus::Ready {
            plan.message = "copy can be applied by the local backend".to_owned();
        }
        plan
    })
}

fn link_plan(
    request: StorageLinkPlanRequest,
    status: StorageLinkPlanStatus,
    source: Option<ObjectMetadata>,
    target: Option<ObjectMetadata>,
    message: impl Into<String>,
) -> StorageLinkPlan {
    StorageLinkPlan {
        source_uri: request.source_uri,
        target_uri: request.target_uri,
        kind: request.kind,
        status,
        can_apply: status == StorageLinkPlanStatus::Ready,
        source,
        target,
        message: message.into(),
    }
}

fn report_from_plan(
    request: StorageApplyRequest,
    status: StorageApplyStatus,
    applied: bool,
    target_created: bool,
    source: Option<ObjectMetadata>,
    target: Option<ObjectMetadata>,
    message: impl Into<String>,
) -> StorageApplyReport {
    StorageApplyReport {
        source_uri: request.source_uri,
        target_uri: request.target_uri,
        kind: request.kind,
        status,
        applied,
        target_created,
        source: source.map(StorageApplyObject::from_metadata),
        target: target.map(StorageApplyObject::from_metadata),
        message: message.into(),
    }
}

fn plan_error_report(
    request: StorageApplyRequest,
    status: StorageApplyStatus,
    message: impl Into<String>,
) -> StorageApplyReport {
    report_from_plan(request, status, false, false, None, None, message)
}

fn apply_request_error_report(
    request: StorageApplyRequest,
    err: NakoError,
    fallback_message: &'static str,
) -> StorageApplyReport {
    match err {
        NakoError::InvalidInput { .. } => {
            plan_error_report(request, StorageApplyStatus::Unsupported, fallback_message)
        }
        NakoError::Storage {
            kind: StorageErrorKind::SecurityViolation,
            ..
        } => plan_error_report(
            request,
            StorageApplyStatus::SecurityViolation,
            "storage apply request escaped the backend root",
        ),
        _ => plan_error_report(
            request,
            StorageApplyStatus::ApplyFailed,
            "storage apply request could not be validated",
        ),
    }
}

fn apply_status_from_link_status(status: StorageLinkPlanStatus) -> StorageApplyStatus {
    match status {
        StorageLinkPlanStatus::Ready => StorageApplyStatus::Applied,
        StorageLinkPlanStatus::Unsupported => StorageApplyStatus::Unsupported,
        StorageLinkPlanStatus::SourceMissing => StorageApplyStatus::SourceMissing,
        StorageLinkPlanStatus::SourceNotFile => StorageApplyStatus::SourceNotFile,
        StorageLinkPlanStatus::TargetParentMissing => StorageApplyStatus::TargetParentMissing,
        StorageLinkPlanStatus::TargetParentNotDirectory => {
            StorageApplyStatus::TargetParentNotDirectory
        }
        StorageLinkPlanStatus::TargetExists => StorageApplyStatus::TargetExists,
        StorageLinkPlanStatus::SecurityViolation => StorageApplyStatus::SecurityViolation,
    }
}

#[cfg(windows)]
fn create_file_symlink(source_path: &Path, target_path: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source_path, target_path)
}

#[cfg(unix)]
fn create_file_symlink(source_path: &Path, target_path: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source_path, target_path)
}

fn copy_file_create_new(source_path: &Path, target_path: &Path) -> std::io::Result<()> {
    let mut source = fs::File::open(source_path)?;
    let mut target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target_path)?;
    let copy_result = std::io::copy(&mut source, &mut target)
        .and_then(|_| target.sync_all())
        .map(|_| ());

    if copy_result.is_err() {
        let _ = fs::remove_file(target_path);
    }

    copy_result
}
