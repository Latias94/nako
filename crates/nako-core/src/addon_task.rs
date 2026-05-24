use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    AddonId, AddonManifestFingerprint, Job, JobId, JobLeaseRecord, JobRunToken, JobWorkerId,
    LibraryId, MediaSourceId, NakoError, Result,
};

pub const ADDON_TASK_RUN_INPUT_SCHEMA: &str = "nako.addon.task_run.input.v1";
pub const ADDON_TASK_RUN_PROGRESS_SCHEMA: &str = "nako.addon.task_run.progress.v1";
pub const ADDON_TASK_RUN_RESULT_SCHEMA: &str = "nako.addon.task_run.result.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunRequestFingerprint {
    value: String,
}

impl AddonTaskRunRequestFingerprint {
    #[must_use]
    pub fn new(
        manifest_id: &str,
        manifest_version: &str,
        manifest_fingerprint: &AddonManifestFingerprint,
        declaration_id: &str,
        declaration_path: &str,
        input_json: &str,
    ) -> Self {
        let mut hasher = Sha256::new();
        update_fingerprint_part(&mut hasher, "addon-task-run-v1");
        update_fingerprint_part(&mut hasher, manifest_id);
        update_fingerprint_part(&mut hasher, manifest_version);
        update_fingerprint_part(&mut hasher, manifest_fingerprint.as_str());
        update_fingerprint_part(&mut hasher, declaration_id);
        update_fingerprint_part(&mut hasher, declaration_path);
        update_fingerprint_part(&mut hasher, input_json);
        let digest = hasher.finalize();

        Self {
            value: format!("sha256:{}", lowercase_hex(&digest)),
        }
    }

    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let valid = value.strip_prefix("sha256:").is_some_and(|digest| {
            digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit())
        });
        if !valid {
            return Err(NakoError::Database {
                message: format!("invalid addon task run request fingerprint: {value}"),
            });
        }

        Ok(Self {
            value: value.to_ascii_lowercase(),
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for AddonTaskRunRequestFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

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
    pub request_fingerprint: AddonTaskRunRequestFingerprint,
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
    pub request_fingerprint: AddonTaskRunRequestFingerprint,
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

fn update_fingerprint_part(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(value.as_bytes());
    hasher.update(b";");
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
