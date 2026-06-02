use nako_core::{
    MediaItemId, MetadataCandidateGraph, MetadataCandidateReviewApplicationAction,
    MetadataCandidateReviewApplicationPlan, MetadataCandidateReviewApplicationReason,
    MetadataCandidateReviewId, MetadataCandidateReviewPlan, MetadataCandidateReviewRecord,
    MetadataCandidateReviewRepository, MetadataCandidateReviewStatus, MetadataCandidateSource,
    MetadataCandidateSubject, MetadataSource, NakoError, PageRequest, ProviderMapping,
    ProviderMappingId, ProviderMappingRepository, ProviderMappingStatus, ProviderSubject,
    ProviderSubjectId, Result,
};
use serde::{Deserialize, Serialize};

#[must_use]
pub fn build_candidate_review_plan(graph: &MetadataCandidateGraph) -> MetadataCandidateReviewPlan {
    MetadataCandidateReviewPlan::from_graph(graph)
}

pub async fn build_candidate_review_application_plan<R>(
    repository: &R,
    review: &MetadataCandidateReviewRecord,
) -> Result<MetadataCandidateReviewApplicationPlan>
where
    R: ProviderMappingRepository,
{
    let root_subject = review.plan.root.subject.clone();
    let source = metadata_source_from_candidate_source(&review.source);
    let existing_mapping = match root_subject.as_ref() {
        Some(subject) => existing_mapping_for_subject(repository, review.item_id, subject).await?,
        None => None,
    };
    let existing_mapping_id = existing_mapping.as_ref().map(|mapping| mapping.id);
    let existing_mapping_status = existing_mapping.as_ref().map(|mapping| mapping.status);

    let mut reasons = Vec::new();
    if review.status != MetadataCandidateReviewStatus::Accepted {
        reasons.push(MetadataCandidateReviewApplicationReason::ReviewNotAccepted);
    }
    if root_subject.is_none() {
        reasons.push(MetadataCandidateReviewApplicationReason::MissingRootSubject);
    }
    if source.is_none() {
        reasons.push(MetadataCandidateReviewApplicationReason::UnsupportedSource);
    }
    match existing_mapping_status {
        Some(ProviderMappingStatus::Accepted) => {
            reasons.push(MetadataCandidateReviewApplicationReason::ExistingAcceptedMapping);
        }
        Some(ProviderMappingStatus::Candidate) => {
            reasons.push(MetadataCandidateReviewApplicationReason::ExistingCandidateMapping);
        }
        Some(ProviderMappingStatus::Rejected) => {
            reasons.push(MetadataCandidateReviewApplicationReason::ExistingRejectedMapping);
        }
        None => {}
    }

    let action = if review.status != MetadataCandidateReviewStatus::Accepted
        || root_subject.is_none()
        || source.is_none()
        || existing_mapping_status == Some(ProviderMappingStatus::Rejected)
    {
        MetadataCandidateReviewApplicationAction::Skip
    } else if existing_mapping_status == Some(ProviderMappingStatus::Accepted) {
        MetadataCandidateReviewApplicationAction::Noop
    } else {
        reasons.push(MetadataCandidateReviewApplicationReason::Ready);
        MetadataCandidateReviewApplicationAction::Apply
    };

    Ok(MetadataCandidateReviewApplicationPlan {
        review_id: review.id,
        item_id: review.item_id,
        action,
        reasons,
        source,
        root_subject,
        existing_mapping_id,
        existing_mapping_status,
    })
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateReviewDecision {
    Accept,
    Reject,
}

impl MetadataCandidateReviewDecision {
    const fn target_status(self) -> MetadataCandidateReviewStatus {
        match self {
            Self::Accept => MetadataCandidateReviewStatus::Accepted,
            Self::Reject => MetadataCandidateReviewStatus::Rejected,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewDecisionRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub decision: MetadataCandidateReviewDecision,
    pub decided_at_ms: i64,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewDecisionSummary {
    pub review: MetadataCandidateReviewRecord,
    pub changed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewApplicationPlanRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewApplicationPlanSummary {
    pub review: MetadataCandidateReviewRecord,
    pub plan: MetadataCandidateReviewApplicationPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewApplicationRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub applied_at_ms: i64,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewApplicationSummary {
    pub review: MetadataCandidateReviewRecord,
    pub plan: MetadataCandidateReviewApplicationPlan,
    pub provider_subject: Option<ProviderSubject>,
    pub provider_mapping: Option<ProviderMapping>,
    pub changed: bool,
}

#[derive(Debug)]
pub struct MetadataCandidateReviewDecisionService<R> {
    repository: R,
}

#[derive(Debug)]
pub struct MetadataCandidateReviewApplicationService<R> {
    repository: R,
}

impl<R> MetadataCandidateReviewDecisionService<R> {
    #[must_use]
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> MetadataCandidateReviewApplicationService<R> {
    #[must_use]
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }
}

impl<R> MetadataCandidateReviewDecisionService<R>
where
    R: MetadataCandidateReviewRepository,
{
    pub async fn decide(
        &self,
        request: MetadataCandidateReviewDecisionRequest,
    ) -> Result<MetadataCandidateReviewDecisionSummary> {
        let review = self
            .repository
            .get_metadata_candidate_review(request.review_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "metadata_candidate_review",
                id: request.review_id.to_string(),
            })?;

        reject_stale_review_operation(&review, request.item_id, request.expected_updated_at_ms)?;

        let target_status = request.decision.target_status();
        if review.status == target_status {
            return Ok(MetadataCandidateReviewDecisionSummary {
                review,
                changed: false,
            });
        }

        if review.status == MetadataCandidateReviewStatus::Pending
            && review
                .expires_at_ms
                .is_some_and(|expires_at_ms| expires_at_ms <= request.decided_at_ms)
        {
            self.repository
                .set_metadata_candidate_review_status(
                    review.id,
                    MetadataCandidateReviewStatus::Expired,
                    request.decided_at_ms,
                )
                .await?
                .ok_or_else(|| NakoError::NotFound {
                    entity: "metadata_candidate_review",
                    id: review.id.to_string(),
                })?;

            return Err(NakoError::Conflict {
                message: format!("metadata candidate review {} is expired", review.id),
            });
        }

        if review.status != MetadataCandidateReviewStatus::Pending {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} is already {}",
                    review.id,
                    review.status.as_str()
                ),
            });
        }

        let updated = self
            .repository
            .set_metadata_candidate_review_status(review.id, target_status, request.decided_at_ms)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "metadata_candidate_review",
                id: review.id.to_string(),
            })?;

        Ok(MetadataCandidateReviewDecisionSummary {
            review: updated,
            changed: true,
        })
    }
}

impl<R> MetadataCandidateReviewApplicationService<R>
where
    R: MetadataCandidateReviewRepository + ProviderMappingRepository,
{
    pub async fn plan(
        &self,
        request: MetadataCandidateReviewApplicationPlanRequest,
    ) -> Result<MetadataCandidateReviewApplicationPlanSummary> {
        let review = self
            .repository
            .get_metadata_candidate_review(request.review_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "metadata_candidate_review",
                id: request.review_id.to_string(),
            })?;

        reject_stale_review_operation(&review, request.item_id, request.expected_updated_at_ms)?;

        let plan = build_candidate_review_application_plan(&self.repository, &review).await?;

        Ok(MetadataCandidateReviewApplicationPlanSummary { review, plan })
    }

    pub async fn apply(
        &self,
        request: MetadataCandidateReviewApplicationRequest,
    ) -> Result<MetadataCandidateReviewApplicationSummary> {
        let planned = self
            .plan(MetadataCandidateReviewApplicationPlanRequest {
                review_id: request.review_id,
                item_id: request.item_id,
                expected_updated_at_ms: request.expected_updated_at_ms,
            })
            .await?;
        let review = planned.review;
        let plan = planned.plan;
        match plan.action {
            MetadataCandidateReviewApplicationAction::Skip => {
                return Err(NakoError::Conflict {
                    message: format!(
                        "metadata candidate review {} cannot be applied: {:?}",
                        review.id, plan.reasons
                    ),
                });
            }
            MetadataCandidateReviewApplicationAction::Noop => {
                let provider_subject = match plan.root_subject.as_ref() {
                    Some(subject) => {
                        existing_provider_subject_for_candidate(&self.repository, subject).await?
                    }
                    None => None,
                };
                let provider_mapping = match plan.root_subject.as_ref() {
                    Some(subject) => {
                        existing_mapping_for_subject(&self.repository, review.item_id, subject)
                            .await?
                    }
                    None => None,
                };
                return Ok(MetadataCandidateReviewApplicationSummary {
                    review,
                    plan,
                    provider_subject,
                    provider_mapping,
                    changed: false,
                });
            }
            MetadataCandidateReviewApplicationAction::Apply => {}
        }

        let source = plan.source.clone().ok_or_else(|| NakoError::Conflict {
            message: format!(
                "metadata candidate review {} has no supported application source",
                review.id
            ),
        })?;
        let root_subject = plan
            .root_subject
            .clone()
            .ok_or_else(|| NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} has no root provider subject",
                    review.id
                ),
            })?;
        let provider_subject =
            upsert_provider_subject_for_candidate(&self.repository, root_subject).await?;
        let provider_mapping = ProviderMapping {
            id: plan
                .existing_mapping_id
                .unwrap_or_else(ProviderMappingId::new),
            item_id: review.item_id,
            subject_id: provider_subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: None,
            source,
        };
        self.repository
            .upsert_provider_mapping(&provider_mapping)
            .await?;

        Ok(MetadataCandidateReviewApplicationSummary {
            review,
            plan,
            provider_subject: Some(provider_subject),
            provider_mapping: Some(provider_mapping),
            changed: true,
        })
    }
}

fn reject_stale_review_operation(
    review: &MetadataCandidateReviewRecord,
    item_id: MediaItemId,
    expected_updated_at_ms: Option<i64>,
) -> Result<()> {
    if review.item_id != item_id {
        return Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} belongs to item {}, not {}",
                review.id, review.item_id, item_id
            ),
        });
    }

    if let Some(expected_updated_at_ms) = expected_updated_at_ms {
        if review.updated_at_ms != expected_updated_at_ms {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} changed from {} to {} before decision",
                    review.id, expected_updated_at_ms, review.updated_at_ms
                ),
            });
        }
    }

    Ok(())
}

fn metadata_source_from_candidate_source(
    source: &MetadataCandidateSource,
) -> Option<MetadataSource> {
    match source {
        MetadataCandidateSource::Local => Some(MetadataSource::Local),
        MetadataCandidateSource::Nfo => Some(MetadataSource::Nfo),
        MetadataCandidateSource::Provider(provider) => {
            Some(MetadataSource::Provider(provider.clone()))
        }
        MetadataCandidateSource::Addon(addon_id) => Some(MetadataSource::Addon(*addon_id)),
        MetadataCandidateSource::User => Some(MetadataSource::User),
        MetadataCandidateSource::Automation(_) | MetadataCandidateSource::Other(_) => None,
    }
}

async fn existing_mapping_for_subject<R>(
    repository: &R,
    item_id: MediaItemId,
    subject: &MetadataCandidateSubject,
) -> Result<Option<ProviderMapping>>
where
    R: ProviderMappingRepository,
{
    let Some(provider_subject) = repository
        .find_provider_subject(
            &subject.provider,
            &subject.subject_kind,
            &subject.subject_key,
        )
        .await?
    else {
        return Ok(None);
    };

    let mut offset = 0;
    loop {
        let mappings = repository
            .list_provider_mappings_for_item(
                item_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let returned = mappings.len();
        if let Some(mapping) = mappings
            .into_iter()
            .find(|mapping| mapping.subject_id == provider_subject.id)
        {
            return Ok(Some(mapping));
        }
        if returned < PageRequest::MAX_LIMIT as usize {
            return Ok(None);
        }
        offset += u64::from(PageRequest::MAX_LIMIT);
    }
}

async fn existing_provider_subject_for_candidate<R>(
    repository: &R,
    subject: &MetadataCandidateSubject,
) -> Result<Option<ProviderSubject>>
where
    R: ProviderMappingRepository,
{
    repository
        .find_provider_subject(
            &subject.provider,
            &subject.subject_kind,
            &subject.subject_key,
        )
        .await
}

async fn upsert_provider_subject_for_candidate<R>(
    repository: &R,
    subject: MetadataCandidateSubject,
) -> Result<ProviderSubject>
where
    R: ProviderMappingRepository,
{
    let existing = existing_provider_subject_for_candidate(repository, &subject).await?;
    let provider_subject = subject.into_provider_subject(
        existing
            .as_ref()
            .map(|subject| subject.id)
            .unwrap_or_else(ProviderSubjectId::new),
    );
    repository
        .upsert_provider_subject(&provider_subject)
        .await?;
    Ok(provider_subject)
}
