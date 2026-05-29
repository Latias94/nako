use std::sync::Arc;

use nako_addon_protocol::{
    AddonLibraryFileRole, AddonLibraryFileWritePayload, AddonLibraryFileWritePolicy,
};
use nako_core::{
    AddonSideEffectId, AddonSideEffectRecord, AddonSideEffectTargetKind, Library, LibraryId,
    LibraryRepository, MediaItemId, MediaRepository, MediaSource, MediaSourceId, NakoError, Result,
    StorageErrorKind,
};
use nako_db::NakoDatabase;
use nako_nfo::{MovieNfoCodec, NfoExportSourceRequest, NfoExportSourceSummary, NfoFailureKind};
use nako_vfs::{
    StorageBackend as _, StorageBackupMode, StorageBackupPolicy, StorageUri, StorageWriteRequest,
};
use tokio::sync::Semaphore;

use super::{
    super::{
        nfo::ensure_nfo_export_writable, storage::StorageBackendRegistry,
        subtitle_sidecar::subtitle_sidecar_uri_for_source,
    },
    side_effect_apply::AddonSideEffectApplyCommand,
};

#[derive(Clone, Debug)]
pub(super) struct AddonLibraryFileWriteAdapter {
    runtime: LibraryFileWriteRuntime,
}

impl AddonLibraryFileWriteAdapter {
    pub(super) fn new(
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            runtime: LibraryFileWriteRuntime::new(store, permits, storage_backends),
        }
    }

    pub(super) async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectApplyCommand> {
        let outcome = self.runtime.apply(side_effect).await?;

        Ok(AddonSideEffectApplyCommand::applied(
            outcome.side_effect_id,
            outcome.item_id,
            outcome.applied_source,
            Some(outcome.report_json),
        ))
    }
}

#[derive(Clone, Debug)]
pub(super) struct LibraryFileWriteRuntime {
    store: NakoDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
}

impl LibraryFileWriteRuntime {
    pub(super) fn new(
        store: NakoDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            store,
            permits,
            storage_backends,
        }
    }

    async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonLibraryFileWriteOutcome> {
        let command = AddonLibraryFileWriteCommand::from_side_effect(side_effect)?;
        let target = self.resolve_target(&command).await?;

        let library = self
            .store
            .get_library(command.library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: command.library_id.to_string(),
            })?;
        let _permit = self.acquire_file_write_permit().await?;
        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        ensure_nfo_export_writable(backend.as_ref(), &library).await?;
        let service = nako_nfo::NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        let summary = service
            .export_media_source(NfoExportSourceRequest {
                library_id: command.library_id,
                source_id: target.source.id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: command.policy.force(),
            })
            .await?;

        if let Some(failure) = summary.failures.first() {
            return Err(nfo_export_failure_error(failure.kind));
        }

        Ok(AddonLibraryFileWriteOutcome {
            side_effect_id: command.side_effect_id,
            item_id: target.source.item_id,
            applied_source: "nfo_export",
            report_json: nfo_export_apply_report(&command, &target, &library, &summary)?,
        })
    }

    async fn resolve_target(
        &self,
        command: &AddonLibraryFileWriteCommand,
    ) -> Result<AddonLibraryFileWriteTarget> {
        match command.file_role {
            AddonLibraryFileRole::Nfo => self.resolve_nfo_target(command).await,
        }
    }

    async fn resolve_nfo_target(
        &self,
        command: &AddonLibraryFileWriteCommand,
    ) -> Result<AddonLibraryFileWriteTarget> {
        if command.target_kind != AddonSideEffectTargetKind::MediaSource {
            return Err(NakoError::InvalidInput {
                message: "addon library_file_write NFO export requires a media_source target"
                    .to_owned(),
            });
        }
        let source_id: MediaSourceId =
            command
                .target_id
                .parse()
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid addon library_file_write media source target: {err}"),
                })?;
        let source = self
            .store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })?;
        if source.library_id != command.library_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "addon library_file_write target media source {} is not in library {}",
                    source.id, command.library_id
                ),
            });
        }

        Ok(AddonLibraryFileWriteTarget { source })
    }

    async fn acquire_file_write_permit(&self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        self.permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| NakoError::InvalidInput {
                message: format!("Library File Write limiter is unavailable: {err}"),
            })
    }

    pub(super) async fn write_subtitle_sidecar(
        &self,
        request: SubtitleSidecarWriteRequest,
    ) -> Result<SubtitleSidecarWriteReport> {
        let library = self
            .store
            .get_library(request.library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: request.library_id.to_string(),
            })?;
        if request.source.library_id != library.id {
            return Err(NakoError::InvalidInput {
                message: "subtitle sidecar source is not in the target library".to_owned(),
            });
        }

        let (source_uri, backend) = self
            .storage_backends
            .backend_for_media_source(&request.source)
            .await?;
        let sidecar_uri = subtitle_sidecar_uri_for_source(&source_uri, &request.file_name)?;
        let _permit = self.acquire_file_write_permit().await?;
        let target_existed = storage_object_exists(backend.as_ref(), &sidecar_uri).await?;

        if target_existed {
            let existing = backend.read_to_string(&sidecar_uri).await?;
            if existing == request.content {
                return Ok(SubtitleSidecarWriteReport {
                    status: SubtitleSidecarWriteStatus::AlreadyApplied,
                    write_mode: "unchanged",
                    byte_len: request.content.len() as u64,
                    target_existed,
                    backup_created: false,
                });
            }
            if request.conflict_policy == SubtitleSidecarConflictPolicy::CreateMissing {
                return Err(NakoError::Conflict {
                    message: "subtitle sidecar already exists".to_owned(),
                });
            }
        }

        let write_mode = match request.conflict_policy {
            SubtitleSidecarConflictPolicy::CreateMissing => "create_missing",
            SubtitleSidecarConflictPolicy::ReplaceExisting => "atomic_replace",
        };
        let mut write = match request.conflict_policy {
            SubtitleSidecarConflictPolicy::CreateMissing => {
                StorageWriteRequest::direct(sidecar_uri, request.content)
            }
            SubtitleSidecarConflictPolicy::ReplaceExisting => {
                StorageWriteRequest::atomic_replace(sidecar_uri, request.content)
            }
        };
        if request.backup_policy == SubtitleSidecarBackupPolicy::ExistingFileKeepLatest {
            write = write.with_backup_policy(StorageBackupPolicy::existing_file().keep_latest(1));
        } else {
            write = write.with_backup(StorageBackupMode::None);
        }
        let byte_len = write.content.len() as u64;
        let report = backend.write(write).await?;

        Ok(SubtitleSidecarWriteReport {
            status: SubtitleSidecarWriteStatus::Applied,
            write_mode,
            byte_len,
            target_existed,
            backup_created: report.backup.is_some(),
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct SubtitleSidecarWriteRequest {
    pub(super) library_id: LibraryId,
    pub(super) source: MediaSource,
    pub(super) file_name: String,
    pub(super) content: String,
    pub(super) conflict_policy: SubtitleSidecarConflictPolicy,
    pub(super) backup_policy: SubtitleSidecarBackupPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubtitleSidecarConflictPolicy {
    CreateMissing,
    ReplaceExisting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubtitleSidecarBackupPolicy {
    None,
    ExistingFileKeepLatest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SubtitleSidecarWriteStatus {
    Applied,
    AlreadyApplied,
}

#[derive(Debug)]
pub(super) struct SubtitleSidecarWriteReport {
    pub(super) status: SubtitleSidecarWriteStatus,
    pub(super) write_mode: &'static str,
    pub(super) byte_len: u64,
    pub(super) target_existed: bool,
    pub(super) backup_created: bool,
}

#[derive(Clone, Debug)]
struct AddonLibraryFileWriteCommand {
    side_effect_id: AddonSideEffectId,
    library_id: LibraryId,
    target_kind: AddonSideEffectTargetKind,
    target_id: String,
    file_role: AddonLibraryFileRole,
    policy: AddonLibraryFileWritePolicy,
}

impl AddonLibraryFileWriteCommand {
    fn from_side_effect(side_effect: &AddonSideEffectRecord) -> Result<Self> {
        let payload = parse_addon_library_file_write_payload(&side_effect.payload_json)?;
        match payload.file_role {
            AddonLibraryFileRole::Nfo => {}
        }

        Ok(Self {
            side_effect_id: side_effect.id,
            library_id: side_effect.library_id,
            target_kind: side_effect.target.kind,
            target_id: side_effect.target.id.clone(),
            file_role: payload.file_role,
            policy: payload.policy,
        })
    }
}

#[derive(Clone, Debug)]
struct AddonLibraryFileWriteTarget {
    source: MediaSource,
}

#[derive(Debug)]
struct AddonLibraryFileWriteOutcome {
    side_effect_id: AddonSideEffectId,
    item_id: MediaItemId,
    applied_source: &'static str,
    report_json: String,
}

trait AddonLibraryFileWritePolicyExt {
    fn force(self) -> bool;
}

impl AddonLibraryFileWritePolicyExt for AddonLibraryFileWritePolicy {
    fn force(self) -> bool {
        match self {
            Self::CreateMissing => false,
            Self::ReplaceExistingPreserving => true,
        }
    }
}

fn parse_addon_library_file_write_payload(
    payload_json: &str,
) -> Result<AddonLibraryFileWritePayload> {
    serde_json::from_str::<AddonLibraryFileWritePayload>(payload_json).map_err(|err| {
        NakoError::InvalidInput {
            message: format!("invalid addon library_file_write payload: {err}"),
        }
    })
}

fn nfo_export_apply_report(
    command: &AddonLibraryFileWriteCommand,
    target: &AddonLibraryFileWriteTarget,
    library: &Library,
    summary: &NfoExportSourceSummary,
) -> Result<String> {
    let report = serde_json::json!({
        "kind": "nfo_export",
        "target_kind": AddonSideEffectTargetKind::MediaSource.as_str(),
        "file_role": command.file_role.as_str(),
        "policy": command.policy.as_str(),
        "write_mode": nfo_export_write_mode(command.policy),
        "backup_policy": nfo_export_backup_policy(command.policy),
        "library_id": library.id.to_string(),
        "source_id": target.source.id.to_string(),
        "item_id": target.source.item_id.to_string(),
        "scanned_sources": summary.scanned_sources,
        "exported_items": summary.exported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "backed_up_items": summary.backed_up_items,
        "pruned_backup_items": summary.pruned_backup_items,
        "pruned_backups": summary.pruned_backups,
        "prune_failures": summary.prune_failures.len(),
    });

    serde_json::to_string(&report).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to serialize addon NFO export report: {err}"),
    })
}

fn nfo_export_write_mode(policy: AddonLibraryFileWritePolicy) -> &'static str {
    match policy {
        AddonLibraryFileWritePolicy::CreateMissing => "create_missing",
        AddonLibraryFileWritePolicy::ReplaceExistingPreserving => "atomic_replace",
    }
}

fn nfo_export_backup_policy(policy: AddonLibraryFileWritePolicy) -> &'static str {
    match policy {
        AddonLibraryFileWritePolicy::CreateMissing => "none",
        AddonLibraryFileWritePolicy::ReplaceExistingPreserving => "existing_file_keep_latest",
    }
}

fn nfo_export_failure_error(kind: NfoFailureKind) -> NakoError {
    match kind {
        NfoFailureKind::StorageRead | NfoFailureKind::StorageWrite => NakoError::storage(
            "nfo_export",
            StorageErrorKind::Unknown,
            "NFO export storage operation failed",
        ),
        NfoFailureKind::StorageBackup => NakoError::storage(
            "nfo_export",
            StorageErrorKind::Backup,
            "NFO export backup failed",
        ),
        NfoFailureKind::StorageUnsupported => {
            NakoError::Unsupported("NFO export storage backend is unsupported")
        }
        NfoFailureKind::MissingMediaItem => NakoError::NotFound {
            entity: "media_item",
            id: "nfo_export_target".to_owned(),
        },
        NfoFailureKind::NfoConflict => NakoError::Conflict {
            message: "NFO export preservation conflict".to_owned(),
        },
        NfoFailureKind::InvalidSidecarPath
        | NfoFailureKind::NfoParse
        | NfoFailureKind::NfoPreservation
        | NfoFailureKind::NfoRender
        | NfoFailureKind::Unknown => NakoError::InvalidInput {
            message: format!("NFO export failed: {kind:?}"),
        },
    }
}

async fn storage_object_exists(
    backend: &dyn nako_vfs::StorageBackend,
    uri: &StorageUri,
) -> Result<bool> {
    match backend.stat(uri).await {
        Ok(_) => Ok(true),
        Err(NakoError::NotFound { .. }) => Ok(false),
        Err(err) => Err(err),
    }
}
