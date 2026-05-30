use std::collections::HashSet;

use async_trait::async_trait;
use nako_api::extension::{
    AutomationArtifactsResponse, AutomationProviderResponse, AutomationProvidersResponse,
    EnqueueAutomationJobRequest, UpsertAutomationProviderRequest,
};
use nako_automation::AutomationJobService;
use nako_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderId, AutomationRepository, CanonicalMetadata,
    GeneratedArtifactAcceptanceActionKind, GeneratedArtifactAcceptanceBoundary,
    GeneratedArtifactAcceptancePlan, GeneratedArtifactAcceptancePlanReason,
    GeneratedArtifactAcceptancePlanStatus, GeneratedArtifactMetadataApplyFieldPlan,
    GeneratedArtifactMetadataApplyOutcomeCommit, GeneratedArtifactMetadataApplyOutcomeId,
    GeneratedArtifactMetadataApplyOutcomeRecord, GeneratedArtifactMetadataApplyOutcomeStatus,
    GeneratedArtifactMetadataApplyPlan, GeneratedArtifactMetadataApplyPlanReason,
    GeneratedArtifactMetadataApplyPlanStatus, GeneratedArtifactMetadataApplyRequest,
    GeneratedArtifactMetadataApplyResult, GeneratedArtifactMetadataApplyResultStatus,
    GeneratedArtifactMetadataFieldAction, GeneratedArtifactMetadataFieldReason,
    GeneratedArtifactMetadataValueSummary, GeneratedArtifactProposal,
    GeneratedArtifactReviewDecision, GeneratedArtifactReviewResult, Job, JobId, JobRepository,
    LibraryRepository, MediaItem, MediaItemId, MediaRepository,
    MetadataApplicationPersistenceCommit, MetadataField, MetadataFieldLock, MetadataMergePolicy,
    MetadataRepository, MetadataSource, NakoError, NewAutomationProviderConfig, PageRequest,
    Result,
};
use nako_db::NakoDatabase;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::app::metadata_application::{
    MetadataApplication, MetadataApplicationCommand, MetadataApplicationLockScope,
    MetadataApplicationMode, MetadataApplicationProvenance,
};

#[derive(Clone, Debug)]
struct UnavailableAutomationProvider;

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
}

impl AutomationAppService {
    pub(crate) fn new(store: NakoDatabase) -> Self {
        Self { store }
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
                    0,
                    0,
                    0,
                ));
            }
        };
        if suggested_fields.is_empty() {
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

        if apply_field_count == 0 && status.executable() {
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
            apply_field_count,
            skipped_field_count,
            noop_field_count,
        ))
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
        let changed = applied.changed;
        let applied_source = applied.applied_source.clone();
        let item_id = applied.item.id;

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
                    applied_source: Some(applied_source),
                    item_id: Some(item_id),
                    plan,
                    error_code: None,
                    error_message: None,
                    metadata_application: Some(MetadataApplicationPersistenceCommit {
                        item: applied.item,
                        catalog_projection: applied.projection,
                    }),
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
                },
            )
            .await?;

        generated_artifact_metadata_apply_result_from_outcome(outcome, false)
    }

    fn generated_artifact_metadata_apply_plan(
        &self,
        proposal: GeneratedArtifactProposal,
        status: GeneratedArtifactMetadataApplyPlanStatus,
        reasons: Vec<GeneratedArtifactMetadataApplyPlanReason>,
        fields: Vec<GeneratedArtifactMetadataApplyFieldPlan>,
        apply_field_count: u32,
        skipped_field_count: u32,
        noop_field_count: u32,
    ) -> GeneratedArtifactMetadataApplyPlan {
        GeneratedArtifactMetadataApplyPlan {
            artifact_id: proposal.id,
            status,
            executable: status.executable(),
            reasons,
            target: proposal.target,
            payload: proposal.payload,
            fields,
            apply_field_count,
            skipped_field_count,
            noop_field_count,
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
