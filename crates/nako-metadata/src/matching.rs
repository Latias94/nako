use nako_core::{ExternalProvider, MediaItem, MediaKind};
use serde::{Deserialize, Serialize};

use crate::{MetadataCandidate, MetadataLookup};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidateMatch {
    pub provider: ExternalProvider,
    pub provider_key: String,
    pub media_kind: MediaKind,
    pub title: String,
    pub release_year: Option<u16>,
    pub score: f32,
    pub decision: MetadataCandidateMatchDecision,
    pub reasons: Vec<MetadataCandidateMatchReason>,
    pub message: String,
}

impl MetadataCandidateMatch {
    #[must_use]
    pub fn needs_confirmation(&self) -> bool {
        self.decision == MetadataCandidateMatchDecision::NeedsConfirmation
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateMatchDecision {
    Accepted,
    NeedsConfirmation,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateMatchReason {
    ScoreAccepted,
    ScoreNeedsConfirmation,
    ScoreRejected,
    NearbyHighConfidenceConflict,
    ExactTitle,
    DifferentTitle,
    MissingLookupTitle,
    MissingCandidateTitle,
    ReleaseYearMatch,
    ReleaseYearMismatch,
    MissingLookupYear,
    MissingCandidateReleaseYear,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidateMatchingPolicy {
    pub accept_score: f32,
    pub confirmation_score: f32,
    pub conflict_delta: f32,
}

impl MetadataCandidateMatchingPolicy {
    #[must_use]
    pub fn strict() -> Self {
        Self {
            accept_score: 0.90,
            confirmation_score: 0.60,
            conflict_delta: 0.05,
        }
    }

    #[must_use]
    pub fn evaluate(&self, candidates: Vec<MetadataCandidate>) -> Vec<MetadataCandidateMatch> {
        self.evaluate_for_lookup(&MetadataLookup::default(), candidates)
    }

    #[must_use]
    pub fn evaluate_for_lookup(
        &self,
        lookup: &MetadataLookup,
        mut candidates: Vec<MetadataCandidate>,
    ) -> Vec<MetadataCandidateMatch> {
        candidates.sort_by(|left, right| right.score.total_cmp(&left.score));
        let top_score = candidates.first().map_or(0.0, |candidate| candidate.score);
        let conflicting_top_candidates = candidates
            .iter()
            .filter(|candidate| {
                candidate.score >= self.accept_score
                    && (top_score - candidate.score).abs() <= self.conflict_delta
            })
            .count();

        candidates
            .into_iter()
            .map(|candidate| self.evaluate_candidate(lookup, candidate, conflicting_top_candidates))
            .collect()
    }

    fn evaluate_candidate(
        &self,
        lookup: &MetadataLookup,
        candidate: MetadataCandidate,
        conflicting_top_candidates: usize,
    ) -> MetadataCandidateMatch {
        let mut reasons = evidence_reasons(lookup, &candidate);
        let has_high_confidence_conflict =
            conflicting_top_candidates > 1 && candidate.score >= self.accept_score;

        let decision = if candidate.score >= self.accept_score && !has_high_confidence_conflict {
            reasons.push(MetadataCandidateMatchReason::ScoreAccepted);
            MetadataCandidateMatchDecision::Accepted
        } else if candidate.score >= self.confirmation_score {
            if has_high_confidence_conflict {
                reasons.push(MetadataCandidateMatchReason::NearbyHighConfidenceConflict);
            }
            reasons.push(MetadataCandidateMatchReason::ScoreNeedsConfirmation);
            MetadataCandidateMatchDecision::NeedsConfirmation
        } else {
            reasons.push(MetadataCandidateMatchReason::ScoreRejected);
            MetadataCandidateMatchDecision::Rejected
        };

        let metadata = candidate.metadata();
        let media_kind = candidate.graph.root.kind;
        let message = message_for(decision, candidate.score, &reasons);

        MetadataCandidateMatch {
            provider: candidate.provider,
            provider_key: candidate.provider_key,
            media_kind,
            title: metadata.title,
            release_year: release_year(metadata.release_date.as_deref()),
            score: candidate.score,
            decision,
            reasons,
            message,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MetadataCandidateConflictReview {
    pub item_id: nako_core::MediaItemId,
    pub lookup: MetadataLookup,
    pub decisions: Vec<MetadataCandidateMatch>,
    pub status: MetadataCandidateConflictReviewStatus,
    pub message: String,
}

impl MetadataCandidateConflictReview {
    #[must_use]
    pub fn requires_confirmation(&self) -> bool {
        self.status == MetadataCandidateConflictReviewStatus::NeedsConfirmation
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCandidateConflictReviewStatus {
    Accepted,
    NeedsConfirmation,
    NoCandidates,
    NoAcceptableCandidates,
}

#[must_use]
pub fn build_candidate_conflict_review(
    item: &MediaItem,
    language: Option<String>,
    candidates: Vec<MetadataCandidate>,
) -> MetadataCandidateConflictReview {
    let lookup = MetadataLookup {
        kind: Some(item.kind),
        title: item.metadata.title.clone(),
        year: release_year(item.metadata.release_date.as_deref()),
        language,
        external_ids: item.metadata.external_ids.clone(),
    };
    let decisions = MetadataCandidateMatchingPolicy::strict().evaluate_for_lookup(
        &lookup,
        candidates
            .into_iter()
            .filter(|candidate| candidate.provider != ExternalProvider::Local)
            .collect(),
    );
    let status = if decisions.is_empty() {
        MetadataCandidateConflictReviewStatus::NoCandidates
    } else if decisions
        .iter()
        .any(|decision| decision.decision == MetadataCandidateMatchDecision::NeedsConfirmation)
    {
        MetadataCandidateConflictReviewStatus::NeedsConfirmation
    } else if decisions
        .iter()
        .any(|decision| decision.decision == MetadataCandidateMatchDecision::Accepted)
    {
        MetadataCandidateConflictReviewStatus::Accepted
    } else {
        MetadataCandidateConflictReviewStatus::NoAcceptableCandidates
    };
    let message = match status {
        MetadataCandidateConflictReviewStatus::Accepted => {
            "candidate review has an automatic winner".to_owned()
        }
        MetadataCandidateConflictReviewStatus::NeedsConfirmation => {
            "candidate review requires manual confirmation before canonical metadata changes"
                .to_owned()
        }
        MetadataCandidateConflictReviewStatus::NoCandidates => {
            "candidate review found no provider candidates".to_owned()
        }
        MetadataCandidateConflictReviewStatus::NoAcceptableCandidates => {
            "candidate review found provider candidates but all were rejected".to_owned()
        }
    };

    MetadataCandidateConflictReview {
        item_id: item.id,
        lookup,
        decisions,
        status,
        message,
    }
}

impl Default for MetadataCandidateMatchingPolicy {
    fn default() -> Self {
        Self::strict()
    }
}

fn evidence_reasons(
    lookup: &MetadataLookup,
    candidate: &MetadataCandidate,
) -> Vec<MetadataCandidateMatchReason> {
    let metadata = candidate.metadata();
    let mut reasons = Vec::new();

    match (
        normalized_title(&lookup.title),
        normalized_title(&metadata.title),
    ) {
        (None, _) => reasons.push(MetadataCandidateMatchReason::MissingLookupTitle),
        (_, None) => reasons.push(MetadataCandidateMatchReason::MissingCandidateTitle),
        (Some(lookup_title), Some(candidate_title)) if lookup_title == candidate_title => {
            reasons.push(MetadataCandidateMatchReason::ExactTitle);
        }
        (Some(_), Some(_)) => reasons.push(MetadataCandidateMatchReason::DifferentTitle),
    }

    match (lookup.year, release_year(metadata.release_date.as_deref())) {
        (None, _) => reasons.push(MetadataCandidateMatchReason::MissingLookupYear),
        (_, None) => reasons.push(MetadataCandidateMatchReason::MissingCandidateReleaseYear),
        (Some(lookup_year), Some(candidate_year)) if lookup_year == candidate_year => {
            reasons.push(MetadataCandidateMatchReason::ReleaseYearMatch);
        }
        (Some(_), Some(_)) => reasons.push(MetadataCandidateMatchReason::ReleaseYearMismatch),
    }

    reasons
}

fn message_for(
    decision: MetadataCandidateMatchDecision,
    score: f32,
    reasons: &[MetadataCandidateMatchReason],
) -> String {
    let decision_text = match decision {
        MetadataCandidateMatchDecision::Accepted => "accepted",
        MetadataCandidateMatchDecision::NeedsConfirmation => "needs confirmation",
        MetadataCandidateMatchDecision::Rejected => "rejected",
    };
    let reason_text = reasons
        .iter()
        .map(reason_label)
        .collect::<Vec<_>>()
        .join(", ");

    format!("candidate score {score:.2} {decision_text}: {reason_text}")
}

fn reason_label(reason: &MetadataCandidateMatchReason) -> &'static str {
    match reason {
        MetadataCandidateMatchReason::ScoreAccepted => "score meets automatic threshold",
        MetadataCandidateMatchReason::ScoreNeedsConfirmation => {
            "score is below automatic threshold"
        }
        MetadataCandidateMatchReason::ScoreRejected => "score is below confirmation threshold",
        MetadataCandidateMatchReason::NearbyHighConfidenceConflict => {
            "nearby high-confidence candidate conflict"
        }
        MetadataCandidateMatchReason::ExactTitle => "exact title",
        MetadataCandidateMatchReason::DifferentTitle => "different title",
        MetadataCandidateMatchReason::MissingLookupTitle => "missing lookup title",
        MetadataCandidateMatchReason::MissingCandidateTitle => "missing candidate title",
        MetadataCandidateMatchReason::ReleaseYearMatch => "release year match",
        MetadataCandidateMatchReason::ReleaseYearMismatch => "release year mismatch",
        MetadataCandidateMatchReason::MissingLookupYear => "missing lookup year",
        MetadataCandidateMatchReason::MissingCandidateReleaseYear => {
            "missing candidate release year"
        }
    }
}

fn normalized_title(value: &str) -> Option<String> {
    let normalized = value
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn release_year(value: Option<&str>) -> Option<u16> {
    let value = value?;
    let year = value.get(0..4)?;
    if year.chars().all(|character| character.is_ascii_digit()) {
        year.parse().ok()
    } else {
        None
    }
}
