use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use nako_api::extension::{
    AutomationArtifactsResponse, AutomationProviderResponse, AutomationProvidersResponse,
    EnqueueAutomationJobRequest, UpsertAutomationProviderRequest,
};
use nako_automation::AutomationJobService;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderId, AutomationRepository, CanonicalMetadata, ExternalProvider,
    GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS,
    GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS,
    GeneratedArtifactAcceptanceActionKind, GeneratedArtifactAcceptanceBoundary,
    GeneratedArtifactAcceptancePlan, GeneratedArtifactAcceptancePlanReason,
    GeneratedArtifactAcceptancePlanStatus, GeneratedArtifactMetadataApplyFieldPlan,
    GeneratedArtifactMetadataApplyOutcomeCommit, GeneratedArtifactMetadataApplyOutcomeId,
    GeneratedArtifactMetadataApplyOutcomeRecord, GeneratedArtifactMetadataApplyOutcomeStatus,
    GeneratedArtifactMetadataApplyPlan, GeneratedArtifactMetadataApplyPlanReason,
    GeneratedArtifactMetadataApplyPlanStatus, GeneratedArtifactMetadataApplyRequest,
    GeneratedArtifactMetadataApplyResult, GeneratedArtifactMetadataApplyResultStatus,
    GeneratedArtifactMetadataBulkApplyBatchCommit, GeneratedArtifactMetadataBulkApplyBatchId,
    GeneratedArtifactMetadataBulkApplyBatchItemCommit,
    GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
    GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    GeneratedArtifactMetadataBulkApplyBatchRecord, GeneratedArtifactMetadataBulkApplyBatchRequest,
    GeneratedArtifactMetadataBulkApplyBatchStatus, GeneratedArtifactMetadataBulkApplyPlan,
    GeneratedArtifactMetadataBulkApplyPlanItem, GeneratedArtifactMetadataBulkApplyPlanItemReason,
    GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    GeneratedArtifactMetadataBulkApplyPlanRequest, GeneratedArtifactMetadataBulkApplyPlanSelection,
    GeneratedArtifactMetadataBulkApplyPlanSummary, GeneratedArtifactMetadataFieldAction,
    GeneratedArtifactMetadataFieldReason, GeneratedArtifactMetadataValueSummary,
    GeneratedArtifactProposal, GeneratedArtifactProviderMappingAction,
    GeneratedArtifactProviderMappingApplyCommit, GeneratedArtifactProviderMappingPlan,
    GeneratedArtifactProviderMappingReason, GeneratedArtifactProviderSubjectPlan,
    GeneratedArtifactReviewDecision, GeneratedArtifactReviewResult, Job, JobId, JobKind,
    JobRepository, LibraryRepository, MediaItem, MediaItemId, MediaRepository,
    MetadataApplicationPersistenceCommit, MetadataField, MetadataFieldLock, MetadataMergePolicy,
    MetadataRepository, MetadataSource, NakoError, NewAutomationProviderConfig, NewJob,
    PageRequest, ProviderMapping, ProviderMappingId, ProviderMappingRepository,
    ProviderMappingStatus, ProviderSubject, ProviderSubjectId, ProviderSubjectKind, Result,
};
use nako_db::NakoDatabase;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::app::{
    job_runtime::{
        DurableJobOperationError, DurableJobOperationResult, DurableJobRunOutcome,
        DurableJobRuntime,
    },
    metadata_application::{
        MetadataApplication, MetadataApplicationCommand, MetadataApplicationLockScope,
        MetadataApplicationMode, MetadataApplicationProvenance,
    },
};

#[derive(Clone, Debug)]
struct UnavailableAutomationProvider;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GeneratedArtifactMetadataBulkApplyJobInput {
    batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
}

#[async_trait]
impl nako_automation::AutomationProvider for UnavailableAutomationProvider {
    fn descriptor(&self) -> nako_automation::AutomationProviderDescriptor {
        nako_automation::AutomationProviderDescriptor {
            id: AutomationProviderId::new(),
            name: "unavailable".to_owned(),
            capabilities: vec![
                AutomationCapability::Recommendation,
                AutomationCapability::MetadataCleanup,
                AutomationCapability::Summary,
                AutomationCapability::TitleMatch,
            ],
        }
    }

    async fn run(
        &self,
        _request: nako_automation::AutomationRequest,
    ) -> Result<nako_automation::AutomationOutcome> {
        Err(NakoError::Provider {
            provider: "automation".to_owned(),
            message: "no concrete automation provider runner is configured".to_owned(),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AutomationAppService {
    store: NakoDatabase,
    metadata_permits: Arc<Semaphore>,
}

impl AutomationAppService {
    pub(crate) fn new(store: NakoDatabase, metadata_permits: Arc<Semaphore>) -> Self {
        Self {
            store,
            metadata_permits,
        }
    }

    fn normalize_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<NewAutomationProviderConfig> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "automation provider name cannot be empty".to_owned(),
            });
        }

        let base_url = request.base_url.trim().to_owned();
        if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
            return Err(NakoError::InvalidInput {
                message: "automation provider base_url must use http or https".to_owned(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        let capabilities = request
            .capabilities
            .into_iter()
            .filter(|capability| seen.insert(*capability))
            .collect::<Vec<_>>();
        if capabilities.is_empty() {
            return Err(NakoError::InvalidInput {
                message: "automation provider must declare at least one capability".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(30_000);
        if !(100..=120_000).contains(&timeout_ms) {
            return Err(NakoError::InvalidInput {
                message: "automation provider timeout_ms must be between 100 and 120000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(2);
        if !(1..=5).contains(&max_attempts) {
            return Err(NakoError::InvalidInput {
                message: "automation provider max_attempts must be between 1 and 5".to_owned(),
            });
        }

        let secret_env = request.secret_env.and_then(|value| {
            let trimmed = value.trim().to_owned();
            (!trimmed.is_empty()).then_some(trimmed)
        });

        Ok(NewAutomationProviderConfig {
            id: request.id.unwrap_or_else(AutomationProviderId::new),
            name,
            base_url,
            secret_env,
            capabilities,
            timeout_ms,
            max_attempts,
            status: request.status,
        })
    }

    pub async fn upsert_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<AutomationProviderResponse> {
        let provider = self.normalize_automation_provider(request)?;
        let provider = self.store.upsert_automation_provider(provider).await?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn get_automation_provider(
        &self,
        provider_id: AutomationProviderId,
    ) -> Result<AutomationProviderResponse> {
        let provider = self
            .store
            .get_automation_provider(provider_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_provider",
                id: provider_id.to_string(),
            })?;

        Ok(AutomationProviderResponse { provider })
    }

    pub async fn list_enabled_automation_providers(&self) -> Result<AutomationProvidersResponse> {
        let providers = self.store.list_enabled_automation_providers().await?;

        Ok(AutomationProvidersResponse { providers })
    }

    pub async fn enqueue_automation_job(
        &self,
        request: EnqueueAutomationJobRequest,
    ) -> Result<Job> {
        let input = request
            .into_job_input()
            .map_err(|err| NakoError::InvalidInput {
                message: format!("failed to serialize automation prompt: {err}"),
            })?;
        let service = AutomationJobService::new(UnavailableAutomationProvider);

        service.enqueue_job(&self.store, input).await
    }

    pub async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<AutomationArtifactsResponse> {
        self.store
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })?;
        let artifacts = self.store.list_automation_artifacts_for_job(job_id).await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }

    pub async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<AutomationArtifactsResponse> {
        self.store
            .get_media_item(item_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_item",
                id: item_id.to_string(),
            })?;
        let artifacts = self
            .store
            .list_automation_artifacts_for_item(item_id, page)
            .await?;

        Ok(AutomationArtifactsResponse { artifacts })
    }

    pub async fn list_generated_artifact_proposals(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactProposal>> {
        self.store.list_generated_artifact_proposals(page).await
    }

    pub async fn plan_generated_artifact_review(
        &self,
        artifact_id: AutomationArtifactId,
        decision: GeneratedArtifactReviewDecision,
    ) -> Result<GeneratedArtifactAcceptancePlan> {
        let proposal = self.generated_artifact_proposal(artifact_id).await?;

        Ok(generated_artifact_acceptance_plan(proposal, decision))
    }

    pub async fn review_generated_artifact(
        &self,
        artifact_id: AutomationArtifactId,
        decision: GeneratedArtifactReviewDecision,
    ) -> Result<GeneratedArtifactReviewResult> {
        let plan = self
            .plan_generated_artifact_review(artifact_id, decision)
            .await?;
        let existing = self
            .store
            .get_automation_artifact(artifact_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_artifact",
                id: artifact_id.to_string(),
            })?;
        let target_status = match decision {
            GeneratedArtifactReviewDecision::Accept => AutomationArtifactStatus::Accepted,
            GeneratedArtifactReviewDecision::Reject => AutomationArtifactStatus::Rejected,
        };
        if existing.status == target_status {
            return Ok(GeneratedArtifactReviewResult {
                artifact_id,
                decision,
                artifact_status: existing.status,
                accepted_at: existing.accepted_at,
                idempotent_replay: true,
                plan,
            });
        }
        if existing.status != AutomationArtifactStatus::Proposed {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "cannot change reviewed generated artifact {} from {:?} to {:?}",
                    artifact_id, existing.status, target_status
                ),
            });
        }
        if !plan.status.executable() {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact review plan is not executable: {:?}",
                    plan.status
                ),
            });
        }

        let artifact = self
            .store
            .set_automation_artifact_status(artifact_id, target_status)
            .await?;

        Ok(GeneratedArtifactReviewResult {
            artifact_id,
            decision,
            artifact_status: artifact.status,
            accepted_at: artifact.accepted_at,
            idempotent_replay: false,
            plan,
        })
    }

    pub async fn plan_generated_artifact_metadata_apply(
        &self,
        artifact_id: AutomationArtifactId,
    ) -> Result<GeneratedArtifactMetadataApplyPlan> {
        let proposal = self.generated_artifact_proposal(artifact_id).await?;
        let artifact = self
            .store
            .get_automation_artifact(artifact_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_artifact",
                id: artifact_id.to_string(),
            })?;
        let mut reasons = Vec::new();
        let mut status = GeneratedArtifactMetadataApplyPlanStatus::Ready;

        if proposal.status != AutomationArtifactStatus::Accepted {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::ArtifactNotAccepted);
        }
        if proposal.kind != AutomationArtifactKind::MetadataSuggestion {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::UnsupportedArtifactKind);
        }
        if !status.executable() {
            return Ok(self.generated_artifact_metadata_apply_plan(
                proposal,
                status,
                reasons,
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                0,
                0,
                0,
            ));
        }

        let Some(item_id) = proposal.target.item_id else {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::MissingMediaItemTarget);
            return Ok(self.generated_artifact_metadata_apply_plan(
                proposal,
                status,
                reasons,
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                0,
                0,
                0,
            ));
        };
        let Some(library_id) = proposal.target.library_id else {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::MissingLibraryTarget);
            return Ok(self.generated_artifact_metadata_apply_plan(
                proposal,
                status,
                reasons,
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                0,
                0,
                0,
            ));
        };

        let item = match self.store.get_media_item(item_id).await? {
            Some(item) => item,
            None => {
                status = GeneratedArtifactMetadataApplyPlanStatus::Stale;
                reasons.push(GeneratedArtifactMetadataApplyPlanReason::MissingMediaItem);
                return Ok(self.generated_artifact_metadata_apply_plan(
                    proposal,
                    status,
                    reasons,
                    Vec::new(),
                    Vec::new(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ));
            }
        };
        let library = match self.store.get_library(library_id).await? {
            Some(library) => library,
            None => {
                status = GeneratedArtifactMetadataApplyPlanStatus::Stale;
                reasons.push(GeneratedArtifactMetadataApplyPlanReason::MissingLibrary);
                return Ok(self.generated_artifact_metadata_apply_plan(
                    proposal,
                    status,
                    reasons,
                    Vec::new(),
                    Vec::new(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ));
            }
        };

        if let Some(source_id) = proposal.target.source_id {
            match self.store.get_media_source(source_id).await? {
                Some(source) if source.item_id != item_id || source.library_id != library_id => {
                    status = GeneratedArtifactMetadataApplyPlanStatus::Stale;
                    reasons.push(GeneratedArtifactMetadataApplyPlanReason::TargetMismatch);
                }
                Some(_) => {}
                None => {
                    status = GeneratedArtifactMetadataApplyPlanStatus::Stale;
                    reasons.push(GeneratedArtifactMetadataApplyPlanReason::MissingMediaSource);
                }
            }
        }

        let locks = self.store.list_field_locks(item.id).await?;
        let (incoming, suggested_fields) = match parse_generated_artifact_metadata_patch(
            &artifact.artifact_json,
            &item.metadata,
        ) {
            Ok(parsed) => parsed,
            Err(_) => {
                status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
                reasons.push(GeneratedArtifactMetadataApplyPlanReason::InvalidPayloadJson);
                return Ok(self.generated_artifact_metadata_apply_plan(
                    proposal,
                    status,
                    reasons,
                    Vec::new(),
                    Vec::new(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ));
            }
        };
        let provider_mappings = self
            .generated_artifact_provider_mapping_plans(item.id, &artifact.artifact_json)
            .await?;
        let (
            apply_provider_mapping_count,
            skipped_provider_mapping_count,
            noop_provider_mapping_count,
        ) = count_provider_mapping_actions(&provider_mappings);
        if suggested_fields.is_empty() && provider_mappings.is_empty() {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::NoSupportedMetadataFields);
            return Ok(self.generated_artifact_metadata_apply_plan(
                proposal,
                status,
                if reasons.is_empty() {
                    vec![GeneratedArtifactMetadataApplyPlanReason::Ready]
                } else {
                    reasons
                },
                Vec::new(),
                Vec::new(),
                0,
                0,
                0,
                0,
                0,
                0,
            ));
        }
        let policy = MetadataMergePolicy::from_locks_and_mode(
            &locks,
            library.options.metadata_profile.refresh_mode,
        );
        let merged = policy.merge(&item.metadata, &incoming);
        let locked_fields = locked_metadata_fields(&locks);
        let mut fields = Vec::new();
        let mut apply_field_count = 0_u32;
        let mut skipped_field_count = 0_u32;
        let mut noop_field_count = 0_u32;

        for field in suggested_fields {
            let current = summarize_metadata_field(&item.metadata, field)?;
            let incoming_summary = summarize_metadata_field(&incoming, field)?;
            let merged_summary = summarize_metadata_field(&merged, field)?;
            let mut field_reasons = Vec::new();
            let action = if locked_fields.contains(&field) {
                field_reasons.push(GeneratedArtifactMetadataFieldReason::FieldLocked);
                skipped_field_count = skipped_field_count.saturating_add(1);
                GeneratedArtifactMetadataFieldAction::Skip
            } else if merged_summary == current {
                if current != incoming_summary {
                    field_reasons.push(GeneratedArtifactMetadataFieldReason::ExistingValuePresent);
                } else {
                    field_reasons.push(GeneratedArtifactMetadataFieldReason::Unchanged);
                }
                if current == incoming_summary {
                    noop_field_count = noop_field_count.saturating_add(1);
                    GeneratedArtifactMetadataFieldAction::Noop
                } else {
                    skipped_field_count = skipped_field_count.saturating_add(1);
                    GeneratedArtifactMetadataFieldAction::Skip
                }
            } else {
                field_reasons.push(GeneratedArtifactMetadataFieldReason::Ready);
                apply_field_count = apply_field_count.saturating_add(1);
                GeneratedArtifactMetadataFieldAction::Apply
            };
            fields.push(GeneratedArtifactMetadataApplyFieldPlan {
                field,
                action,
                reasons: field_reasons,
                current,
                incoming: incoming_summary,
            });
        }

        if apply_field_count == 0 && apply_provider_mapping_count == 0 && status.executable() {
            status = GeneratedArtifactMetadataApplyPlanStatus::Blocked;
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::NoApplicableMetadataFields);
        }

        if reasons.is_empty() {
            reasons.push(GeneratedArtifactMetadataApplyPlanReason::Ready);
        }

        Ok(self.generated_artifact_metadata_apply_plan(
            proposal,
            status,
            reasons,
            fields,
            provider_mappings,
            apply_field_count,
            skipped_field_count,
            noop_field_count,
            apply_provider_mapping_count,
            skipped_provider_mapping_count,
            noop_provider_mapping_count,
        ))
    }

    pub async fn plan_generated_artifact_metadata_bulk_apply(
        &self,
        request: GeneratedArtifactMetadataBulkApplyPlanRequest,
    ) -> Result<GeneratedArtifactMetadataBulkApplyPlan> {
        let requested_count = request.artifact_ids.len();
        if requested_count == 0 {
            return Err(NakoError::InvalidInput {
                message:
                    "generated artifact metadata bulk apply plan requires at least one artifact_id"
                        .to_owned(),
            });
        }
        if requested_count > GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact metadata bulk apply plan supports at most {} artifact_ids",
                    GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS
                ),
            });
        }

        let mut seen = HashSet::new();
        let mut selected_artifact_ids = Vec::with_capacity(requested_count);
        for artifact_id in request.artifact_ids {
            if seen.insert(artifact_id) {
                selected_artifact_ids.push(artifact_id);
            }
        }

        let mut summary = GeneratedArtifactMetadataBulkApplyPlanSummary::default();
        let mut items = Vec::with_capacity(selected_artifact_ids.len());

        for artifact_id in selected_artifact_ids.iter().copied() {
            match self
                .plan_generated_artifact_metadata_apply(artifact_id)
                .await
            {
                Ok(plan) => {
                    summary.planned_artifact_count =
                        summary.planned_artifact_count.saturating_add(1);
                    if plan.executable {
                        summary.executable_artifact_count =
                            summary.executable_artifact_count.saturating_add(1);
                    }
                    match plan.status {
                        GeneratedArtifactMetadataApplyPlanStatus::Ready => {
                            summary.ready_artifact_count =
                                summary.ready_artifact_count.saturating_add(1);
                        }
                        GeneratedArtifactMetadataApplyPlanStatus::Blocked => {
                            summary.blocked_artifact_count =
                                summary.blocked_artifact_count.saturating_add(1);
                        }
                        GeneratedArtifactMetadataApplyPlanStatus::Stale => {
                            summary.stale_artifact_count =
                                summary.stale_artifact_count.saturating_add(1);
                        }
                    }
                    summary.apply_field_count = summary
                        .apply_field_count
                        .saturating_add(plan.apply_field_count);
                    summary.skipped_field_count = summary
                        .skipped_field_count
                        .saturating_add(plan.skipped_field_count);
                    summary.noop_field_count = summary
                        .noop_field_count
                        .saturating_add(plan.noop_field_count);

                    items.push(GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned,
                        executable: plan.executable,
                        reasons: vec![GeneratedArtifactMetadataBulkApplyPlanItemReason::Planned],
                        plan: Some(plan),
                    });
                }
                Err(NakoError::NotFound { entity, .. })
                    if entity == "generated_artifact_proposal"
                        || entity == "automation_artifact" =>
                {
                    summary.missing_artifact_count =
                        summary.missing_artifact_count.saturating_add(1);
                    items.push(GeneratedArtifactMetadataBulkApplyPlanItem {
                        artifact_id,
                        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Missing,
                        executable: false,
                        reasons: vec![
                            GeneratedArtifactMetadataBulkApplyPlanItemReason::MissingArtifact,
                        ],
                        plan: None,
                    });
                }
                Err(error) => return Err(error),
            }
        }

        Ok(GeneratedArtifactMetadataBulkApplyPlan {
            selection: GeneratedArtifactMetadataBulkApplyPlanSelection {
                requested_artifact_count: requested_count as u32,
                selected_artifact_count: selected_artifact_ids.len() as u32,
                duplicate_artifact_count: requested_count
                    .saturating_sub(selected_artifact_ids.len())
                    as u32,
                max_artifact_count: GENERATED_ARTIFACT_METADATA_BULK_APPLY_PLAN_MAX_ARTIFACTS
                    as u32,
            },
            summary,
            items,
        })
    }

    pub async fn create_generated_artifact_metadata_bulk_apply_batch(
        &self,
        request: GeneratedArtifactMetadataBulkApplyBatchRequest,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        let idempotency_key = normalize_generated_artifact_metadata_bulk_apply_idempotency_key(
            &request.idempotency_key,
        )?;
        if let Some(existing) = self
            .store
            .find_generated_artifact_metadata_bulk_apply_batch(&idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        let plan = self
            .plan_generated_artifact_metadata_bulk_apply(
                GeneratedArtifactMetadataBulkApplyPlanRequest {
                    artifact_ids: request.artifact_ids,
                },
            )
            .await?;
        if plan.summary.executable_artifact_count == 0 {
            return Err(NakoError::InvalidInput {
                message: "generated artifact metadata bulk apply batch requires at least one executable item"
                    .to_owned(),
            });
        }

        let batch_id = GeneratedArtifactMetadataBulkApplyBatchId::new();
        let job_id = JobId::new();
        let job_input = GeneratedArtifactMetadataBulkApplyJobInput { batch_id };
        let input_json =
            serde_json::to_string(&job_input).map_err(|err| NakoError::InvalidInput {
                message: format!(
                    "failed to serialize generated artifact metadata bulk apply job input: {err}"
                ),
            })?;
        let items = plan
            .items
            .iter()
            .enumerate()
            .map(
                |(index, item)| GeneratedArtifactMetadataBulkApplyBatchItemCommit {
                    artifact_id: item.artifact_id,
                    position: index as u32,
                    status: if item.executable {
                        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
                    } else {
                        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
                    },
                    idempotency_key: generated_artifact_metadata_bulk_apply_item_idempotency_key(
                        batch_id,
                        item.artifact_id,
                    ),
                    plan_item: item.clone(),
                },
            )
            .collect();

        self.store
            .commit_generated_artifact_metadata_bulk_apply_batch(
                &GeneratedArtifactMetadataBulkApplyBatchCommit {
                    id: batch_id,
                    job: NewJob {
                        id: job_id,
                        kind: JobKind::GeneratedArtifactMetadataBulkApply,
                        resource_class: GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS
                            .to_owned(),
                        library_id: None,
                        source_id: None,
                        input_json: Some(input_json),
                    },
                    idempotency_key,
                    status: GeneratedArtifactMetadataBulkApplyBatchStatus::Queued,
                    selection: plan.selection,
                    summary: plan.summary,
                    items,
                },
            )
            .await
    }

    pub async fn get_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        self.store
            .get_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "generated_artifact_metadata_bulk_apply_batch",
                id: batch_id.to_string(),
            })
    }

    pub(crate) async fn execute_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        let batch = self
            .store
            .get_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "generated_artifact_metadata_bulk_apply_batch",
                id: batch_id.to_string(),
            })?;
        if generated_artifact_metadata_bulk_apply_batch_status_is_terminal(batch.status) {
            return Ok(batch);
        }

        let _permit = self
            .acquire_generated_artifact_metadata_bulk_apply_permit()
            .await?;
        let runtime = DurableJobRuntime::new(self.store.clone());
        let run = runtime
            .run_job_with_context(
                batch.job_id,
                "generated artifact metadata bulk apply job",
                |context| async move {
                    self.run_generated_artifact_metadata_bulk_apply_batch(batch_id, context)
                        .await
                },
                |batch| {
                    DurableJobRuntime::serialize_summary(
                        &batch.execution_summary,
                        "generated artifact metadata bulk apply job summary",
                    )
                },
            )
            .await;

        match run {
            Ok(DurableJobRunOutcome::Completed(run)) => Ok(run.output),
            Ok(DurableJobRunOutcome::Cancelled(_job)) => {
                self.update_generated_artifact_metadata_bulk_apply_batch_status_best_effort(
                    batch_id,
                    GeneratedArtifactMetadataBulkApplyBatchStatus::Running,
                    GeneratedArtifactMetadataBulkApplyBatchStatus::Cancelled,
                )
                .await
            }
            Err(err) => {
                let _ = self
                    .update_generated_artifact_metadata_bulk_apply_batch_status_best_effort(
                        batch_id,
                        GeneratedArtifactMetadataBulkApplyBatchStatus::Running,
                        GeneratedArtifactMetadataBulkApplyBatchStatus::Failed,
                    )
                    .await;
                Err(err)
            }
        }
    }

    pub async fn apply_generated_artifact_metadata(
        &self,
        request: GeneratedArtifactMetadataApplyRequest,
    ) -> Result<GeneratedArtifactMetadataApplyResult> {
        let artifact_id = request.artifact_id;
        let idempotency_key =
            normalize_generated_artifact_metadata_apply_idempotency_key(&request.idempotency_key)?;
        if let Some(outcome) = self
            .store
            .find_generated_artifact_metadata_apply_outcome(artifact_id, &idempotency_key)
            .await?
        {
            return generated_artifact_metadata_apply_result_from_outcome(outcome, true);
        }

        let plan = self
            .plan_generated_artifact_metadata_apply(artifact_id)
            .await?;
        if !plan.executable {
            if generated_artifact_metadata_apply_is_idempotent_noop(&plan) {
                let outcome = self
                    .store
                    .commit_generated_artifact_metadata_apply_outcome(
                        &GeneratedArtifactMetadataApplyOutcomeCommit {
                            id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                            artifact_id,
                            idempotency_key,
                            status: GeneratedArtifactMetadataApplyOutcomeStatus::Noop,
                            applied: false,
                            changed: false,
                            applied_source: None,
                            item_id: plan.target.item_id,
                            plan,
                            error_code: None,
                            error_message: None,
                            metadata_application: None,
                            provider_mappings: Vec::new(),
                        },
                    )
                    .await?;
                return generated_artifact_metadata_apply_result_from_outcome(outcome, false);
            }

            let message = format!(
                "generated artifact metadata apply plan is not executable: {:?}",
                plan.status
            );
            return self
                .commit_generated_artifact_metadata_apply_failure(
                    artifact_id,
                    idempotency_key,
                    plan,
                    "plan_not_executable",
                    message,
                )
                .await;
        }

        let artifact = self
            .store
            .get_automation_artifact(artifact_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_artifact",
                id: artifact_id.to_string(),
            })?;
        let (item, library_id) = match self
            .resolve_generated_artifact_metadata_apply_target(&plan)
            .await
        {
            Ok(target) => target,
            Err(error) => {
                return self
                    .commit_generated_artifact_metadata_apply_failure(
                        artifact_id,
                        idempotency_key,
                        plan,
                        "target_stale",
                        error.to_string(),
                    )
                    .await;
            }
        };
        let item_id = item.id;
        let mut changed = false;
        let mut applied_source = None;
        let mut metadata_application = None;

        if plan.apply_field_count > 0 {
            let (incoming, _) =
                parse_generated_artifact_metadata_patch(&artifact.artifact_json, &item.metadata)?;
            let applied = MetadataApplication::new(self.store.clone())
                .apply(MetadataApplicationCommand {
                    item,
                    source: MetadataSource::User,
                    incoming,
                    mode: MetadataApplicationMode::LibraryProfile { library_id },
                    lock_scope: MetadataApplicationLockScope::ProtectAllLocks,
                    provenance: MetadataApplicationProvenance::GeneratedArtifact {
                        artifact_id,
                        provider_id: artifact.provider_id,
                        library_id,
                    },
                })
                .await?;
            changed = applied.changed;
            applied_source = Some(applied.applied_source.clone());
            metadata_application = Some(MetadataApplicationPersistenceCommit {
                item: applied.item,
                catalog_projection: applied.projection,
            });
        }

        let provider_mappings = self
            .generated_artifact_provider_mapping_apply_commits(item_id, &plan)
            .await?;
        if !provider_mappings.is_empty() {
            changed = true;
            applied_source.get_or_insert_with(|| "user".to_owned());
        }

        let outcome = self
            .store
            .commit_generated_artifact_metadata_apply_outcome(
                &GeneratedArtifactMetadataApplyOutcomeCommit {
                    id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                    artifact_id,
                    idempotency_key,
                    status: GeneratedArtifactMetadataApplyOutcomeStatus::Applied,
                    applied: true,
                    changed,
                    applied_source,
                    item_id: Some(item_id),
                    plan,
                    error_code: None,
                    error_message: None,
                    metadata_application,
                    provider_mappings,
                },
            )
            .await?;

        generated_artifact_metadata_apply_result_from_outcome(outcome, false)
    }

    async fn generated_artifact_proposal(
        &self,
        artifact_id: AutomationArtifactId,
    ) -> Result<GeneratedArtifactProposal> {
        self.store
            .list_generated_artifact_proposals(PageRequest::new(PageRequest::MAX_LIMIT, 0))
            .await?
            .into_iter()
            .find(|proposal| proposal.id == artifact_id)
            .ok_or_else(|| NakoError::NotFound {
                entity: "generated_artifact_proposal",
                id: artifact_id.to_string(),
            })
    }

    async fn generated_artifact_provider_mapping_plans(
        &self,
        item_id: MediaItemId,
        artifact_json: &str,
    ) -> Result<Vec<GeneratedArtifactProviderMappingPlan>> {
        let proposals = parse_generated_artifact_provider_subject_proposals(artifact_json)?;
        if proposals.is_empty() {
            return Ok(Vec::new());
        }

        let existing_mappings = self.provider_mappings_for_item(item_id).await?;
        let mut seen = HashSet::new();
        let mut plans = Vec::new();

        for proposal in proposals {
            let mut reasons = proposal.reasons.clone();
            let valid_subject_key = proposal.subject_key.as_ref().filter(|key| key.len() <= 512);
            let duplicate = match (
                proposal.provider.clone(),
                proposal.subject_kind.clone(),
                valid_subject_key.cloned(),
            ) {
                (Some(provider), Some(subject_kind), Some(subject_key)) => {
                    !seen.insert((provider, subject_kind, subject_key))
                }
                _ => false,
            };
            if duplicate {
                reasons.push(GeneratedArtifactProviderMappingReason::DuplicateProposal);
            }

            let existing_mapping_status = if reasons.is_empty() {
                match (
                    proposal.provider.as_ref(),
                    proposal.subject_kind.as_ref(),
                    proposal.subject_key.as_ref(),
                ) {
                    (Some(provider), Some(subject_kind), Some(subject_key)) => self
                        .store
                        .find_provider_subject(provider, subject_kind, subject_key)
                        .await?
                        .and_then(|subject| {
                            existing_mappings
                                .iter()
                                .find(|mapping| mapping.subject_id == subject.id)
                                .map(|mapping| mapping.status)
                        }),
                    _ => None,
                }
            } else {
                None
            };

            match existing_mapping_status {
                Some(ProviderMappingStatus::Accepted) => {
                    reasons.push(GeneratedArtifactProviderMappingReason::ExistingAcceptedMapping)
                }
                Some(ProviderMappingStatus::Candidate) => {
                    reasons.push(GeneratedArtifactProviderMappingReason::ExistingCandidateMapping);
                    reasons.push(GeneratedArtifactProviderMappingReason::Ready);
                }
                Some(ProviderMappingStatus::Rejected) => {
                    reasons.push(GeneratedArtifactProviderMappingReason::ExistingRejectedMapping);
                }
                None if reasons.is_empty() => {
                    reasons.push(GeneratedArtifactProviderMappingReason::Ready);
                }
                None => {}
            }

            let action = provider_mapping_action(&reasons);
            plans.push(GeneratedArtifactProviderMappingPlan {
                subject: GeneratedArtifactProviderSubjectPlan {
                    provider: proposal.provider,
                    provider_name: proposal.provider_name,
                    subject_kind: proposal.subject_kind,
                    subject_kind_name: proposal.subject_kind_name,
                    subject_key: proposal.subject_key,
                    title: proposal.title,
                    release_year: proposal.release_year,
                    locale: proposal.locale,
                },
                action,
                reasons,
                confidence_milli: proposal.confidence_milli,
                existing_mapping_status,
            });
        }

        Ok(plans)
    }

    async fn provider_mappings_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<nako_core::ProviderMapping>> {
        let mut offset = 0;
        let mut all = Vec::new();

        loop {
            let page = self
                .store
                .list_provider_mappings_for_item(
                    item_id,
                    PageRequest {
                        limit: PageRequest::MAX_LIMIT,
                        offset,
                    },
                )
                .await?;
            let returned = page.len();
            all.extend(page);
            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(all);
            }
            offset += u64::from(PageRequest::MAX_LIMIT);
        }
    }

    async fn generated_artifact_provider_mapping_apply_commits(
        &self,
        item_id: MediaItemId,
        plan: &GeneratedArtifactMetadataApplyPlan,
    ) -> Result<Vec<GeneratedArtifactProviderMappingApplyCommit>> {
        let mut commits = Vec::new();
        let existing_mappings = self.provider_mappings_for_item(item_id).await?;

        for mapping_plan in plan
            .provider_mappings
            .iter()
            .filter(|mapping| mapping.action == GeneratedArtifactProviderMappingAction::Apply)
        {
            let provider =
                mapping_plan
                    .subject
                    .provider
                    .clone()
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "provider mapping apply plan is missing provider for apply action"
                            .to_owned(),
                    })?;
            let subject_kind = mapping_plan.subject.subject_kind.clone().ok_or_else(|| {
                NakoError::InvalidInput {
                    message: "provider mapping apply plan is missing subject kind for apply action"
                        .to_owned(),
                }
            })?;
            let subject_key = mapping_plan.subject.subject_key.clone().ok_or_else(|| {
                NakoError::InvalidInput {
                    message: "provider mapping apply plan is missing subject key for apply action"
                        .to_owned(),
                }
            })?;

            let existing_subject = self
                .store
                .find_provider_subject(&provider, &subject_kind, &subject_key)
                .await?;
            let mut subject = existing_subject.clone().unwrap_or_else(|| ProviderSubject {
                id: ProviderSubjectId::new(),
                provider,
                subject_kind,
                subject_key,
                title: None,
                release_year: None,
                locale: None,
            });
            if mapping_plan.subject.title.is_some() {
                subject.title = mapping_plan.subject.title.clone();
            }
            if mapping_plan.subject.release_year.is_some() {
                subject.release_year = mapping_plan.subject.release_year;
            }
            if mapping_plan.subject.locale.is_some() {
                subject.locale = mapping_plan.subject.locale.clone();
            }

            let existing_mapping = existing_mappings
                .iter()
                .find(|mapping| mapping.subject_id == subject.id);
            if let Some(existing) = existing_mapping
                && existing.status == ProviderMappingStatus::Rejected
            {
                return Err(NakoError::InvalidInput {
                    message: format!(
                        "provider mapping proposal for subject {} became rejected before apply",
                        subject.id
                    ),
                });
            }

            commits.push(GeneratedArtifactProviderMappingApplyCommit {
                mapping: ProviderMapping {
                    id: existing_mapping
                        .map(|mapping| mapping.id)
                        .unwrap_or_else(ProviderMappingId::new),
                    item_id,
                    subject_id: subject.id,
                    status: ProviderMappingStatus::Accepted,
                    confidence_milli: mapping_plan.confidence_milli,
                    source: MetadataSource::User,
                },
                subject,
            });
        }

        Ok(commits)
    }

    async fn resolve_generated_artifact_metadata_apply_target(
        &self,
        plan: &GeneratedArtifactMetadataApplyPlan,
    ) -> Result<(MediaItem, nako_core::LibraryId)> {
        let item_id = plan.target.item_id.ok_or_else(|| NakoError::InvalidInput {
            message: "generated artifact metadata apply target is missing media item".to_owned(),
        })?;
        let library_id = plan
            .target
            .library_id
            .ok_or_else(|| NakoError::InvalidInput {
                message: "generated artifact metadata apply target is missing library".to_owned(),
            })?;
        let item =
            self.store
                .get_media_item(item_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "media_item",
                    id: item_id.to_string(),
                })?;
        self.store
            .get_library(library_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "library",
                id: library_id.to_string(),
            })?;

        if let Some(source_id) = plan.target.source_id {
            match self.store.get_media_source(source_id).await? {
                Some(source) if source.item_id == item_id && source.library_id == library_id => {}
                Some(_) => {
                    return Err(NakoError::InvalidInput {
                        message: format!(
                            "generated artifact metadata apply target is stale: source {source_id} no longer belongs to item {item_id} in library {library_id}"
                        ),
                    });
                }
                None => {
                    return Err(NakoError::InvalidInput {
                        message: format!(
                            "generated artifact metadata apply target is stale: source {source_id} is missing"
                        ),
                    });
                }
            }
        }

        Ok((item, library_id))
    }

    async fn commit_generated_artifact_metadata_apply_failure(
        &self,
        artifact_id: AutomationArtifactId,
        idempotency_key: String,
        plan: GeneratedArtifactMetadataApplyPlan,
        error_code: impl Into<String>,
        message: String,
    ) -> Result<GeneratedArtifactMetadataApplyResult> {
        let outcome = self
            .store
            .commit_generated_artifact_metadata_apply_outcome(
                &GeneratedArtifactMetadataApplyOutcomeCommit {
                    id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                    artifact_id,
                    idempotency_key,
                    status: GeneratedArtifactMetadataApplyOutcomeStatus::Failed,
                    applied: false,
                    changed: false,
                    applied_source: None,
                    item_id: plan.target.item_id,
                    plan,
                    error_code: Some(error_code.into()),
                    error_message: Some(message),
                    metadata_application: None,
                    provider_mappings: Vec::new(),
                },
            )
            .await?;

        generated_artifact_metadata_apply_result_from_outcome(outcome, false)
    }

    async fn run_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
        context: crate::app::job_runtime::DurableJobContext,
    ) -> DurableJobOperationResult<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        let batch = self
            .store
            .update_generated_artifact_metadata_bulk_apply_batch_status(
                batch_id,
                GeneratedArtifactMetadataBulkApplyBatchStatus::Queued,
                GeneratedArtifactMetadataBulkApplyBatchStatus::Running,
            )
            .await
            .map_err(DurableJobOperationError::from)?;

        for item in batch.items.clone() {
            context.check_cancelled().await?;
            if item.status != GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending {
                continue;
            }

            let outcome_commit = match self
                .apply_generated_artifact_metadata(GeneratedArtifactMetadataApplyRequest {
                    artifact_id: item.artifact_id,
                    idempotency_key: item.idempotency_key.clone(),
                })
                .await
            {
                Ok(result) => GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit {
                    batch_id,
                    artifact_id: item.artifact_id,
                    status: match result.status {
                        GeneratedArtifactMetadataApplyResultStatus::Applied => {
                            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied
                        }
                        GeneratedArtifactMetadataApplyResultStatus::Noop => {
                            GeneratedArtifactMetadataBulkApplyBatchItemStatus::Noop
                        }
                    },
                    outcome_id: result.outcome_id,
                    error_code: None,
                    error_message: None,
                },
                Err(error) => self
                    .generated_artifact_metadata_bulk_apply_item_failure_commit(&item, &error)
                    .await
                    .map_err(DurableJobOperationError::from)?,
            };

            self.store
                .commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(&outcome_commit)
                .await
                .map_err(DurableJobOperationError::from)?;
        }

        context.check_cancelled().await?;
        self.store
            .update_generated_artifact_metadata_bulk_apply_batch_status(
                batch_id,
                GeneratedArtifactMetadataBulkApplyBatchStatus::Running,
                GeneratedArtifactMetadataBulkApplyBatchStatus::Completed,
            )
            .await
            .map_err(DurableJobOperationError::from)
    }

    async fn generated_artifact_metadata_bulk_apply_item_failure_commit(
        &self,
        item: &nako_core::GeneratedArtifactMetadataBulkApplyBatchItemRecord,
        error: &NakoError,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit> {
        if let Some(outcome) = self
            .store
            .find_generated_artifact_metadata_apply_outcome(item.artifact_id, &item.idempotency_key)
            .await?
        {
            return Ok(GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit {
                batch_id: item.batch_id,
                artifact_id: item.artifact_id,
                status: if outcome.plan.status == GeneratedArtifactMetadataApplyPlanStatus::Stale {
                    GeneratedArtifactMetadataBulkApplyBatchItemStatus::Stale
                } else {
                    GeneratedArtifactMetadataBulkApplyBatchItemStatus::Failed
                },
                outcome_id: Some(outcome.id),
                error_code: outcome.error_code,
                error_message: outcome
                    .error_message
                    .map(redact_generated_artifact_metadata_bulk_apply_error_message),
            });
        }

        Ok(GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit {
            batch_id: item.batch_id,
            artifact_id: item.artifact_id,
            status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::Failed,
            outcome_id: None,
            error_code: Some("apply_failed".to_owned()),
            error_message: Some(redact_generated_artifact_metadata_bulk_apply_error_message(
                error.to_string(),
            )),
        })
    }

    async fn acquire_generated_artifact_metadata_bulk_apply_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit> {
        self.metadata_permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|err| NakoError::InvalidInput {
                message: format!("metadata bulk apply concurrency limiter is unavailable: {err}"),
            })
    }

    async fn update_generated_artifact_metadata_bulk_apply_batch_status_best_effort(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
        expected: GeneratedArtifactMetadataBulkApplyBatchStatus,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        match self
            .store
            .update_generated_artifact_metadata_bulk_apply_batch_status(batch_id, expected, status)
            .await
        {
            Ok(batch) => Ok(batch),
            Err(_) => self
                .store
                .get_generated_artifact_metadata_bulk_apply_batch(batch_id)
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "generated_artifact_metadata_bulk_apply_batch",
                    id: batch_id.to_string(),
                }),
        }
    }

    fn generated_artifact_metadata_apply_plan(
        &self,
        proposal: GeneratedArtifactProposal,
        status: GeneratedArtifactMetadataApplyPlanStatus,
        reasons: Vec<GeneratedArtifactMetadataApplyPlanReason>,
        fields: Vec<GeneratedArtifactMetadataApplyFieldPlan>,
        provider_mappings: Vec<GeneratedArtifactProviderMappingPlan>,
        apply_field_count: u32,
        skipped_field_count: u32,
        noop_field_count: u32,
        apply_provider_mapping_count: u32,
        skipped_provider_mapping_count: u32,
        noop_provider_mapping_count: u32,
    ) -> GeneratedArtifactMetadataApplyPlan {
        GeneratedArtifactMetadataApplyPlan {
            artifact_id: proposal.id,
            status,
            executable: status.executable(),
            reasons,
            target: proposal.target,
            payload: proposal.payload,
            fields,
            provider_mappings,
            apply_field_count,
            skipped_field_count,
            noop_field_count,
            apply_provider_mapping_count,
            skipped_provider_mapping_count,
            noop_provider_mapping_count,
        }
    }
}

fn generated_artifact_metadata_apply_is_idempotent_noop(
    plan: &GeneratedArtifactMetadataApplyPlan,
) -> bool {
    plan.status == GeneratedArtifactMetadataApplyPlanStatus::Blocked
        && plan.apply_field_count == 0
        && plan
            .reasons
            .contains(&GeneratedArtifactMetadataApplyPlanReason::NoApplicableMetadataFields)
}

fn generated_artifact_metadata_bulk_apply_batch_status_is_terminal(
    status: GeneratedArtifactMetadataBulkApplyBatchStatus,
) -> bool {
    matches!(
        status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Completed
            | GeneratedArtifactMetadataBulkApplyBatchStatus::Failed
            | GeneratedArtifactMetadataBulkApplyBatchStatus::Cancelled
    )
}

fn redact_generated_artifact_metadata_bulk_apply_error_message(message: String) -> String {
    const MAX_LEN: usize = 512;
    if message.chars().count() <= MAX_LEN {
        return message;
    }

    message.chars().take(MAX_LEN).collect()
}

fn normalize_generated_artifact_metadata_apply_idempotency_key(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata apply idempotency_key cannot be empty".to_owned(),
        });
    }
    if value.len() > 512 {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata apply idempotency_key must be 512 bytes or fewer"
                .to_owned(),
        });
    }

    Ok(value.to_owned())
}

fn normalize_generated_artifact_metadata_bulk_apply_idempotency_key(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply idempotency_key cannot be empty"
                .to_owned(),
        });
    }
    if value.len() > 512 {
        return Err(NakoError::InvalidInput {
            message:
                "generated artifact metadata bulk apply idempotency_key must be 512 bytes or fewer"
                    .to_owned(),
        });
    }

    Ok(value.to_owned())
}

fn generated_artifact_metadata_bulk_apply_item_idempotency_key(
    batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    artifact_id: AutomationArtifactId,
) -> String {
    format!("generated-artifact-metadata-bulk-apply:{batch_id}:{artifact_id}")
}

fn generated_artifact_metadata_apply_result_from_outcome(
    outcome: GeneratedArtifactMetadataApplyOutcomeRecord,
    idempotent_replay: bool,
) -> Result<GeneratedArtifactMetadataApplyResult> {
    if outcome.status == GeneratedArtifactMetadataApplyOutcomeStatus::Failed {
        return Err(NakoError::InvalidInput {
            message: outcome.error_message.unwrap_or_else(|| {
                format!(
                    "generated artifact metadata apply outcome {} failed",
                    outcome.id
                )
            }),
        });
    }

    let status = match outcome.status {
        GeneratedArtifactMetadataApplyOutcomeStatus::Applied => {
            GeneratedArtifactMetadataApplyResultStatus::Applied
        }
        GeneratedArtifactMetadataApplyOutcomeStatus::Noop => {
            GeneratedArtifactMetadataApplyResultStatus::Noop
        }
        GeneratedArtifactMetadataApplyOutcomeStatus::Failed => unreachable!("handled above"),
    };

    Ok(GeneratedArtifactMetadataApplyResult {
        outcome_id: Some(outcome.id),
        artifact_id: outcome.artifact_id,
        status,
        applied: outcome.applied,
        changed: outcome.changed,
        idempotent_replay,
        applied_source: outcome.applied_source,
        plan: outcome.plan,
    })
}

fn generated_artifact_acceptance_plan(
    proposal: GeneratedArtifactProposal,
    decision: GeneratedArtifactReviewDecision,
) -> GeneratedArtifactAcceptancePlan {
    let mut reasons = Vec::new();
    let proposal_status = proposal.status;
    let proposal_kind = proposal.kind;
    let proposal_target = proposal.target.clone();
    let proposal_payload = proposal.payload.clone();
    let proposal_readiness = proposal.readiness.clone();
    let mut status = if proposal.readiness.actionable {
        GeneratedArtifactAcceptancePlanStatus::Ready
    } else if proposal.readiness.status == nako_core::GeneratedArtifactReadinessStatus::Stale {
        GeneratedArtifactAcceptancePlanStatus::Stale
    } else {
        GeneratedArtifactAcceptancePlanStatus::Blocked
    };

    if proposal.status == AutomationArtifactStatus::Accepted {
        status = GeneratedArtifactAcceptancePlanStatus::AlreadyAccepted;
        reasons.push(GeneratedArtifactAcceptancePlanReason::ArtifactAlreadyAccepted);
    } else if proposal.status == AutomationArtifactStatus::Rejected {
        status = GeneratedArtifactAcceptancePlanStatus::AlreadyRejected;
        reasons.push(GeneratedArtifactAcceptancePlanReason::ArtifactAlreadyRejected);
    } else if !proposal.readiness.actionable
        && !(decision == GeneratedArtifactReviewDecision::Reject
            && proposal.status == AutomationArtifactStatus::Proposed)
    {
        reasons.push(GeneratedArtifactAcceptancePlanReason::ProposalNotReady);
    }

    let (action, boundary) = if decision == GeneratedArtifactReviewDecision::Reject
        && proposal_status == AutomationArtifactStatus::Proposed
    {
        status = GeneratedArtifactAcceptancePlanStatus::Ready;
        reasons.push(GeneratedArtifactAcceptancePlanReason::OperatorRejected);
        (
            GeneratedArtifactAcceptanceActionKind::RejectProposal,
            GeneratedArtifactAcceptanceBoundary::no_mutation(),
        )
    } else if status == GeneratedArtifactAcceptancePlanStatus::Ready {
        match decision {
            GeneratedArtifactReviewDecision::Accept
                if proposal.kind == AutomationArtifactKind::MetadataSuggestion =>
            {
                if proposal.target.item_id.is_some() {
                    reasons.push(GeneratedArtifactAcceptancePlanReason::Ready);
                    reasons.push(
                        GeneratedArtifactAcceptancePlanReason::MetadataAuthorityApplyRequired,
                    );
                    (
                        GeneratedArtifactAcceptanceActionKind::StageMetadataAuthorityReview,
                        GeneratedArtifactAcceptanceBoundary::deferred_metadata_authority(),
                    )
                } else {
                    status = GeneratedArtifactAcceptancePlanStatus::Blocked;
                    reasons.push(GeneratedArtifactAcceptancePlanReason::MissingMediaItemTarget);
                    (
                        GeneratedArtifactAcceptanceActionKind::Noop,
                        GeneratedArtifactAcceptanceBoundary::no_mutation(),
                    )
                }
            }
            GeneratedArtifactReviewDecision::Accept => {
                status = GeneratedArtifactAcceptancePlanStatus::Blocked;
                reasons.push(GeneratedArtifactAcceptancePlanReason::UnsupportedArtifactKind);
                (
                    GeneratedArtifactAcceptanceActionKind::Noop,
                    GeneratedArtifactAcceptanceBoundary::no_mutation(),
                )
            }
            GeneratedArtifactReviewDecision::Reject => {
                unreachable!("reject handled before ready plan")
            }
        }
    } else {
        (
            GeneratedArtifactAcceptanceActionKind::Noop,
            GeneratedArtifactAcceptanceBoundary::no_mutation(),
        )
    };

    if reasons.is_empty() {
        reasons.push(GeneratedArtifactAcceptancePlanReason::Ready);
    }

    GeneratedArtifactAcceptancePlan {
        artifact_id: proposal.id,
        decision,
        status,
        action,
        reasons,
        capability: proposal.capability,
        kind: proposal_kind,
        target: proposal_target,
        payload: proposal_payload,
        readiness: proposal_readiness,
        boundary,
    }
}

#[derive(Default, Debug, Deserialize)]
struct GeneratedArtifactMetadataPatch {
    title: Option<String>,
    original_title: Option<String>,
    sort_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    runtime_minutes: Option<u32>,
    tagline: Option<String>,
    genres: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
struct GeneratedArtifactProviderSubjectProposal {
    provider: Option<ExternalProvider>,
    provider_name: Option<String>,
    subject_kind: Option<ProviderSubjectKind>,
    subject_kind_name: Option<String>,
    subject_key: Option<String>,
    title: Option<String>,
    release_year: Option<i32>,
    locale: Option<String>,
    confidence_milli: Option<u16>,
    reasons: Vec<GeneratedArtifactProviderMappingReason>,
}

fn parse_generated_artifact_provider_subject_proposals(
    artifact_json: &str,
) -> Result<Vec<GeneratedArtifactProviderSubjectProposal>> {
    let value: serde_json::Value =
        serde_json::from_str(artifact_json).map_err(|error| NakoError::InvalidInput {
            message: format!("generated artifact metadata payload is not valid JSON: {error}"),
        })?;
    let Some(object) = value.as_object() else {
        return Ok(Vec::new());
    };

    let mut proposals = Vec::new();
    if let Some(value) = object.get("provider_subject") {
        if let Some(proposal) = parse_generated_artifact_provider_subject_proposal(value) {
            proposals.push(proposal);
        }
    }
    if let Some(value) = object.get("provider_subjects") {
        if let Some(values) = value.as_array() {
            for value in values {
                if let Some(proposal) = parse_generated_artifact_provider_subject_proposal(value) {
                    proposals.push(proposal);
                }
            }
        } else if let Some(proposal) = parse_generated_artifact_provider_subject_proposal(value) {
            proposals.push(proposal);
        }
    }

    Ok(proposals)
}

fn parse_generated_artifact_provider_subject_proposal(
    value: &serde_json::Value,
) -> Option<GeneratedArtifactProviderSubjectProposal> {
    let object = value.as_object()?;
    let mut reasons = Vec::new();
    let provider_name = json_string_field(object, "provider").map(|value| value.to_lowercase());
    let provider = provider_name
        .as_deref()
        .and_then(parse_generated_artifact_external_provider);
    if provider.is_none() {
        reasons.push(GeneratedArtifactProviderMappingReason::UnsupportedProvider);
    }

    let subject_kind_name =
        json_string_field(object, "subject_kind").map(|value| value.to_lowercase());
    let subject_kind = subject_kind_name
        .as_deref()
        .and_then(parse_generated_artifact_provider_subject_kind);
    if subject_kind.is_none() {
        reasons.push(GeneratedArtifactProviderMappingReason::UnsupportedSubjectKind);
    }

    let subject_key = json_string_field(object, "subject_key");
    match &subject_key {
        None => reasons.push(GeneratedArtifactProviderMappingReason::MissingSubjectKey),
        Some(value) if value.len() > 512 => {
            reasons.push(GeneratedArtifactProviderMappingReason::InvalidSubjectKey);
        }
        Some(_) => {}
    }

    Some(GeneratedArtifactProviderSubjectProposal {
        provider,
        provider_name,
        subject_kind,
        subject_kind_name,
        subject_key,
        title: json_string_field(object, "title"),
        release_year: object
            .get("release_year")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        locale: json_string_field(object, "locale"),
        confidence_milli: object
            .get("confidence_milli")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u16::try_from(value.min(1_000)).ok()),
        reasons,
    })
}

fn parse_generated_artifact_external_provider(value: &str) -> Option<ExternalProvider> {
    match value {
        "tmdb" => Some(ExternalProvider::Tmdb),
        "douban" => Some(ExternalProvider::Douban),
        "bangumi" => Some(ExternalProvider::Bangumi),
        "imdb" => Some(ExternalProvider::Imdb),
        _ => None,
    }
}

fn parse_generated_artifact_provider_subject_kind(value: &str) -> Option<ProviderSubjectKind> {
    match value {
        "movie" => Some(ProviderSubjectKind::Movie),
        "series" => Some(ProviderSubjectKind::Series),
        "season" => Some(ProviderSubjectKind::Season),
        "episode" => Some(ProviderSubjectKind::Episode),
        "collection" => Some(ProviderSubjectKind::Collection),
        "subject" => Some(ProviderSubjectKind::Subject),
        "person" => Some(ProviderSubjectKind::Person),
        _ => None,
    }
}

fn json_string_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Option<String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .and_then(|value| non_empty_trimmed(value.to_owned()))
}

fn provider_mapping_action(
    reasons: &[GeneratedArtifactProviderMappingReason],
) -> GeneratedArtifactProviderMappingAction {
    if reasons.iter().any(|reason| {
        matches!(
            reason,
            GeneratedArtifactProviderMappingReason::UnsupportedProvider
                | GeneratedArtifactProviderMappingReason::UnsupportedSubjectKind
                | GeneratedArtifactProviderMappingReason::MissingSubjectKey
                | GeneratedArtifactProviderMappingReason::InvalidSubjectKey
                | GeneratedArtifactProviderMappingReason::DuplicateProposal
                | GeneratedArtifactProviderMappingReason::ExistingRejectedMapping
        )
    }) {
        GeneratedArtifactProviderMappingAction::Skip
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            GeneratedArtifactProviderMappingReason::ExistingAcceptedMapping
        )
    }) {
        GeneratedArtifactProviderMappingAction::Noop
    } else {
        GeneratedArtifactProviderMappingAction::Apply
    }
}

fn count_provider_mapping_actions(
    plans: &[GeneratedArtifactProviderMappingPlan],
) -> (u32, u32, u32) {
    let mut apply_count = 0_u32;
    let mut skipped_count = 0_u32;
    let mut noop_count = 0_u32;

    for plan in plans {
        match plan.action {
            GeneratedArtifactProviderMappingAction::Apply => {
                apply_count = apply_count.saturating_add(1);
            }
            GeneratedArtifactProviderMappingAction::Skip => {
                skipped_count = skipped_count.saturating_add(1);
            }
            GeneratedArtifactProviderMappingAction::Noop => {
                noop_count = noop_count.saturating_add(1);
            }
        }
    }

    (apply_count, skipped_count, noop_count)
}

fn parse_generated_artifact_metadata_patch(
    artifact_json: &str,
    existing: &CanonicalMetadata,
) -> Result<(CanonicalMetadata, Vec<MetadataField>)> {
    let patch: GeneratedArtifactMetadataPatch =
        serde_json::from_str(artifact_json).map_err(|error| NakoError::InvalidInput {
            message: format!("generated artifact metadata payload is not valid JSON: {error}"),
        })?;
    let mut incoming = existing.clone();
    let mut fields = Vec::new();

    if let Some(title) = patch.title.and_then(non_empty_trimmed) {
        incoming.title = title;
        fields.push(MetadataField::Title);
    }
    if let Some(value) = patch.original_title.and_then(normalize_optional_text) {
        incoming.original_title = Some(value);
        fields.push(MetadataField::OriginalTitle);
    }
    if let Some(value) = patch.sort_title.and_then(normalize_optional_text) {
        incoming.sort_title = Some(value);
        fields.push(MetadataField::SortTitle);
    }
    if let Some(value) = patch.overview.and_then(normalize_optional_text) {
        incoming.overview = Some(value);
        fields.push(MetadataField::Overview);
    }
    if let Some(value) = patch.release_date.and_then(normalize_optional_text) {
        incoming.release_date = Some(value);
        fields.push(MetadataField::ReleaseDate);
    }
    if let Some(runtime_minutes) = patch.runtime_minutes {
        if runtime_minutes == 0 {
            return Err(NakoError::InvalidInput {
                message: "generated artifact metadata runtime_minutes must be greater than zero"
                    .to_owned(),
            });
        }
        incoming.runtime_minutes = Some(runtime_minutes);
        fields.push(MetadataField::RuntimeMinutes);
    }
    if let Some(value) = patch.tagline.and_then(normalize_optional_text) {
        incoming.tagline = Some(value);
        fields.push(MetadataField::Tagline);
    }
    if let Some(genres) = patch.genres {
        let normalized = normalize_label_list(genres);
        if !normalized.is_empty() {
            incoming.genres = normalized;
            fields.push(MetadataField::Genres);
        }
    }
    if let Some(tags) = patch.tags {
        let normalized = normalize_label_list(tags);
        if !normalized.is_empty() {
            incoming.tags = normalized;
            fields.push(MetadataField::Tags);
        }
    }

    let fields = dedupe_metadata_fields(fields);
    Ok((incoming, fields))
}

fn summarize_metadata_field(
    metadata: &CanonicalMetadata,
    field: MetadataField,
) -> Result<GeneratedArtifactMetadataValueSummary> {
    match field {
        MetadataField::Title => Ok(text_summary(&metadata.title)),
        MetadataField::OriginalTitle => Ok(option_text_summary(&metadata.original_title)),
        MetadataField::SortTitle => Ok(option_text_summary(&metadata.sort_title)),
        MetadataField::Overview => Ok(option_text_summary(&metadata.overview)),
        MetadataField::ReleaseDate => Ok(option_text_summary(&metadata.release_date)),
        MetadataField::RuntimeMinutes => Ok(option_number_summary(metadata.runtime_minutes)),
        MetadataField::Tagline => Ok(option_text_summary(&metadata.tagline)),
        MetadataField::Genres => Ok(list_summary(&metadata.genres)?),
        MetadataField::Tags => Ok(list_summary(&metadata.tags)?),
        MetadataField::Ratings => Ok(list_summary(&metadata.ratings)?),
        MetadataField::Images => Ok(list_summary(&metadata.images)?),
        MetadataField::Credits => Ok(list_summary(&metadata.credits)?),
        MetadataField::Collections => Ok(list_summary(&metadata.collections)?),
        MetadataField::Studios => Ok(list_summary(&metadata.studios)?),
        MetadataField::ExternalIds => Ok(list_summary(&metadata.external_ids)?),
    }
}

fn locked_metadata_fields(locks: &[MetadataFieldLock]) -> HashSet<MetadataField> {
    locks
        .iter()
        .filter(|lock| lock.locked)
        .map(|lock| lock.field)
        .collect()
}

fn dedupe_metadata_fields(fields: Vec<MetadataField>) -> Vec<MetadataField> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for field in fields {
        if seen.insert(field) {
            deduped.push(field);
        }
    }
    deduped
}

fn normalize_optional_text(value: String) -> Option<String> {
    non_empty_trimmed(value)
}

fn normalize_label_list(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(non_empty_trimmed)
        .filter(|value| seen.insert(value.to_lowercase()))
        .collect()
}

fn non_empty_trimmed(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn option_text_summary(value: &Option<String>) -> GeneratedArtifactMetadataValueSummary {
    match value {
        Some(value) => text_summary(value),
        None => GeneratedArtifactMetadataValueSummary::missing(),
    }
}

fn option_number_summary(value: Option<u32>) -> GeneratedArtifactMetadataValueSummary {
    match value {
        Some(value) => {
            let serialized = value.to_string();
            GeneratedArtifactMetadataValueSummary {
                present: true,
                empty: false,
                value_fingerprint: Some(stable_fingerprint(&serialized)),
                value_bytes: Some(u64::try_from(serialized.len()).unwrap_or(u64::MAX)),
                item_count: None,
            }
        }
        None => GeneratedArtifactMetadataValueSummary::missing(),
    }
}

fn text_summary(value: &str) -> GeneratedArtifactMetadataValueSummary {
    let empty = value.trim().is_empty();
    if empty {
        GeneratedArtifactMetadataValueSummary::missing()
    } else {
        GeneratedArtifactMetadataValueSummary {
            present: true,
            empty: false,
            value_fingerprint: Some(stable_fingerprint(value)),
            value_bytes: Some(u64::try_from(value.len()).unwrap_or(u64::MAX)),
            item_count: None,
        }
    }
}

fn list_summary<T: serde::Serialize>(
    values: &[T],
) -> Result<GeneratedArtifactMetadataValueSummary> {
    if values.is_empty() {
        return Ok(GeneratedArtifactMetadataValueSummary::missing());
    }
    let serialized = serde_json::to_string(values).map_err(|error| NakoError::InvalidInput {
        message: format!("generated artifact metadata field summary serialization failed: {error}"),
    })?;
    Ok(GeneratedArtifactMetadataValueSummary {
        present: true,
        empty: false,
        value_fingerprint: Some(stable_fingerprint(&serialized)),
        value_bytes: Some(u64::try_from(serialized.len()).unwrap_or(u64::MAX)),
        item_count: Some(u32::try_from(values.len()).unwrap_or(u32::MAX)),
    })
}

fn stable_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let prefix = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{prefix}")
}
