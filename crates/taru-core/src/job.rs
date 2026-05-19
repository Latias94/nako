use serde::{Deserialize, Serialize};

use crate::{JobId, JobRunToken, JobWorkerId, LibraryId, MediaSourceId, Result, TaruError};

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
    Transcode,
    WebhookDelivery,
    Automation,
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
            Self::Transcode => "transcode",
            Self::WebhookDelivery => "webhook_delivery",
            Self::Automation => "automation",
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
            "transcode" => Ok(Self::Transcode),
            "webhook_delivery" => Ok(Self::WebhookDelivery),
            "automation" => Ok(Self::Automation),
            _ => Err(TaruError::Database {
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
            _ => Err(TaruError::Database {
                message: format!("unknown job status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewJob {
    pub id: JobId,
    pub kind: JobKind,
    pub resource_class: String,
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
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub input_json: Option<String>,
    pub summary_json: Option<String>,
    pub error: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
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
