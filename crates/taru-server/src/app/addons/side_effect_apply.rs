use std::sync::Arc;

use taru_core::{
    AddonPermission, AddonRepository, AddonSideEffectApplyOutcome, AddonSideEffectApplyStatus,
    AddonSideEffectRecord, AddonSideEffectValidationStatus, Result, TaruError,
};
use taru_db::TaruDatabase;
use tokio::sync::Semaphore;

use super::{
    AddonAppService, artwork_write::AddonArtworkWriteAdapter,
    library_file_write::AddonLibraryFileWriteAdapter, metadata_write::AddonMetadataWriteAdapter,
};
use crate::app::storage::StorageBackendRegistry;

impl AddonAppService {
    pub(super) async fn apply_addon_side_effect(
        &self,
        side_effect: AddonSideEffectRecord,
    ) -> Result<AddonSideEffectRecord> {
        AddonSideEffectApplyRouter::new(
            self.store.clone(),
            self.permits.clone(),
            self.storage_backends.clone(),
        )
        .apply(side_effect)
        .await
    }
}

#[derive(Clone, Debug)]
struct AddonSideEffectApplyRouter {
    store: TaruDatabase,
    metadata_write: AddonMetadataWriteAdapter,
    library_file_write: AddonLibraryFileWriteAdapter,
    artwork_write: AddonArtworkWriteAdapter,
}

impl AddonSideEffectApplyRouter {
    fn new(
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

    async fn apply(&self, side_effect: AddonSideEffectRecord) -> Result<AddonSideEffectRecord> {
        if side_effect.validation_status != AddonSideEffectValidationStatus::Accepted {
            return self
                .store
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
                .await;
        }

        match side_effect.permission {
            AddonPermission::MetadataWrite => match self.metadata_write.apply(&side_effect).await {
                Ok(applied) => {
                    debug_assert_eq!(applied.side_effect.applied_item_id, Some(applied.item_id));
                    debug_assert_eq!(
                        applied.side_effect.applied_source.as_deref(),
                        Some(applied.source.as_str())
                    );
                    Ok(applied.side_effect)
                }
                Err(error) => self.record_apply_failure(side_effect.id, error).await,
            },
            AddonPermission::LibraryFileWrite => {
                match self.library_file_write.apply(&side_effect).await {
                    Ok(applied) => {
                        self.store
                            .set_addon_side_effect_apply_outcome(
                                side_effect.id,
                                AddonSideEffectApplyOutcome {
                                    status: AddonSideEffectApplyStatus::Applied,
                                    error_code: None,
                                    item_id: Some(applied.item_id),
                                    source: Some(applied.source),
                                    report_json: Some(applied.report_json),
                                },
                            )
                            .await
                    }
                    Err(error) => self.record_apply_failure(side_effect.id, error).await,
                }
            }
            AddonPermission::ArtworkWrite => match self.artwork_write.apply(&side_effect).await {
                Ok(applied) => {
                    self.store
                        .set_addon_side_effect_apply_outcome(
                            side_effect.id,
                            AddonSideEffectApplyOutcome {
                                status: AddonSideEffectApplyStatus::Applied,
                                error_code: None,
                                item_id: Some(applied.item_id),
                                source: Some(applied.source),
                                report_json: Some(applied.report_json),
                            },
                        )
                        .await
                }
                Err(error) => self.record_apply_failure(side_effect.id, error).await,
            },
            _ => {
                self.store
                    .set_addon_side_effect_apply_outcome(
                        side_effect.id,
                        AddonSideEffectApplyOutcome {
                            status: AddonSideEffectApplyStatus::Skipped,
                            error_code: Some("unsupported".to_owned()),
                            item_id: None,
                            source: None,
                            report_json: None,
                        },
                    )
                    .await
            }
        }
    }

    async fn record_apply_failure(
        &self,
        side_effect_id: taru_core::AddonSideEffectId,
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

pub(super) fn side_effect_apply_error_code(error: &TaruError) -> &'static str {
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
