use serde::{Deserialize, Serialize};

use crate::{JobId, JobRunToken, JobWorkerId, LibraryId, MediaSourceId, NakoError, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    LibraryScan,
    LibraryProbe,
    MetadataRefresh,
    MetadataMaintenance,
    NfoImport,
    NfoExport,
    ManagedArtworkIngest,
    GeneratedArtifactMetadataBulkApply,
    MetadataCandidateReviewBatchApply,
    Transcode,
    WebhookDelivery,
    Automation,
    AddonTask,
}

impl JobKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LibraryScan => "library_scan",
            Self::LibraryProbe => "library_probe",
            Self::MetadataRefresh => "metadata_refresh",
            Self::MetadataMaintenance => "metadata_maintenance",
            Self::NfoImport => "nfo_import",
            Self::NfoExport => "nfo_export",
            Self::ManagedArtworkIngest => "managed_artwork_ingest",
            Self::GeneratedArtifactMetadataBulkApply => "generated_artifact_metadata_bulk_apply",
            Self::MetadataCandidateReviewBatchApply => "metadata_candidate_review_batch_apply",
            Self::Transcode => "transcode",
            Self::WebhookDelivery => "webhook_delivery",
            Self::Automation => "automation",
            Self::AddonTask => "addon_task",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "library_scan" => Ok(Self::LibraryScan),
            "library_probe" => Ok(Self::LibraryProbe),
            "metadata_refresh" => Ok(Self::MetadataRefresh),
            "metadata_maintenance" => Ok(Self::MetadataMaintenance),
            "nfo_import" => Ok(Self::NfoImport),
            "nfo_export" => Ok(Self::NfoExport),
            "managed_artwork_ingest" => Ok(Self::ManagedArtworkIngest),
            "generated_artifact_metadata_bulk_apply" => {
                Ok(Self::GeneratedArtifactMetadataBulkApply)
            }
            "metadata_candidate_review_batch_apply" => Ok(Self::MetadataCandidateReviewBatchApply),
            "transcode" => Ok(Self::Transcode),
            "webhook_delivery" => Ok(Self::WebhookDelivery),
            "automation" => Ok(Self::Automation),
            "addon_task" => Ok(Self::AddonTask),
            _ => Err(NakoError::Database {
                message: format!("unknown job kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(NakoError::Database {
                message: format!("unknown job status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    Low,
    #[default]
    Normal,
    High,
}

impl JobPriority {
    pub const LOW_SCORE: i64 = 0;
    pub const NORMAL_SCORE: i64 = 50;
    pub const HIGH_SCORE: i64 = 100;

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
        }
    }

    #[must_use]
    pub const fn score(self) -> i64 {
        match self {
            Self::Low => Self::LOW_SCORE,
            Self::Normal => Self::NORMAL_SCORE,
            Self::High => Self::HIGH_SCORE,
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "low" => Ok(Self::Low),
            "normal" => Ok(Self::Normal),
            "high" => Ok(Self::High),
            _ => Err(NakoError::Database {
                message: format!("unknown job priority stored in database: {value}"),
            }),
        }
    }

    pub fn from_score(value: i64) -> Result<Self> {
        match value {
            Self::LOW_SCORE => Ok(Self::Low),
            Self::NORMAL_SCORE => Ok(Self::Normal),
            Self::HIGH_SCORE => Ok(Self::High),
            _ => Err(NakoError::Database {
                message: format!("unknown job priority score stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewJob {
    pub id: JobId,
    pub kind: JobKind,
    pub resource_class: String,
    pub priority: JobPriority,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub input_json: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub priority: JobPriority,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub input_json: Option<String>,
    pub summary_json: Option<String>,
    pub error: Option<String>,
    pub attempt: u32,
    pub max_attempts: u32,
    pub retry_of_job_id: Option<JobId>,
    pub next_attempt_at: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnqueueJobRetry {
    pub source_job_id: JobId,
    pub retry_job_id: JobId,
    pub max_attempts: u32,
    pub next_attempt_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobQueuePressureSummary {
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub count: u64,
    pub claimable_count: u64,
    pub delayed_retry_count: u64,
    pub oldest_queued_at: Option<String>,
    pub next_attempt_at: Option<String>,
}

impl EnqueueJobRetry {
    pub fn next_attempt_for(&self, source: &Job) -> Result<u32> {
        self.validate_source(source)?;
        source
            .attempt
            .checked_add(1)
            .ok_or_else(|| NakoError::InvalidInput {
                message: "job retry attempt would overflow u32".to_owned(),
            })
    }

    pub fn validate_source(&self, source: &Job) -> Result<()> {
        if self.source_job_id != source.id {
            return Err(NakoError::InvalidInput {
                message: "retry source job does not match loaded job".to_owned(),
            });
        }
        if self.retry_job_id == source.id {
            return Err(NakoError::InvalidInput {
                message: "retry job id must differ from source job id".to_owned(),
            });
        }
        if self.max_attempts == 0 {
            return Err(NakoError::InvalidInput {
                message: "retry max_attempts must be greater than zero".to_owned(),
            });
        }
        if source.status != JobStatus::Failed {
            return Err(NakoError::Conflict {
                message: "only failed jobs can be retried".to_owned(),
            });
        }
        if source.attempt >= self.max_attempts {
            return Err(NakoError::Conflict {
                message: "job retry attempts are exhausted".to_owned(),
            });
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobLeaseClaimFilter {
    pub job_id: Option<JobId>,
    pub kind: Option<JobKind>,
    pub resource_class: Option<String>,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobLeaseClaimRequest {
    pub worker_id: JobWorkerId,
    pub lease_duration_ms: u64,
    pub filter: JobLeaseClaimFilter,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobLeaseRecord {
    pub job_id: JobId,
    pub worker_id: JobWorkerId,
    pub run_token: JobRunToken,
    pub heartbeat_at: String,
    pub lease_expires_at: String,
    pub cancel_requested_at: Option<String>,
    pub cancel_reason: Option<String>,
}

impl JobLeaseRecord {
    #[must_use]
    pub const fn guard(&self) -> JobLeaseGuard {
        JobLeaseGuard {
            job_id: self.job_id,
            run_token: self.run_token,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LeasedJob {
    pub job: Job,
    pub lease: JobLeaseRecord,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobLeaseGuard {
    pub job_id: JobId,
    pub run_token: JobRunToken,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobLeaseHeartbeat {
    pub guard: JobLeaseGuard,
    pub lease_duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompleteLeasedJob {
    pub guard: JobLeaseGuard,
    pub summary_json: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FailLeasedJob {
    pub guard: JobLeaseGuard,
    pub error: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RequestJobCancellation {
    pub job_id: JobId,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobCancellationRequestRecord {
    pub job: Job,
    pub requested: bool,
    pub terminal: bool,
    pub cancel_requested_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CancelLeasedJob {
    pub guard: JobLeaseGuard,
    pub summary_json: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecoverExpiredJobLeases {
    pub filter: JobLeaseClaimFilter,
    pub expired_before: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_status_round_trips_cancelled() {
        assert_eq!(JobStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(JobStatus::parse("cancelled").unwrap(), JobStatus::Cancelled);
    }

    #[test]
    fn job_priority_round_trips_scores() {
        assert_eq!(JobPriority::Low.as_str(), "low");
        assert_eq!(JobPriority::Normal.as_str(), "normal");
        assert_eq!(JobPriority::High.as_str(), "high");
        assert_eq!(JobPriority::parse("normal").unwrap(), JobPriority::Normal);
        assert_eq!(
            JobPriority::from_score(JobPriority::High.score()).unwrap(),
            JobPriority::High
        );
    }

    #[test]
    fn addon_task_kind_round_trips() {
        assert_eq!(JobKind::AddonTask.as_str(), "addon_task");
        assert_eq!(JobKind::parse("addon_task").unwrap(), JobKind::AddonTask);
    }

    #[test]
    fn generated_artifact_metadata_bulk_apply_kind_round_trips() {
        assert_eq!(
            JobKind::GeneratedArtifactMetadataBulkApply.as_str(),
            "generated_artifact_metadata_bulk_apply"
        );
        assert_eq!(
            JobKind::parse("generated_artifact_metadata_bulk_apply").unwrap(),
            JobKind::GeneratedArtifactMetadataBulkApply
        );
    }

    #[test]
    fn metadata_candidate_review_batch_apply_kind_round_trips() {
        assert_eq!(
            JobKind::MetadataCandidateReviewBatchApply.as_str(),
            "metadata_candidate_review_batch_apply"
        );
        assert_eq!(
            JobKind::parse("metadata_candidate_review_batch_apply").unwrap(),
            JobKind::MetadataCandidateReviewBatchApply
        );
    }

    #[test]
    fn job_lease_record_exposes_fencing_guard() {
        let job_id = JobId::new();
        let run_token = JobRunToken::new();
        let lease = JobLeaseRecord {
            job_id,
            worker_id: JobWorkerId::new(),
            run_token,
            heartbeat_at: "2026-05-19T00:00:00.000Z".to_owned(),
            lease_expires_at: "2026-05-19T00:01:00.000Z".to_owned(),
            cancel_requested_at: None,
            cancel_reason: None,
        };

        assert_eq!(lease.guard().job_id, job_id);
        assert_eq!(lease.guard().run_token, run_token);
    }
}
