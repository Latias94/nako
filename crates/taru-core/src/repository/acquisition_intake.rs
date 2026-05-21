use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateListFilter,
    AcquisitionIntakeCandidateRecord, AcquisitionIntakeCandidateState, AcquisitionIntakeSourceKind,
    LibraryId, ManagedImportArtifactId, NewAcquisitionIntakeCandidate, Result,
};

#[async_trait]
pub trait AcquisitionIntakeRepository: Send + Sync {
    async fn upsert_acquisition_intake_candidate(
        &self,
        candidate: NewAcquisitionIntakeCandidate,
    ) -> Result<AcquisitionIntakeCandidateRecord>;

    async fn get_acquisition_intake_candidate(
        &self,
        id: AcquisitionIntakeCandidateId,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;

    async fn find_acquisition_intake_candidate_by_source_key(
        &self,
        target_library_id: LibraryId,
        source_kind: &AcquisitionIntakeSourceKind,
        source_key: &str,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;

    async fn list_acquisition_intake_candidates(
        &self,
        filter: AcquisitionIntakeCandidateListFilter,
        page: PageRequest,
    ) -> Result<Vec<AcquisitionIntakeCandidateRecord>>;

    async fn set_acquisition_intake_candidate_state(
        &self,
        id: AcquisitionIntakeCandidateId,
        state: AcquisitionIntakeCandidateState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;

    async fn link_acquisition_intake_candidate_managed_import_artifact(
        &self,
        id: AcquisitionIntakeCandidateId,
        managed_import_artifact_id: ManagedImportArtifactId,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<AcquisitionIntakeCandidateRecord>>;
}
