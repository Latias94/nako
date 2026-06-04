use nako_core::{
    LibraryId, LibraryItemRepository, MediaItem, MediaItemId, MediaKind, MediaRepository,
    MetadataCandidateGraph, MetadataCandidateRelationshipKind,
    MetadataCandidateReviewApplicationAction, MetadataCandidateReviewApplicationPlan,
    MetadataCandidateReviewApplicationReason, MetadataCandidateReviewId,
    MetadataCandidateReviewNode, MetadataCandidateReviewPlan, MetadataCandidateReviewRecord,
    MetadataCandidateReviewRelatedHierarchyApplicationAction,
    MetadataCandidateReviewRelatedHierarchyApplicationPlan,
    MetadataCandidateReviewRelatedHierarchyApplicationReason,
    MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRelatedHierarchyApplicationPlanRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRelatedHierarchyApplicationPlanSummary {
    pub review: MetadataCandidateReviewRecord,
    pub plan: MetadataCandidateReviewRelatedHierarchyApplicationPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRelatedHierarchyApplicationRequest {
    pub review_id: MetadataCandidateReviewId,
    pub item_id: MediaItemId,
    pub applied_at_ms: i64,
    pub expected_updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataCandidateReviewRelatedHierarchyApplicationSummary {
    pub review: MetadataCandidateReviewRecord,
    pub plan: MetadataCandidateReviewRelatedHierarchyApplicationPlan,
    pub provider_subjects: Vec<ProviderSubject>,
    pub provider_mappings: Vec<ProviderMapping>,
    pub confirmed_item_ids: Vec<MediaItemId>,
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

impl<R> MetadataCandidateReviewApplicationService<R>
where
    R: LibraryItemRepository
        + MediaRepository
        + MetadataCandidateReviewRepository
        + ProviderMappingRepository,
{
    pub async fn plan_related_hierarchy(
        &self,
        request: MetadataCandidateReviewRelatedHierarchyApplicationPlanRequest,
    ) -> Result<MetadataCandidateReviewRelatedHierarchyApplicationPlanSummary> {
        let (review, plan, _targets) = self.plan_related_hierarchy_with_targets(request).await?;

        Ok(MetadataCandidateReviewRelatedHierarchyApplicationPlanSummary { review, plan })
    }

    pub async fn apply_related_hierarchy(
        &self,
        request: MetadataCandidateReviewRelatedHierarchyApplicationRequest,
    ) -> Result<MetadataCandidateReviewRelatedHierarchyApplicationSummary> {
        let (review, plan, targets) = self
            .plan_related_hierarchy_with_targets(
                MetadataCandidateReviewRelatedHierarchyApplicationPlanRequest {
                    review_id: request.review_id,
                    item_id: request.item_id,
                    expected_updated_at_ms: request.expected_updated_at_ms,
                },
            )
            .await?;

        match plan.action {
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Skip => {
                return Err(NakoError::Conflict {
                    message: related_hierarchy_skip_message(review.id, &plan.reasons),
                });
            }
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Noop => {
                return Ok(MetadataCandidateReviewRelatedHierarchyApplicationSummary {
                    review,
                    plan,
                    provider_subjects: targets
                        .iter()
                        .filter_map(|target| target.existing_subject.clone())
                        .collect(),
                    provider_mappings: targets
                        .iter()
                        .filter_map(|target| target.existing_mapping.clone())
                        .collect(),
                    confirmed_item_ids: unique_related_item_ids(&targets),
                    changed: false,
                });
            }
            MetadataCandidateReviewRelatedHierarchyApplicationAction::Apply => {}
        }

        let mut summary = MetadataCandidateReviewRelatedHierarchyApplicationSummary {
            review,
            plan,
            provider_subjects: Vec::new(),
            provider_mappings: Vec::new(),
            confirmed_item_ids: Vec::new(),
            changed: false,
        };

        for target in targets {
            let mapping_changed = target.existing_mapping.as_ref().map_or(true, |mapping| {
                mapping.status != ProviderMappingStatus::Accepted
            });
            let provider_subject = if mapping_changed || target.existing_subject.is_none() {
                upsert_provider_subject_for_candidate(&self.repository, target.subject).await?
            } else {
                target.existing_subject.clone().expect("checked above")
            };
            let provider_mapping = ProviderMapping {
                id: target
                    .existing_mapping
                    .as_ref()
                    .map(|mapping| mapping.id)
                    .unwrap_or_else(ProviderMappingId::new),
                item_id: target.item.id,
                subject_id: provider_subject.id,
                status: ProviderMappingStatus::Accepted,
                confidence_milli: None,
                source: target.source.clone(),
            };

            if mapping_changed {
                self.repository
                    .upsert_provider_mapping(&provider_mapping)
                    .await?;
            }

            let mut state_changed = false;
            for library_id in target.provisional_library_ids {
                self.repository
                    .upsert_library_item_state(&nako_core::LibraryItemState {
                        library_id,
                        item_id: target.item.id,
                        provisional: false,
                    })
                    .await?;
                state_changed = true;
            }

            if !summary.confirmed_item_ids.contains(&target.item.id) {
                summary.confirmed_item_ids.push(target.item.id);
            }
            summary.provider_subjects.push(provider_subject);
            summary.provider_mappings.push(provider_mapping);
            summary.changed |= mapping_changed || state_changed;
        }

        Ok(summary)
    }

    async fn plan_related_hierarchy_with_targets(
        &self,
        request: MetadataCandidateReviewRelatedHierarchyApplicationPlanRequest,
    ) -> Result<(
        MetadataCandidateReviewRecord,
        MetadataCandidateReviewRelatedHierarchyApplicationPlan,
        Vec<RelatedHierarchyTarget>,
    )> {
        let review = self
            .repository
            .get_metadata_candidate_review(request.review_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "metadata_candidate_review",
                id: request.review_id.to_string(),
            })?;

        reject_stale_review_operation(&review, request.item_id, request.expected_updated_at_ms)?;

        let root_plan = build_candidate_review_application_plan(&self.repository, &review).await?;
        let preflight_reasons = related_hierarchy_preflight_reasons(&review, &root_plan);
        if !preflight_reasons.is_empty() {
            reject_related_hierarchy_application_preflight(review.id, &preflight_reasons)?;
            let plan = build_related_hierarchy_application_plan(
                &review,
                &root_plan,
                MetadataCandidateReviewRelatedHierarchyApplicationAction::Skip,
                preflight_reasons,
                Vec::new(),
            );
            return Ok((review, plan, Vec::new()));
        }

        let targets =
            resolve_related_hierarchy_targets(&self.repository, &review, &root_plan).await?;
        if targets.is_empty() {
            let plan = build_related_hierarchy_application_plan(
                &review,
                &root_plan,
                MetadataCandidateReviewRelatedHierarchyApplicationAction::Skip,
                vec![
                    MetadataCandidateReviewRelatedHierarchyApplicationReason::NoSafeRelatedHierarchyRelationships,
                ],
                Vec::new(),
            );
            return Ok((review, plan, Vec::new()));
        }

        let target_plans = targets
            .iter()
            .map(related_hierarchy_target_plan)
            .collect::<Vec<_>>();
        let mutation_required = target_plans.iter().any(|target| {
            target.mapping_change_required || target.provisional_library_state_count > 0
        });
        let (action, reasons) = if mutation_required {
            (
                MetadataCandidateReviewRelatedHierarchyApplicationAction::Apply,
                vec![MetadataCandidateReviewRelatedHierarchyApplicationReason::Ready],
            )
        } else {
            (
                MetadataCandidateReviewRelatedHierarchyApplicationAction::Noop,
                vec![MetadataCandidateReviewRelatedHierarchyApplicationReason::AlreadyApplied],
            )
        };
        let plan = build_related_hierarchy_application_plan(
            &review,
            &root_plan,
            action,
            reasons,
            target_plans,
        );

        Ok((review, plan, targets))
    }
}

#[derive(Clone, Debug)]
struct RelatedHierarchyTarget {
    item: MediaItem,
    library_ids: Vec<LibraryId>,
    provisional_library_ids: Vec<LibraryId>,
    subject: MetadataCandidateSubject,
    source: MetadataSource,
    existing_subject: Option<ProviderSubject>,
    existing_mapping: Option<ProviderMapping>,
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

fn related_hierarchy_preflight_reasons(
    review: &MetadataCandidateReviewRecord,
    plan: &MetadataCandidateReviewApplicationPlan,
) -> Vec<MetadataCandidateReviewRelatedHierarchyApplicationReason> {
    let mut reasons = Vec::new();

    if review.status != MetadataCandidateReviewStatus::Accepted {
        reasons.push(MetadataCandidateReviewRelatedHierarchyApplicationReason::ReviewNotAccepted);
    }
    if plan.source.is_none() {
        reasons.push(MetadataCandidateReviewRelatedHierarchyApplicationReason::UnsupportedSource);
    }
    if plan.root_subject.is_none() {
        reasons.push(MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingRootSubject);
    }
    if plan.existing_mapping_status != Some(ProviderMappingStatus::Accepted) {
        reasons.push(
            MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingAcceptedRootMapping,
        );
    }

    reasons
}

fn reject_related_hierarchy_application_preflight(
    review_id: MetadataCandidateReviewId,
    reasons: &[MetadataCandidateReviewRelatedHierarchyApplicationReason],
) -> Result<()> {
    if reasons
        .contains(&MetadataCandidateReviewRelatedHierarchyApplicationReason::ReviewNotAccepted)
        || reasons.contains(
            &MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingAcceptedRootMapping,
        )
    {
        return Err(NakoError::Conflict {
            message: related_hierarchy_skip_message(review_id, reasons),
        });
    }

    Ok(())
}

fn build_related_hierarchy_application_plan(
    review: &MetadataCandidateReviewRecord,
    root_plan: &MetadataCandidateReviewApplicationPlan,
    action: MetadataCandidateReviewRelatedHierarchyApplicationAction,
    reasons: Vec<MetadataCandidateReviewRelatedHierarchyApplicationReason>,
    targets: Vec<MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan>,
) -> MetadataCandidateReviewRelatedHierarchyApplicationPlan {
    let mapping_change_count = targets
        .iter()
        .filter(|target| target.mapping_change_required)
        .count();
    let provisional_state_change_count = targets.iter().fold(0_u32, |total, target| {
        total.saturating_add(target.provisional_library_state_count)
    });

    MetadataCandidateReviewRelatedHierarchyApplicationPlan {
        review_id: review.id,
        item_id: review.item_id,
        action,
        reasons,
        source: root_plan.source.clone(),
        root_subject: root_plan.root_subject.clone(),
        root_mapping_id: root_plan.existing_mapping_id,
        root_mapping_status: root_plan.existing_mapping_status,
        target_count: saturating_u32_len(targets.len()),
        mapping_change_count: saturating_u32_len(mapping_change_count),
        provisional_state_change_count,
        targets,
    }
}

fn related_hierarchy_target_plan(
    target: &RelatedHierarchyTarget,
) -> MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan {
    let existing_mapping_status = target
        .existing_mapping
        .as_ref()
        .map(|mapping| mapping.status);

    MetadataCandidateReviewRelatedHierarchyApplicationTargetPlan {
        item_id: target.item.id,
        library_ids: target.library_ids.clone(),
        subject: target.subject.clone(),
        source: target.source.clone(),
        existing_subject_id: target.existing_subject.as_ref().map(|subject| subject.id),
        existing_mapping_id: target.existing_mapping.as_ref().map(|mapping| mapping.id),
        existing_mapping_status,
        mapping_change_required: existing_mapping_status != Some(ProviderMappingStatus::Accepted),
        provisional_library_state_count: saturating_u32_len(target.provisional_library_ids.len()),
    }
}

fn unique_related_item_ids(targets: &[RelatedHierarchyTarget]) -> Vec<MediaItemId> {
    let mut item_ids = Vec::new();
    for target in targets {
        if !item_ids.contains(&target.item.id) {
            item_ids.push(target.item.id);
        }
    }

    item_ids
}

fn related_hierarchy_skip_message(
    review_id: MetadataCandidateReviewId,
    reasons: &[MetadataCandidateReviewRelatedHierarchyApplicationReason],
) -> String {
    if reasons
        .contains(&MetadataCandidateReviewRelatedHierarchyApplicationReason::ReviewNotAccepted)
    {
        return format!(
            "metadata candidate review {review_id} cannot apply related hierarchy before it is accepted"
        );
    }
    if reasons
        .contains(&MetadataCandidateReviewRelatedHierarchyApplicationReason::UnsupportedSource)
    {
        return format!(
            "metadata candidate review {review_id} has no supported application source"
        );
    }
    if reasons
        .contains(&MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingRootSubject)
    {
        return format!("metadata candidate review {review_id} has no root provider subject");
    }
    if reasons.contains(
        &MetadataCandidateReviewRelatedHierarchyApplicationReason::MissingAcceptedRootMapping,
    ) {
        return format!(
            "metadata candidate review {review_id} requires an accepted root provider mapping before related hierarchy application"
        );
    }
    if reasons.contains(
        &MetadataCandidateReviewRelatedHierarchyApplicationReason::NoSafeRelatedHierarchyRelationships,
    ) {
        return format!(
            "metadata candidate review {review_id} has no safe related hierarchy relationships"
        );
    }

    format!("metadata candidate review {review_id} cannot apply related hierarchy: {reasons:?}")
}

fn saturating_u32_len(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

async fn resolve_related_hierarchy_targets<R>(
    repository: &R,
    review: &MetadataCandidateReviewRecord,
    plan: &MetadataCandidateReviewApplicationPlan,
) -> Result<Vec<RelatedHierarchyTarget>>
where
    R: LibraryItemRepository + MediaRepository + ProviderMappingRepository,
{
    let source = plan.source.clone().ok_or_else(|| NakoError::Conflict {
        message: format!(
            "metadata candidate review {} has no supported application source",
            review.id
        ),
    })?;
    let root_subject = plan
        .root_subject
        .as_ref()
        .ok_or_else(|| NakoError::Conflict {
            message: format!(
                "metadata candidate review {} has no root provider subject",
                review.id
            ),
        })?;
    let mut targets = Vec::new();
    let mut seen_child_subjects: Vec<MetadataCandidateSubject> = Vec::new();

    for relationship in &review.plan.relationships {
        if relationship.kind != MetadataCandidateRelationshipKind::Contains {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} contains unsupported related hierarchy relationship {:?}",
                    review.id, relationship.kind
                ),
            });
        }
        if relationship.parent_subject != *root_subject {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} related hierarchy must be anchored at the accepted root provider subject",
                    review.id
                ),
            });
        }
        if seen_child_subjects
            .iter()
            .any(|subject| subject == &relationship.child_subject)
        {
            continue;
        }
        seen_child_subjects.push(relationship.child_subject.clone());

        let related_node = single_related_node_for_subject(review, &relationship.child_subject)?;
        reject_unsafe_related_hierarchy_shape(review, related_node)?;
        let node_source = metadata_source_from_candidate_source(&related_node.source).ok_or_else(
            || NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} related node has no supported application source",
                    review.id
                ),
            },
        )?;
        if node_source != source {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} related node source does not match the accepted root source",
                    review.id
                ),
            });
        }

        let (item, library_ids) =
            resolve_single_related_item(repository, review, related_node).await?;
        let provisional_library_ids =
            provisional_related_library_ids(repository, &library_ids, item.id).await?;
        let existing_subject =
            existing_provider_subject_for_candidate(repository, &relationship.child_subject)
                .await?;
        let existing_mapping = existing_related_mapping_for_target(
            repository,
            review,
            &relationship.child_subject,
            item.id,
            existing_subject.as_ref(),
        )
        .await?;

        targets.push(RelatedHierarchyTarget {
            item,
            library_ids,
            provisional_library_ids,
            subject: relationship.child_subject.clone(),
            source: source.clone(),
            existing_subject,
            existing_mapping,
        });
    }

    Ok(targets)
}

async fn provisional_related_library_ids<R>(
    repository: &R,
    library_ids: &[LibraryId],
    item_id: MediaItemId,
) -> Result<Vec<LibraryId>>
where
    R: LibraryItemRepository,
{
    let mut provisional_library_ids = Vec::new();
    for library_id in library_ids {
        if repository
            .get_library_item_state(*library_id, item_id)
            .await?
            .is_some_and(|state| state.provisional)
        {
            provisional_library_ids.push(*library_id);
        }
    }

    Ok(provisional_library_ids)
}

fn single_related_node_for_subject<'a>(
    review: &'a MetadataCandidateReviewRecord,
    subject: &MetadataCandidateSubject,
) -> Result<&'a MetadataCandidateReviewNode> {
    let matches = review
        .plan
        .related
        .iter()
        .filter(|node| node.subject.as_ref() == Some(subject))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [node] => Ok(*node),
        [] => Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} relationship references a missing related provider subject",
                review.id
            ),
        }),
        _ => Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} has ambiguous related nodes for one provider subject",
                review.id
            ),
        }),
    }
}

fn reject_unsafe_related_hierarchy_shape(
    review: &MetadataCandidateReviewRecord,
    related_node: &MetadataCandidateReviewNode,
) -> Result<()> {
    match (review.plan.root.kind, related_node.kind) {
        (MediaKind::Series, MediaKind::Season | MediaKind::Episode)
        | (MediaKind::Season, MediaKind::Episode) => Ok(()),
        _ => Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} cannot safely apply {:?} -> {:?} related hierarchy",
                review.id, review.plan.root.kind, related_node.kind
            ),
        }),
    }
}

async fn resolve_single_related_item<R>(
    repository: &R,
    review: &MetadataCandidateReviewRecord,
    related_node: &MetadataCandidateReviewNode,
) -> Result<(MediaItem, Vec<LibraryId>)>
where
    R: LibraryItemRepository + MediaRepository,
{
    let title = related_node
        .metadata
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or_else(|| NakoError::Conflict {
            message: format!(
                "metadata candidate review {} related hierarchy node has no title for safe item matching",
                review.id
            ),
        })?;
    let root_states = repository
        .list_library_item_states_for_item(review.item_id)
        .await?;
    if root_states.is_empty() {
        return Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} root item has no library state for related hierarchy matching",
                review.id
            ),
        });
    }

    let mut matches: Vec<(LibraryId, MediaItem)> = Vec::new();
    for state in root_states {
        matches.extend(
            list_related_item_matches_for_library(
                repository,
                state.library_id,
                review.item_id,
                related_node.kind,
                title,
            )
            .await?,
        );
    }

    if matches.is_empty() {
        return Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} cannot find an existing related media item for safe hierarchy application",
                review.id
            ),
        });
    }

    let first_item = matches[0].1.clone();
    if matches.iter().any(|(_, item)| item.id != first_item.id) {
        return Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} related hierarchy target is ambiguous",
                review.id
            ),
        });
    }

    let mut library_ids = Vec::new();
    for (library_id, _) in matches {
        if !library_ids.contains(&library_id) {
            library_ids.push(library_id);
        }
    }

    Ok((first_item, library_ids))
}

async fn list_related_item_matches_for_library<R>(
    repository: &R,
    library_id: LibraryId,
    parent_id: MediaItemId,
    kind: MediaKind,
    title: &str,
) -> Result<Vec<(LibraryId, MediaItem)>>
where
    R: MediaRepository,
{
    let mut matches = Vec::new();
    let mut offset = 0;
    loop {
        let items = repository
            .list_media_items_for_library(
                library_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let returned = items.len();
        matches.extend(items.into_iter().filter_map(|item| {
            (item.kind == kind
                && item.parent_id == Some(parent_id)
                && item.metadata.title.trim() == title)
                .then_some((library_id, item))
        }));
        if returned < PageRequest::MAX_LIMIT as usize {
            break;
        }
        offset += u64::from(PageRequest::MAX_LIMIT);
    }

    Ok(matches)
}

async fn existing_related_mapping_for_target<R>(
    repository: &R,
    review: &MetadataCandidateReviewRecord,
    subject: &MetadataCandidateSubject,
    item_id: MediaItemId,
    existing_subject: Option<&ProviderSubject>,
) -> Result<Option<ProviderMapping>>
where
    R: ProviderMappingRepository,
{
    let Some(existing_subject) = existing_subject else {
        return Ok(None);
    };
    let mappings = list_all_provider_mappings_for_subject(repository, existing_subject.id).await?;
    let mut target_mapping = None;

    for mapping in mappings {
        if mapping.item_id == item_id {
            if mapping.status == ProviderMappingStatus::Rejected {
                return Err(NakoError::Conflict {
                    message: format!(
                        "metadata candidate review {} related provider subject {} has a rejected mapping on the target item",
                        review.id, subject.subject_key
                    ),
                });
            }
            target_mapping = Some(mapping);
        } else if mapping.status != ProviderMappingStatus::Rejected {
            return Err(NakoError::Conflict {
                message: format!(
                    "metadata candidate review {} related provider subject {} is already mapped to a different item",
                    review.id, subject.subject_key
                ),
            });
        }
    }

    Ok(target_mapping)
}

async fn list_all_provider_mappings_for_subject<R>(
    repository: &R,
    subject_id: ProviderSubjectId,
) -> Result<Vec<ProviderMapping>>
where
    R: ProviderMappingRepository,
{
    let mut mappings = Vec::new();
    let mut offset = 0;
    loop {
        let page = repository
            .list_provider_mappings_for_subject(
                subject_id,
                PageRequest {
                    limit: PageRequest::MAX_LIMIT,
                    offset,
                },
            )
            .await?;
        let returned = page.len();
        mappings.extend(page);
        if returned < PageRequest::MAX_LIMIT as usize {
            return Ok(mappings);
        }
        offset += u64::from(PageRequest::MAX_LIMIT);
    }
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
