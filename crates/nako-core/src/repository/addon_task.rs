use async_trait::async_trait;

use super::PageRequest;
use crate::{
    AddonId, AddonTaskRunClaimRequest, AddonTaskRunListFilter, AddonTaskRunRecord,
    CancelAddonTaskRun, CompleteAddonTaskRun, CreatedAddonTaskRun, FailAddonTaskRun, JobId,
    LeasedAddonTaskRun, NewAddonTaskRun, NewJob, ReportAddonTaskRunProgress, Result,
};

#[async_trait]
pub trait AddonTaskRunRepository: Send + Sync {
    async fn create_addon_task_run(
        &self,
        job: NewJob,
        run: NewAddonTaskRun,
    ) -> Result<CreatedAddonTaskRun>;

    async fn get_addon_task_run(&self, job_id: JobId) -> Result<Option<AddonTaskRunRecord>>;

    async fn list_addon_task_runs(
        &self,
        filter: AddonTaskRunListFilter,
        page: PageRequest,
    ) -> Result<Vec<AddonTaskRunRecord>>;

    async fn claim_next_addon_task_run(
        &self,
        request: AddonTaskRunClaimRequest,
    ) -> Result<Option<LeasedAddonTaskRun>>;

    async fn report_addon_task_run_progress(
        &self,
        progress: ReportAddonTaskRunProgress,
    ) -> Result<LeasedAddonTaskRun>;

    async fn complete_addon_task_run(
        &self,
        completion: CompleteAddonTaskRun,
    ) -> Result<AddonTaskRunRecord>;

    async fn fail_addon_task_run(&self, failure: FailAddonTaskRun) -> Result<AddonTaskRunRecord>;

    async fn cancel_addon_task_run(
        &self,
        cancellation: CancelAddonTaskRun,
    ) -> Result<AddonTaskRunRecord>;

    async fn find_addon_task_run_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonTaskRunRecord>>;
}
