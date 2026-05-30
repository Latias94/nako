use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AutomationArtifactId, AutomationArtifactRecord, AutomationArtifactStatus,
    AutomationProviderConfigRecord, AutomationProviderId,
    GeneratedArtifactMetadataApplyOutcomeCommit, GeneratedArtifactMetadataApplyOutcomeRecord,
    GeneratedArtifactProposal, JobId, MediaItemId, NewAutomationArtifact,
    NewAutomationProviderConfig, Result,
};

#[async_trait]
pub trait AutomationRepository: Send + Sync {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord>;

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>>;

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>>;

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord>;

    async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>>;

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord>;

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>>;

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>>;

    async fn list_generated_artifact_proposals(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactProposal>>;

    async fn find_generated_artifact_metadata_apply_outcome(
        &self,
        artifact_id: AutomationArtifactId,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataApplyOutcomeRecord>>;

    async fn commit_generated_artifact_metadata_apply_outcome(
        &self,
        commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord>;
}
