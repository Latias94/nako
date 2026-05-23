use serde::{Deserialize, Serialize};

use crate::{
    AddonId, AddonManifestFingerprint, Job, JobId, JobLeaseRecord, JobRunToken, JobWorkerId,
    LibraryId, MediaSourceId,
};

pub const ADDON_TASK_RUN_INPUT_SCHEMA: &str = "nako.addon.task_run.input.v1";
pub const ADDON_TASK_RUN_PROGRESS_SCHEMA: &str = "nako.addon.task_run.progress.v1";
pub const ADDON_TASK_RUN_RESULT_SCHEMA: &str = "nako.addon.task_run.result.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAddonTaskRun {
    pub job_id: JobId,
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub manifest_version: String,
    pub manifest_fingerprint: AddonManifestFingerprint,
    pub declaration_id: String,
    pub declaration_name: String,
    pub declaration_path: String,
    pub idempotency_key: String,
    pub attempt: u32,
    pub max_attempts: Option<u32>,
    pub retry_of_job_id: Option<JobId>,
    pub input_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunRecord {
    pub job: Job,
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub manifest_version: String,
    pub manifest_fingerprint: AddonManifestFingerprint,
    pub declaration_id: String,
    pub declaration_name: String,
    pub declaration_path: String,
    pub idempotency_key: String,
    pub attempt: u32,
    pub max_attempts: Option<u32>,
    pub retry_of_job_id: Option<JobId>,
    #[serde(skip_serializing)]
    pub input_json: String,
    #[serde(skip_serializing)]
    pub progress_json: Option<String>,
    #[serde(skip_serializing)]
    pub result_json: Option<String>,
    pub safe_error_code: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreatedAddonTaskRun {
    pub run: AddonTaskRunRecord,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LeasedAddonTaskRun {
    pub run: AddonTaskRunRecord,
    pub lease: JobLeaseRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunClaimRequest {
    pub addon_id: AddonId,
    pub worker_id: JobWorkerId,
    pub lease_duration_ms: u64,
    pub declaration_id: Option<String>,
    pub job_id: Option<JobId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunLeaseGuard {
    pub job_id: JobId,
    pub run_token: JobRunToken,
}

impl From<crate::JobLeaseGuard> for AddonTaskRunLeaseGuard {
    fn from(value: crate::JobLeaseGuard) -> Self {
        Self {
            job_id: value.job_id,
            run_token: value.run_token,
        }
    }
}

impl From<AddonTaskRunLeaseGuard> for crate::JobLeaseGuard {
    fn from(value: AddonTaskRunLeaseGuard) -> Self {
        Self {
            job_id: value.job_id,
            run_token: value.run_token,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReportAddonTaskRunProgress {
    pub guard: AddonTaskRunLeaseGuard,
    pub lease_duration_ms: u64,
    pub progress_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompleteAddonTaskRun {
    pub guard: AddonTaskRunLeaseGuard,
    pub result_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FailAddonTaskRun {
    pub guard: AddonTaskRunLeaseGuard,
    pub safe_error_code: String,
    pub result_json: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CancelAddonTaskRun {
    pub guard: AddonTaskRunLeaseGuard,
    pub result_json: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AddonTaskRunListFilter {
    pub addon_id: Option<AddonId>,
    pub declaration_id: Option<String>,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
}
