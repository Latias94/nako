use async_trait::async_trait;
use taru_api::extension::{
    AutomationArtifactsResponse, AutomationProviderResponse, AutomationProvidersResponse,
    EnqueueAutomationJobRequest, UpsertAutomationProviderRequest,
};
use taru_automation::AutomationJobService;
use taru_core::{
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderId, AutomationRepository, GeneratedArtifactAcceptanceActionKind,
    GeneratedArtifactAcceptanceBoundary, GeneratedArtifactAcceptancePlan,
    GeneratedArtifactAcceptancePlanReason, GeneratedArtifactAcceptancePlanStatus,
    GeneratedArtifactProposal, GeneratedArtifactReviewDecision, GeneratedArtifactReviewResult, Job,
    JobId, JobRepository, MediaItemId, MediaRepository, NewAutomationProviderConfig, PageRequest,
    Result, TaruError,
};
use taru_db::TaruDatabase;

#[derive(Clone, Debug)]
struct UnavailableAutomationProvider;

#[async_trait]
impl taru_automation::AutomationProvider for UnavailableAutomationProvider {
    fn descriptor(&self) -> taru_automation::AutomationProviderDescriptor {
        taru_automation::AutomationProviderDescriptor {
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
        _request: taru_automation::AutomationRequest,
    ) -> Result<taru_automation::AutomationOutcome> {
        Err(TaruError::Provider {
            provider: "automation".to_owned(),
            message: "no concrete automation provider runner is configured".to_owned(),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AutomationAppService {
    store: TaruDatabase,
}

impl AutomationAppService {
    pub(crate) fn new(store: TaruDatabase) -> Self {
        Self { store }
    }

    fn normalize_automation_provider(
        &self,
        request: UpsertAutomationProviderRequest,
    ) -> Result<NewAutomationProviderConfig> {
        let name = request.name.trim().to_owned();
        if name.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "automation provider name cannot be empty".to_owned(),
            });
        }

        let base_url = request.base_url.trim().to_owned();
        if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
            return Err(TaruError::InvalidInput {
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
            return Err(TaruError::InvalidInput {
                message: "automation provider must declare at least one capability".to_owned(),
            });
        }

        let timeout_ms = request.timeout_ms.unwrap_or(30_000);
        if !(100..=120_000).contains(&timeout_ms) {
            return Err(TaruError::InvalidInput {
                message: "automation provider timeout_ms must be between 100 and 120000".to_owned(),
            });
        }

        let max_attempts = request.max_attempts.unwrap_or(2);
        if !(1..=5).contains(&max_attempts) {
            return Err(TaruError::InvalidInput {
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
            .ok_or_else(|| TaruError::NotFound {
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
            .map_err(|err| TaruError::InvalidInput {
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
            .ok_or_else(|| TaruError::NotFound {
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
            .ok_or_else(|| TaruError::NotFound {
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
            .ok_or_else(|| TaruError::NotFound {
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
            return Err(TaruError::InvalidInput {
                message: format!(
                    "cannot change reviewed generated artifact {} from {:?} to {:?}",
                    artifact_id, existing.status, target_status
                ),
            });
        }
        if !plan.status.executable() {
            return Err(TaruError::InvalidInput {
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

    async fn generated_artifact_proposal(
        &self,
        artifact_id: AutomationArtifactId,
    ) -> Result<GeneratedArtifactProposal> {
        self.store
            .list_generated_artifact_proposals(PageRequest::new(PageRequest::MAX_LIMIT, 0))
            .await?
            .into_iter()
            .find(|proposal| proposal.id == artifact_id)
            .ok_or_else(|| TaruError::NotFound {
                entity: "generated_artifact_proposal",
                id: artifact_id.to_string(),
            })
    }
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
    } else if proposal.readiness.status == taru_core::GeneratedArtifactReadinessStatus::Stale {
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
