use nako_core::{
    LibraryId, MediaRepository, MediaSource, MediaSourceId, NakoError, PageRequest, Result,
    ScanRepository, SourceDuplicateEvidenceKind, SourceDuplicateReconciliationAction,
    SourceDuplicateReconciliationApplyResult, SourceDuplicateReconciliationCandidate,
    SourceDuplicateReconciliationPlan, SourceDuplicateRelationship, SourceDuplicateRelationshipId,
    SourceDuplicateRelationshipStatus, SourceDuplicateRepository, SourceFingerprintEvidence,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceDuplicateReconciliationApplyRequest {
    pub(crate) library_id: LibraryId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) duplicate_source_id: MediaSourceId,
    pub(crate) expected_action: SourceDuplicateReconciliationAction,
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

        let source_stale = self.source_stale(&source).await?;
        let (fingerprint, source_evidence) =
            source_fingerprint_evidence_for_reconciliation(&source, source_stale)?;
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

    pub(crate) async fn apply_source_duplicate_reconciliation(
        &self,
        request: SourceDuplicateReconciliationApplyRequest,
    ) -> Result<SourceDuplicateReconciliationApplyResult> {
        if request.expected_action != SourceDuplicateReconciliationAction::SuggestRelationship {
            return Err(NakoError::InvalidInput {
                message: "source duplicate reconciliation apply supports only suggest_relationship"
                    .to_owned(),
            });
        }
        if request.source_id == request.duplicate_source_id {
            return Err(NakoError::InvalidInput {
                message: "source duplicate reconciliation candidate must differ from source"
                    .to_owned(),
            });
        }

        let source = self.source_for_plan(request.source_id).await?;
        if source.library_id != request.library_id {
            return Err(NakoError::InvalidInput {
                message:
                    "source duplicate reconciliation source does not belong to requested library"
                        .to_owned(),
            });
        }

        let duplicate = self.source_for_plan(request.duplicate_source_id).await?;
        if duplicate.library_id != request.library_id {
            return Err(NakoError::InvalidInput {
                message:
                    "source duplicate reconciliation candidate does not belong to requested library"
                        .to_owned(),
            });
        }

        let source_stale = self.source_stale(&source).await?;
        let duplicate_stale = self.source_stale(&duplicate).await?;
        let (source_fingerprint, source_evidence) =
            source_fingerprint_evidence_for_reconciliation(&source, source_stale)?;
        let (duplicate_fingerprint, duplicate_evidence) =
            source_fingerprint_evidence_for_reconciliation(&duplicate, duplicate_stale)?;

        if source_fingerprint != duplicate_fingerprint {
            return Err(NakoError::InvalidInput {
                message:
                    "source duplicate reconciliation candidate fingerprint does not match source fingerprint evidence"
                        .to_owned(),
            });
        }

        if source_evidence.stale || duplicate_evidence.stale {
            return Err(NakoError::Conflict {
                message:
                    "source duplicate reconciliation apply expected suggest_relationship but current recommendation is refresh_source_fingerprint"
                        .to_owned(),
            });
        }

        let relationship = self
            .store
            .get_source_duplicate_relationship_by_pair(source.id, duplicate.id)
            .await?;
        let current_action = source_duplicate_reconciliation_action(false, relationship.as_ref());

        match current_action {
            SourceDuplicateReconciliationAction::SuggestRelationship => {
                let relationship = SourceDuplicateRelationship {
                    id: SourceDuplicateRelationshipId::new(),
                    source_id: source.id,
                    duplicate_source_id: duplicate.id,
                    evidence_kind:
                        SourceDuplicateEvidenceKind::from_source_fingerprint_evidence_kind(
                            source_evidence.kind,
                        ),
                    evidence_value: None,
                    status: SourceDuplicateRelationshipStatus::Suggested,
                    confidence_milli: Some(
                        source_evidence
                            .confidence_milli
                            .min(duplicate_evidence.confidence_milli),
                    ),
                }
                .canonicalized();

                self.store
                    .upsert_source_duplicate_relationship(&relationship)
                    .await?;
                let stored = self
                    .store
                    .get_source_duplicate_relationship_by_pair(source.id, duplicate.id)
                    .await?
                    .ok_or_else(|| NakoError::Database {
                        message: "source duplicate relationship missing after reconciliation apply"
                            .to_owned(),
                    })?;

                Ok(SourceDuplicateReconciliationApplyResult {
                    library_id: request.library_id,
                    source_id: source.id,
                    duplicate_source_id: duplicate.id,
                    relationship_id: stored.id,
                    relationship_status: stored.status,
                    applied_action: SourceDuplicateReconciliationAction::SuggestRelationship,
                    created: true,
                })
            }
            SourceDuplicateReconciliationAction::PreserveSuggested => {
                let relationship = relationship.ok_or_else(|| NakoError::Database {
                    message: "suggested source duplicate relationship missing during reconciliation apply"
                        .to_owned(),
                })?;

                Ok(SourceDuplicateReconciliationApplyResult {
                    library_id: request.library_id,
                    source_id: source.id,
                    duplicate_source_id: duplicate.id,
                    relationship_id: relationship.id,
                    relationship_status: relationship.status,
                    applied_action: SourceDuplicateReconciliationAction::PreserveSuggested,
                    created: false,
                })
            }
            SourceDuplicateReconciliationAction::PreserveConfirmed
            | SourceDuplicateReconciliationAction::PreserveRejected
            | SourceDuplicateReconciliationAction::RefreshSourceFingerprint => {
                Err(NakoError::Conflict {
                    message: format!(
                        "source duplicate reconciliation apply expected suggest_relationship but current recommendation is {}",
                        source_duplicate_reconciliation_action_name(current_action)
                    ),
                })
            }
        }
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

fn source_fingerprint_evidence_for_reconciliation(
    source: &MediaSource,
    stale: bool,
) -> Result<(&str, SourceFingerprintEvidence)> {
    let fingerprint = source
        .fingerprint
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| NakoError::InvalidInput {
            message: "source duplicate reconciliation requires source fingerprint evidence"
                .to_owned(),
        })?;
    let evidence = SourceFingerprintEvidence::from_persisted_fingerprint(fingerprint, stale)
        .ok_or_else(|| NakoError::InvalidInput {
            message:
                "source duplicate reconciliation requires redacted source fingerprint evidence"
                    .to_owned(),
        })?;

    Ok((fingerprint, evidence))
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

fn source_duplicate_reconciliation_action_name(
    action: SourceDuplicateReconciliationAction,
) -> &'static str {
    match action {
        SourceDuplicateReconciliationAction::SuggestRelationship => "suggest_relationship",
        SourceDuplicateReconciliationAction::PreserveSuggested => "preserve_suggested",
        SourceDuplicateReconciliationAction::PreserveConfirmed => "preserve_confirmed",
        SourceDuplicateReconciliationAction::PreserveRejected => "preserve_rejected",
        SourceDuplicateReconciliationAction::RefreshSourceFingerprint => {
            "refresh_source_fingerprint"
        }
    }
}
