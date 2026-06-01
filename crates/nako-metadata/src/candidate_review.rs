use nako_core::{
    MediaItemId, MetadataCandidateGraph, MetadataCandidateReviewId, MetadataCandidateReviewPlan,
    MetadataCandidateReviewRecord, MetadataCandidateReviewRepository,
    MetadataCandidateReviewStatus, NakoError, Result,
};
use serde::{Deserialize, Serialize};

#[must_use]
pub fn build_candidate_review_plan(graph: &MetadataCandidateGraph) -> MetadataCandidateReviewPlan {
    MetadataCandidateReviewPlan::from_graph(graph)
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

#[derive(Debug)]
pub struct MetadataCandidateReviewDecisionService<R> {
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

        reject_stale_decision(&review, &request)?;

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

fn reject_stale_decision(
    review: &MetadataCandidateReviewRecord,
    request: &MetadataCandidateReviewDecisionRequest,
) -> Result<()> {
    if review.item_id != request.item_id {
        return Err(NakoError::Conflict {
            message: format!(
                "metadata candidate review {} belongs to item {}, not {}",
                review.id, review.item_id, request.item_id
            ),
        });
    }

    if let Some(expected_updated_at_ms) = request.expected_updated_at_ms {
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
