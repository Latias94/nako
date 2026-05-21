use std::sync::Arc;

use taru_core::{
    AddonMetadataWritePersistenceCommit, AddonPermission, AddonRepository,
    AddonSideEffectApplyOutcome, AddonSideEffectApplyStatus, AddonSideEffectId,
    AddonSideEffectRecord, AddonSideEffectValidationStatus, MediaItemId, MetadataRepository,
    Result, TaruError,
};
use taru_db::TaruDatabase;
use tokio::sync::Semaphore;

use super::{
    artwork_write::AddonArtworkWriteAdapter, library_file_write::AddonLibraryFileWriteAdapter,
    metadata_write::AddonMetadataWriteAdapter,
};
use crate::app::storage::StorageBackendRegistry;

#[derive(Clone, Debug)]
pub(super) struct AddonSideEffectApplyRouter {
    store: TaruDatabase,
    metadata_write: AddonMetadataWriteAdapter,
    library_file_write: AddonLibraryFileWriteAdapter,
    artwork_write: AddonArtworkWriteAdapter,
}

impl AddonSideEffectApplyRouter {
    pub(super) fn new(
        store: TaruDatabase,
        permits: Arc<Semaphore>,
        storage_backends: StorageBackendRegistry,
    ) -> Self {
        Self {
            metadata_write: AddonMetadataWriteAdapter::new(store.clone()),
            library_file_write: AddonLibraryFileWriteAdapter::new(
                store.clone(),
                permits,
                storage_backends,
            ),
            artwork_write: AddonArtworkWriteAdapter::new(store.clone()),
            store,
        }
    }

    pub(super) async fn apply(
        &self,
        side_effect: AddonSideEffectRecord,
    ) -> Result<AddonSideEffectRecord> {
        if side_effect.validation_status != AddonSideEffectValidationStatus::Accepted {
            return self.record_validation_rejection(&side_effect).await;
        }

        let side_effect_id = side_effect.id;
        let command = match side_effect.permission {
            AddonPermission::MetadataWrite => self.metadata_write.apply(&side_effect).await,
            AddonPermission::LibraryFileWrite => self.library_file_write.apply(&side_effect).await,
            AddonPermission::ArtworkWrite => self.artwork_write.apply(&side_effect).await,
            _ => Ok(AddonSideEffectApplyCommand::skipped(
                side_effect_id,
                "unsupported",
            )),
        };

        match command {
            Ok(command) => {
                let command_side_effect_id = command.side_effect_id();
                match self.commit_apply_command(command).await {
                    Ok(side_effect) => Ok(side_effect),
                    Err(error) => {
                        self.record_apply_failure(command_side_effect_id, error)
                            .await
                    }
                }
            }
            Err(error) => self.record_apply_failure(side_effect_id, error).await,
        }
    }

    pub(super) async fn record_validation_rejection(
        &self,
        side_effect: &AddonSideEffectRecord,
    ) -> Result<AddonSideEffectRecord> {
        self.store
            .set_addon_side_effect_apply_outcome(
                side_effect.id,
                AddonSideEffectApplyOutcome {
                    status: AddonSideEffectApplyStatus::Skipped,
                    error_code: side_effect.safe_error_code.clone(),
                    item_id: None,
                    source: None,
                    report_json: None,
                },
            )
            .await
    }

    async fn commit_apply_command(
        &self,
        command: AddonSideEffectApplyCommand,
    ) -> Result<AddonSideEffectRecord> {
        match command {
            AddonSideEffectApplyCommand::MetadataWrite(commit) => {
                let summary = self.store.commit_addon_metadata_write(&commit).await?;
                Ok(summary.side_effect)
            }
            AddonSideEffectApplyCommand::Outcome {
                side_effect_id,
                outcome,
            } => {
                self.store
                    .set_addon_side_effect_apply_outcome(side_effect_id, outcome)
                    .await
            }
        }
    }

    async fn record_apply_failure(
        &self,
        side_effect_id: AddonSideEffectId,
        error: TaruError,
    ) -> Result<AddonSideEffectRecord> {
        let error_code = side_effect_apply_error_code(&error).to_owned();
        self.store
            .set_addon_side_effect_apply_outcome(
                side_effect_id,
                AddonSideEffectApplyOutcome {
                    status: AddonSideEffectApplyStatus::Failed,
                    error_code: Some(error_code),
                    item_id: None,
                    source: None,
                    report_json: None,
                },
            )
            .await?;
        Err(error)
    }
}

#[derive(Debug)]
pub(super) enum AddonSideEffectApplyCommand {
    MetadataWrite(AddonMetadataWritePersistenceCommit),
    Outcome {
        side_effect_id: AddonSideEffectId,
        outcome: AddonSideEffectApplyOutcome,
    },
}

impl AddonSideEffectApplyCommand {
    fn side_effect_id(&self) -> AddonSideEffectId {
        match self {
            Self::MetadataWrite(commit) => commit.side_effect_id,
            Self::Outcome { side_effect_id, .. } => *side_effect_id,
        }
    }

    pub(super) fn applied(
        side_effect_id: AddonSideEffectId,
        item_id: MediaItemId,
        source: impl Into<String>,
        report_json: Option<String>,
    ) -> Self {
        Self::Outcome {
            side_effect_id,
            outcome: AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Applied,
                error_code: None,
                item_id: Some(item_id),
                source: Some(source.into()),
                report_json,
            },
        }
    }

    fn skipped(side_effect_id: AddonSideEffectId, error_code: impl Into<String>) -> Self {
        Self::Outcome {
            side_effect_id,
            outcome: AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Skipped,
                error_code: Some(error_code.into()),
                item_id: None,
                source: None,
                report_json: None,
            },
        }
    }
}

fn side_effect_apply_error_code(error: &TaruError) -> &'static str {
    match error {
        TaruError::InvalidInput { .. } => "invalid_payload",
        TaruError::Forbidden { .. } => "forbidden",
        TaruError::NotFound { .. } => "not_found",
        TaruError::Unauthorized { .. } => "unauthorized",
        TaruError::Conflict { .. } => "conflict",
        TaruError::Unsupported(_) => "unsupported",
        TaruError::Provider { .. } => "provider_error",
        TaruError::Storage { .. } => "storage_error",
        TaruError::Database { .. } => "database_error",
    }
}
