use nako_core::{
    LibraryId, MediaRepository, MediaSource, MediaSourceId, NakoError, PageRequest, Result,
    ScanRepository, SourceDuplicateEvidenceKind, SourceDuplicateReconciliationAction,
    SourceDuplicateReconciliationCandidate, SourceDuplicateReconciliationPlan,
    SourceDuplicateRelationship, SourceDuplicateRelationshipStatus, SourceDuplicateRepository,
    SourceFingerprintEvidence,
};
use nako_db::NakoDatabase;

#[derive(Clone, Debug)]
pub(crate) struct SourceDuplicateReconciliationAppService {
    store: NakoDatabase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceDuplicateReconciliationPlanRequest {
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) page: PageRequest,
}

impl SourceDuplicateReconciliationAppService {
    pub(super) fn new(store: NakoDatabase) -> Self {
        Self { store }
    }

    pub(crate) async fn plan_source_duplicate_reconciliation(
        &self,
        request: SourceDuplicateReconciliationPlanRequest,
    ) -> Result<SourceDuplicateReconciliationPlan> {
        let source = self.source_for_plan(request.source_id).await?;
        if source.library_id != request.library_id {
            return Err(NakoError::InvalidInput {
                message:
                    "source duplicate reconciliation source does not belong to requested library"
                        .to_owned(),
            });
        }

        let fingerprint = source
            .fingerprint
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| NakoError::InvalidInput {
                message: "source duplicate reconciliation requires source fingerprint evidence"
                    .to_owned(),
            })?;
        let source_stale = self.source_stale(&source).await?;
        let source_evidence =
            SourceFingerprintEvidence::from_persisted_fingerprint(fingerprint, source_stale)
                .ok_or_else(|| {
                    NakoError::InvalidInput {
            message:
                "source duplicate reconciliation requires redacted source fingerprint evidence"
                    .to_owned(),
        }
                })?;
        let evidence_kind = SourceDuplicateEvidenceKind::from_source_fingerprint_evidence_kind(
            source_evidence.kind,
        );
        let matches = self
            .store
            .list_media_sources_by_fingerprint(
                request.library_id,
                fingerprint,
                Some(source.id),
                request.page,
            )
            .await?;
        let mut candidates = Vec::new();

        for matched in matches {
            let candidate_stale = source_stale || matched.stale;
            let candidate_evidence =
                SourceFingerprintEvidence::from_persisted_fingerprint(fingerprint, candidate_stale)
                    .ok_or_else(|| {
                        NakoError::InvalidInput {
                message:
                    "source duplicate reconciliation requires redacted source fingerprint evidence"
                        .to_owned(),
            }
                    })?;
            let relationship = self
                .store
                .get_source_duplicate_relationship_by_pair(source.id, matched.source.id)
                .await?;
            let recommended_action =
                source_duplicate_reconciliation_action(candidate_stale, relationship.as_ref());

            candidates.push(SourceDuplicateReconciliationCandidate {
                source_id: source.id,
                duplicate_source_id: matched.source.id,
                evidence_kind: evidence_kind.clone(),
                confidence_milli: Some(candidate_evidence.confidence_milli),
                stale: candidate_stale,
                relationship_id: relationship.as_ref().map(|relationship| relationship.id),
                existing_status: relationship
                    .as_ref()
                    .map(|relationship| relationship.status),
                recommended_action,
            });
        }

        Ok(SourceDuplicateReconciliationPlan {
            library_id: request.library_id,
            source_id: source.id,
            fingerprint_evidence_kind: source_evidence.kind,
            confidence_milli: source_evidence.confidence_milli,
            stale: source_stale,
            candidates,
        })
    }

    async fn source_for_plan(&self, source_id: MediaSourceId) -> Result<MediaSource> {
        self.store
            .get_media_source(source_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "media_source",
                id: source_id.to_string(),
            })
    }

    async fn source_stale(&self, source: &MediaSource) -> Result<bool> {
        Ok(self
            .store
            .get_source_state(source.library_id, &source.locator)
            .await?
            .is_some_and(|state| state.tombstoned))
    }
}

fn source_duplicate_reconciliation_action(
    stale: bool,
    relationship: Option<&SourceDuplicateRelationship>,
) -> SourceDuplicateReconciliationAction {
    match relationship.map(|relationship| relationship.status) {
        Some(SourceDuplicateRelationshipStatus::Suggested) => {
            SourceDuplicateReconciliationAction::PreserveSuggested
        }
        Some(SourceDuplicateRelationshipStatus::Confirmed) => {
            SourceDuplicateReconciliationAction::PreserveConfirmed
        }
        Some(SourceDuplicateRelationshipStatus::Rejected) => {
            SourceDuplicateReconciliationAction::PreserveRejected
        }
        None if stale => SourceDuplicateReconciliationAction::RefreshSourceFingerprint,
        None => SourceDuplicateReconciliationAction::SuggestRelationship,
    }
}
