use std::sync::Arc;

use taru_core::{
    AddonSideEffectRecord, AddonSideEffectTargetKind, LibraryRepository, MediaItemId,
    MediaRepository, MediaSourceId, Result, StorageErrorKind, TaruError,
};
use taru_db::TaruDatabase;
use taru_nfo::{MovieNfoCodec, NfoExportSourceRequest, NfoExportSourceSummary, NfoFailureKind};
use tokio::sync::Semaphore;

use super::super::{nfo::ensure_nfo_export_writable, storage::StorageBackendRegistry};

#[derive(Clone, Debug)]
pub(super) struct AddonLibraryFileWriteAdapter {
    store: TaruDatabase,
    permits: Arc<Semaphore>,
    storage_backends: StorageBackendRegistry,
}

impl AddonLibraryFileWriteAdapter {
    pub(super) fn new(
        store: TaruDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            store,
            permits,
            storage_backends,
        }
    }

    pub(super) async fn apply(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AppliedLibraryFileWrite> {
        let payload = parse_addon_library_file_write_payload(&side_effect.payload_json)?;
        match payload.file_role {
            AddonLibraryFileRole::Nfo => {}
        }
        if side_effect.target.kind != AddonSideEffectTargetKind::MediaSource {
            return Err(TaruError::InvalidInput {
                message: "addon library_file_write NFO export requires a media_source target"
                    .to_owned(),
            });
        }
        let source_id: MediaSourceId =
            side_effect
                .target
                .id
                .parse()
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("invalid addon library_file_write media source target: {err}"),
                })?;
        let source = self
            .store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })?;
        if source.library_id != side_effect.library_id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "addon library_file_write target media source {} is not in library {}",
                    source.id, side_effect.library_id
                ),
            });
        }

        let library = self
            .store
            .get_library(side_effect.library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: side_effect.library_id.to_string(),
            })?;
        let _permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| TaruError::InvalidInput {
                    message: format!("NFO export limiter is unavailable: {err}"),
                })?;
        let backend = self
            .storage_backends
            .backend_for_library_root(&library)
            .await?;
        ensure_nfo_export_writable(backend.as_ref(), &library).await?;
        let service = taru_nfo::NfoService::new(backend, self.store.clone(), MovieNfoCodec);
        let summary = service
            .export_media_source(NfoExportSourceRequest {
                library_id: side_effect.library_id,
                source_id,
                policy: library.options.metadata_profile.local_metadata_policy,
                force: payload.policy.force(),
            })
            .await?;

        if let Some(failure) = summary.failures.first() {
            return Err(nfo_export_failure_error(failure.kind));
        }

        Ok(AppliedLibraryFileWrite {
            item_id: source.item_id,
            source: "nfo_export".to_owned(),
            report_json: nfo_export_apply_report(payload.policy, &summary)?,
        })
    }
}

pub(super) struct AppliedLibraryFileWrite {
    pub(super) item_id: MediaItemId,
    pub(super) source: String,
    pub(super) report_json: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonLibraryFileRole {
    Nfo,
}

impl AddonLibraryFileRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Nfo => "nfo",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum AddonNfoExportPolicy {
    CreateMissing,
    ReplaceExistingPreserving,
}

impl AddonNfoExportPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CreateMissing => "create_missing",
            Self::ReplaceExistingPreserving => "replace_existing_preserving",
        }
    }

    const fn force(self) -> bool {
        match self {
            Self::CreateMissing => false,
            Self::ReplaceExistingPreserving => true,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AddonLibraryFileWritePayload {
    file_role: AddonLibraryFileRole,
    policy: AddonNfoExportPolicy,
}

fn parse_addon_library_file_write_payload(
    payload_json: &str,
) -> Result<AddonLibraryFileWritePayload> {
    serde_json::from_str::<AddonLibraryFileWritePayload>(payload_json).map_err(|err| {
        TaruError::InvalidInput {
            message: format!("invalid addon library_file_write payload: {err}"),
        }
    })
}

fn nfo_export_apply_report(
    policy: AddonNfoExportPolicy,
    summary: &NfoExportSourceSummary,
) -> Result<String> {
    let report = serde_json::json!({
        "kind": "nfo_export",
        "file_role": AddonLibraryFileRole::Nfo.as_str(),
        "policy": policy.as_str(),
        "scanned_sources": summary.scanned_sources,
        "exported_items": summary.exported_items,
        "skipped_items": summary.skipped_items,
        "failed_items": summary.failed_items,
        "backed_up_items": summary.backed_up_items,
        "pruned_backup_items": summary.pruned_backup_items,
        "pruned_backups": summary.pruned_backups,
        "prune_failures": summary.prune_failures.len(),
    });

    serde_json::to_string(&report).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to serialize addon NFO export report: {err}"),
    })
}

fn nfo_export_failure_error(kind: NfoFailureKind) -> TaruError {
    match kind {
        NfoFailureKind::StorageRead | NfoFailureKind::StorageWrite => TaruError::storage(
            "nfo_export",
            StorageErrorKind::Unknown,
            "NFO export storage operation failed",
        ),
        NfoFailureKind::StorageBackup => TaruError::storage(
            "nfo_export",
            StorageErrorKind::Backup,
            "NFO export backup failed",
        ),
        NfoFailureKind::StorageUnsupported => {
            TaruError::Unsupported("NFO export storage backend is unsupported")
        }
        NfoFailureKind::MissingMediaItem => TaruError::NotFound {
            entity: "media_item",
            id: "nfo_export_target".to_owned(),
        },
        NfoFailureKind::NfoConflict => TaruError::Conflict {
            message: "NFO export preservation conflict".to_owned(),
        },
        NfoFailureKind::InvalidSidecarPath
        | NfoFailureKind::NfoParse
        | NfoFailureKind::NfoPreservation
        | NfoFailureKind::NfoRender
        | NfoFailureKind::Unknown => TaruError::InvalidInput {
            message: format!("NFO export failed: {kind:?}"),
        },
    }
}
