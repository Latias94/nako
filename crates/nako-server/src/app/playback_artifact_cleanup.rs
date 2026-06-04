use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use nako_core::{
    NakoError, PageRequest, Result, TranscodeSessionKind, TranscodeSessionListFilter,
    TranscodeSessionRecord, TranscodeSessionRepository, TranscodeSessionState,
};

use super::startup::ServerStartupPlaybackArtifactCleanupReport;

pub(super) async fn cleanup_expired_playback_artifacts(
    store: &nako_db::NakoDatabase,
    artifact_root: &Path,
    retention_ms: u64,
    now_ms: i64,
) -> Result<ServerStartupPlaybackArtifactCleanupReport> {
    let Ok(root) = artifact_root.canonicalize() else {
        return Ok(ServerStartupPlaybackArtifactCleanupReport::default());
    };

    let mut report = ServerStartupPlaybackArtifactCleanupReport::default();
    for state in [
        TranscodeSessionState::Finished,
        TranscodeSessionState::Failed,
        TranscodeSessionState::Cancelled,
    ] {
        let mut offset = 0;
        loop {
            let sessions = store
                .list_transcode_sessions(
                    TranscodeSessionListFilter {
                        source_id: None,
                        kind: None,
                        state: Some(state),
                    },
                    PageRequest::new(PageRequest::MAX_LIMIT, offset),
                )
                .await?;
            if sessions.is_empty() {
                break;
            }
            offset += sessions.len() as u64;

            for session in sessions {
                cleanup_expired_playback_artifact(
                    &session,
                    &root,
                    retention_ms,
                    now_ms,
                    &mut report,
                )?;
            }
        }
    }

    Ok(report)
}

fn cleanup_expired_playback_artifact(
    session: &TranscodeSessionRecord,
    root: &Path,
    retention_ms: u64,
    now_ms: i64,
    report: &mut ServerStartupPlaybackArtifactCleanupReport,
) -> Result<()> {
    report.examined_artifacts = report.examined_artifacts.saturating_add(1);

    let Some(target) = playback_artifact_target(session) else {
        return Ok(());
    };
    if !target.exists() {
        return Ok(());
    }
    let target = target.canonicalize().map_err(|err| {
        NakoError::storage_io(
            target.display().to_string(),
            format!("failed to resolve playback artifact target: {err}"),
        )
    })?;
    if !target.starts_with(root) {
        report.skipped_security = report.skipped_security.saturating_add(1);
        return Ok(());
    }

    let metadata = fs::symlink_metadata(&target).map_err(|err| {
        NakoError::storage_io(
            target.display().to_string(),
            format!("failed to inspect playback artifact target: {err}"),
        )
    })?;
    let Some(modified_at_ms) = modified_at_ms(&metadata) else {
        return Ok(());
    };
    let retention_ms = i64::try_from(retention_ms).unwrap_or(i64::MAX);
    if now_ms.saturating_sub(modified_at_ms) < retention_ms {
        return Ok(());
    }

    let summary = summarize_artifact_path(&target)?;
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(&target).map_err(|err| {
            NakoError::storage_io(
                target.display().to_string(),
                format!("failed to remove playback artifact directory: {err}"),
            )
        })?;
    } else {
        fs::remove_file(&target).map_err(|err| {
            NakoError::storage_io(
                target.display().to_string(),
                format!("failed to remove playback artifact file: {err}"),
            )
        })?;
    }

    report.deleted_artifacts = report.deleted_artifacts.saturating_add(1);
    report.deleted_files = report.deleted_files.saturating_add(summary.files);
    report.deleted_directories = report
        .deleted_directories
        .saturating_add(summary.directories);
    report.deleted_bytes = report.deleted_bytes.saturating_add(summary.bytes);

    Ok(())
}

fn playback_artifact_target(session: &TranscodeSessionRecord) -> Option<PathBuf> {
    match session.kind {
        TranscodeSessionKind::Remux => Some(session.output_path.clone()),
        TranscodeSessionKind::HlsTranscode => session.output_path.parent().map(Path::to_path_buf),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ArtifactPathSummary {
    files: u32,
    directories: u32,
    bytes: u64,
}

fn summarize_artifact_path(path: &Path) -> Result<ArtifactPathSummary> {
    let metadata = fs::symlink_metadata(path).map_err(|err| {
        NakoError::storage_io(
            path.display().to_string(),
            format!("failed to inspect playback artifact path: {err}"),
        )
    })?;
    if metadata.file_type().is_dir() {
        let mut summary = ArtifactPathSummary {
            files: 0,
            directories: 1,
            bytes: 0,
        };
        for entry in fs::read_dir(path).map_err(|err| {
            NakoError::storage_io(
                path.display().to_string(),
                format!("failed to list playback artifact directory: {err}"),
            )
        })? {
            let entry = entry.map_err(|err| {
                NakoError::storage_io(
                    path.display().to_string(),
                    format!("failed to read playback artifact directory entry: {err}"),
                )
            })?;
            let child = summarize_artifact_path(&entry.path())?;
            summary.files = summary.files.saturating_add(child.files);
            summary.directories = summary.directories.saturating_add(child.directories);
            summary.bytes = summary.bytes.saturating_add(child.bytes);
        }
        Ok(summary)
    } else {
        Ok(ArtifactPathSummary {
            files: 1,
            directories: 0,
            bytes: metadata.len(),
        })
    }
}

fn modified_at_ms(metadata: &fs::Metadata) -> Option<i64> {
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    i64::try_from(duration.as_millis()).ok()
}
