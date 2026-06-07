use nako_client_protocol::PageInfo;
use nako_core::{
    DomainEventKind, DomainEventSubject, EventId, IngestionFailureClass, IngestionFailurePhase,
    IngestionFailureRecord, IngestionFailureStatus, Job, JobCancellationRequestRecord, JobId,
    JobKind, JobPriority, JobStatus, LibraryId, MediaSourceId, OutboxEventRecord,
    OutboxEventStatus, ScanSnapshotId, SourceDuplicateEvidenceKind,
    SourceDuplicateReconciliationAction, SourceDuplicateReconciliationApplyResult,
    SourceDuplicateReconciliationCandidate, SourceDuplicateReconciliationPlan,
    SourceDuplicateRelationshipId, SourceDuplicateRelationshipStatus,
    SourceFingerprintEvidenceKind, StorageFailureClass,
};
use serde::{Deserialize, Serialize};

use super::storage::AdminVfsCacheRepairJobSummary;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JobResponse {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub priority: JobPriority,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub has_input: bool,
    pub has_summary: bool,
    pub has_error: bool,
    pub attempt: u32,
    pub max_attempts: u32,
    pub retry_of_job_id: Option<JobId>,
    pub next_attempt_at: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<AdminJobDiagnostics>,
}

impl JobResponse {
    #[must_use]
    pub fn from_job(job: Job) -> Self {
        let diagnostics = AdminJobDiagnostics::from_job(&job);
        Self {
            id: job.id,
            kind: job.kind,
            status: job.status,
            resource_class: job.resource_class,
            priority: job.priority,
            library_id: job.library_id,
            source_id: job.source_id,
            has_input: job.input_json.is_some(),
            has_summary: job.summary_json.is_some(),
            has_error: job.error.is_some(),
            attempt: job.attempt,
            max_attempts: job.max_attempts,
            retry_of_job_id: job.retry_of_job_id,
            next_attempt_at: job.next_attempt_at,
            queued_at: job.queued_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            diagnostics,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobListResponse {
    pub jobs: Vec<AdminJobListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobCancelRequestResponse {
    pub job: AdminJobListItem,
    pub requested: bool,
    pub terminal: bool,
    pub cancel_requested_at: Option<String>,
}

impl AdminJobCancelRequestResponse {
    #[must_use]
    pub fn from_record(record: JobCancellationRequestRecord) -> Self {
        Self {
            job: AdminJobListItem::from_job(record.job),
            requested: record.requested,
            terminal: record.terminal,
            cancel_requested_at: record.cancel_requested_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobListItem {
    pub id: JobId,
    pub kind: JobKind,
    pub status: JobStatus,
    pub resource_class: String,
    pub priority: JobPriority,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub has_input: bool,
    pub has_summary: bool,
    pub has_error: bool,
    pub attempt: u32,
    pub max_attempts: u32,
    pub retry_of_job_id: Option<JobId>,
    pub next_attempt_at: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<AdminJobDiagnostics>,
}

impl AdminJobListItem {
    #[must_use]
    pub fn from_job(job: Job) -> Self {
        let diagnostics = AdminJobDiagnostics::from_job(&job);
        Self {
            id: job.id,
            kind: job.kind,
            status: job.status,
            resource_class: job.resource_class,
            priority: job.priority,
            library_id: job.library_id,
            source_id: job.source_id,
            has_input: job.input_json.is_some(),
            has_summary: job.summary_json.is_some(),
            has_error: job.error.is_some(),
            attempt: job.attempt,
            max_attempts: job.max_attempts,
            retry_of_job_id: job.retry_of_job_id,
            next_attempt_at: job.next_attempt_at,
            queued_at: job.queued_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            diagnostics,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminJobDiagnostics {
    pub vfs_cache_repair: Option<AdminVfsCacheRepairJobDiagnostics>,
}

impl AdminJobDiagnostics {
    fn from_job(job: &Job) -> Option<Self> {
        match job.kind {
            JobKind::VfsCacheRepair => Some(Self {
                vfs_cache_repair: Some(AdminVfsCacheRepairJobDiagnostics::from_job(job)),
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairJobDiagnostics {
    pub status: AdminVfsCacheRepairJobDiagnosticStatus,
    pub summary: Option<AdminVfsCacheRepairJobSummary>,
    pub failure: Option<AdminVfsCacheRepairJobFailureDiagnostic>,
}

impl AdminVfsCacheRepairJobDiagnostics {
    fn from_job(job: &Job) -> Self {
        let summary = job.summary_json.as_deref().and_then(|summary| {
            serde_json::from_str::<AdminVfsCacheRepairJobSummary>(summary).ok()
        });
        let failure = job
            .error
            .as_ref()
            .map(|_error| AdminVfsCacheRepairJobFailureDiagnostic::redacted(job.status));

        Self {
            status: match (&summary, &failure) {
                (Some(_), _) => AdminVfsCacheRepairJobDiagnosticStatus::SummaryAvailable,
                (None, Some(_)) => AdminVfsCacheRepairJobDiagnosticStatus::Failed,
                (None, None) => AdminVfsCacheRepairJobDiagnosticStatus::Pending,
            },
            summary,
            failure,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairJobDiagnosticStatus {
    Pending,
    SummaryAvailable,
    Failed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairJobFailureDiagnostic {
    pub status: JobStatus,
    pub failure_class: StorageFailureClass,
    pub safe_message: String,
    pub retryable: bool,
}

impl AdminVfsCacheRepairJobFailureDiagnostic {
    fn redacted(status: JobStatus) -> Self {
        Self {
            status,
            failure_class: StorageFailureClass::Unknown,
            safe_message: StorageFailureClass::Unknown.safe_message().to_owned(),
            retryable: StorageFailureClass::Unknown.is_retryable(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSourceFingerprintHashMode {
    Full,
    Partial,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminJobPriority {
    Low,
    Normal,
    High,
}

impl From<AdminJobPriority> for JobPriority {
    fn from(priority: AdminJobPriority) -> Self {
        match priority {
            AdminJobPriority::Low => Self::Low,
            AdminJobPriority::Normal => Self::Normal,
            AdminJobPriority::High => Self::High,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceFingerprintHashEnqueueRequest {
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub mode: AdminSourceFingerprintHashMode,
    pub partial_prefix_bytes: Option<u64>,
    pub priority: Option<AdminJobPriority>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceFingerprintHashRetryRequest {
    pub max_attempts: Option<u32>,
    pub next_attempt_at: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSourceDuplicateReconciliationApplyExpectedAction {
    SuggestRelationship,
}

impl From<AdminSourceDuplicateReconciliationApplyExpectedAction>
    for SourceDuplicateReconciliationAction
{
    fn from(action: AdminSourceDuplicateReconciliationApplyExpectedAction) -> Self {
        match action {
            AdminSourceDuplicateReconciliationApplyExpectedAction::SuggestRelationship => {
                Self::SuggestRelationship
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceDuplicateReconciliationApplyRequest {
    pub duplicate_source_id: MediaSourceId,
    pub expected_action: AdminSourceDuplicateReconciliationApplyExpectedAction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceDuplicateReconciliationApplyResponse {
    pub admin_api_version: String,
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub relationship_id: SourceDuplicateRelationshipId,
    pub relationship_status: SourceDuplicateRelationshipStatus,
    pub applied_action: SourceDuplicateReconciliationAction,
    pub created: bool,
}

impl AdminSourceDuplicateReconciliationApplyResponse {
    #[must_use]
    pub fn from_result(result: SourceDuplicateReconciliationApplyResult) -> Self {
        Self {
            admin_api_version: super::ADMIN_API_VERSION.to_owned(),
            library_id: result.library_id,
            source_id: result.source_id,
            duplicate_source_id: result.duplicate_source_id,
            relationship_id: result.relationship_id,
            relationship_status: result.relationship_status,
            applied_action: result.applied_action,
            created: result.created,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceDuplicateReconciliationPlanResponse {
    pub admin_api_version: String,
    pub library_id: LibraryId,
    pub source_id: MediaSourceId,
    pub fingerprint_evidence_kind: SourceFingerprintEvidenceKind,
    pub confidence_milli: u16,
    pub stale: bool,
    pub candidates: Vec<AdminSourceDuplicateReconciliationCandidate>,
    pub page: PageInfo,
}

impl AdminSourceDuplicateReconciliationPlanResponse {
    #[must_use]
    pub fn from_plan(plan: SourceDuplicateReconciliationPlan, page: PageInfo) -> Self {
        Self {
            admin_api_version: super::ADMIN_API_VERSION.to_owned(),
            library_id: plan.library_id,
            source_id: plan.source_id,
            fingerprint_evidence_kind: plan.fingerprint_evidence_kind,
            confidence_milli: plan.confidence_milli,
            stale: plan.stale,
            candidates: plan
                .candidates
                .into_iter()
                .map(AdminSourceDuplicateReconciliationCandidate::from_candidate)
                .collect(),
            page,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSourceDuplicateReconciliationCandidate {
    pub source_id: MediaSourceId,
    pub duplicate_source_id: MediaSourceId,
    pub evidence_kind: SourceDuplicateEvidenceKind,
    pub confidence_milli: Option<u16>,
    pub stale: bool,
    pub relationship_id: Option<SourceDuplicateRelationshipId>,
    pub existing_status: Option<SourceDuplicateRelationshipStatus>,
    pub recommended_action: SourceDuplicateReconciliationAction,
}

impl AdminSourceDuplicateReconciliationCandidate {
    #[must_use]
    pub fn from_candidate(candidate: SourceDuplicateReconciliationCandidate) -> Self {
        Self {
            source_id: candidate.source_id,
            duplicate_source_id: candidate.duplicate_source_id,
            evidence_kind: candidate.evidence_kind,
            confidence_milli: candidate.confidence_milli,
            stale: candidate.stale,
            relationship_id: candidate.relationship_id,
            existing_status: candidate.existing_status,
            recommended_action: candidate.recommended_action,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOutboxEventListResponse {
    pub events: Vec<AdminOutboxEventListItem>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOutboxEventListItem {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: DomainEventSubject,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub status: OutboxEventStatus,
    pub attempts: u32,
    pub has_payload: bool,
    pub has_error: bool,
    pub occurred_at: String,
    pub updated_at: String,
    pub next_attempt_at: Option<String>,
}

impl AdminOutboxEventListItem {
    #[must_use]
    pub fn from_record(event: OutboxEventRecord) -> Self {
        Self {
            id: event.id,
            kind: event.kind,
            subject: event.subject,
            library_id: event.library_id,
            source_id: event.source_id,
            status: event.status,
            attempts: event.attempts,
            has_payload: !event.payload_json.trim().is_empty(),
            has_error: event.last_error.is_some(),
            occurred_at: event.occurred_at,
            updated_at: event.updated_at,
            next_attempt_at: event.next_attempt_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailuresResponse {
    pub library_id: LibraryId,
    pub failures: Vec<IngestionFailureDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDiagnostic {
    #[serde(flatten)]
    pub failure: IngestionFailureDto,
    pub retryable_now: bool,
}

impl IngestionFailureDiagnostic {
    #[must_use]
    pub fn from_record(failure: IngestionFailureRecord) -> Self {
        let retryable_now = failure.status == IngestionFailureStatus::Open && failure.retryable;
        Self {
            failure: failure.into(),
            retryable_now,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IngestionFailureDto {
    pub library_id: LibraryId,
    pub job_id: Option<JobId>,
    pub scan_id: Option<ScanSnapshotId>,
    pub source_id: Option<MediaSourceId>,
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
    pub target_kind: String,
    pub failure_class: IngestionFailureClass,
    pub status: IngestionFailureStatus,
    pub message: String,
    pub retryable: bool,
    pub attempts: u32,
    pub first_failed_at_ms: i64,
    pub last_failed_at_ms: i64,
    pub resolved_at_ms: Option<i64>,
    pub ignored_at_ms: Option<i64>,
}

impl From<IngestionFailureRecord> for IngestionFailureDto {
    fn from(failure: IngestionFailureRecord) -> Self {
        Self {
            library_id: failure.library_id,
            job_id: failure.job_id,
            scan_id: failure.scan_id,
            source_id: failure.source_id,
            phase: failure.phase,
            target_uri: failure.target_uri,
            target_kind: failure.target_kind,
            failure_class: failure.failure_class,
            status: failure.status,
            message: failure.message,
            retryable: failure.retryable,
            attempts: failure.attempts,
            first_failed_at_ms: failure.first_failed_at_ms,
            last_failed_at_ms: failure.last_failed_at_ms,
            resolved_at_ms: failure.resolved_at_ms,
            ignored_at_ms: failure.ignored_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IgnoreIngestionFailureRequest {
    pub phase: IngestionFailurePhase,
    pub target_uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::{
        AdminVfsCacheRepairAction, AdminVfsCacheRepairCacheState, AdminVfsCacheRepairClassification,
    };

    #[test]
    fn ingestion_failure_diagnostic_serializes_explicit_dto_fields() {
        let record = IngestionFailureRecord {
            library_id: LibraryId::new(),
            job_id: Some(JobId::new()),
            scan_id: Some(ScanSnapshotId::new()),
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: "webdav:///Movies/Broken/".to_owned(),
            target_kind: "directory".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            status: IngestionFailureStatus::Open,
            message: "failed to list directory".to_owned(),
            retryable: true,
            attempts: 2,
            first_failed_at_ms: 10,
            last_failed_at_ms: 20,
            resolved_at_ms: None,
            ignored_at_ms: None,
        };

        let diagnostic = IngestionFailureDiagnostic::from_record(record);
        let value = serde_json::to_value(&diagnostic).unwrap();

        assert_eq!(diagnostic.failure.attempts, 2);
        assert!(diagnostic.retryable_now);
        assert_eq!(value["phase"], "scan");
        assert_eq!(value["failure_class"], "storage");
        assert_eq!(value["status"], "open");
        assert!(value.get("failure").is_none());
    }

    #[test]
    fn job_response_redacts_raw_payloads_summaries_and_errors() {
        let retry_of_job_id = JobId::new();
        let job = Job {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            status: JobStatus::Failed,
            resource_class: "disk.scan".to_owned(),
            priority: JobPriority::High,
            library_id: Some(LibraryId::new()),
            source_id: Some(MediaSourceId::new()),
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
            summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
            error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
            attempt: 2,
            max_attempts: 3,
            retry_of_job_id: Some(retry_of_job_id),
            next_attempt_at: Some("2026-05-17T00:10:00Z".to_owned()),
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        };

        let response = JobResponse::from_job(job);
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.has_input);
        assert!(response.has_summary);
        assert!(response.has_error);
        assert_eq!(response.priority, JobPriority::High);
        assert_eq!(response.attempt, 2);
        assert_eq!(response.max_attempts, 3);
        assert_eq!(response.retry_of_job_id, Some(retry_of_job_id));
        assert_eq!(
            response.next_attempt_at.as_deref(),
            Some("2026-05-17T00:10:00Z")
        );
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
        assert!(!body.contains("\"input\":"));
        assert!(!body.contains("\"summary\":"));
        assert!(!body.contains("\"error\":"));
    }

    #[test]
    fn admin_job_list_item_redacts_raw_payloads_and_errors() {
        let retry_of_job_id = JobId::new();
        let job = Job {
            id: JobId::new(),
            kind: JobKind::LibraryScan,
            status: JobStatus::Failed,
            resource_class: "disk.scan".to_owned(),
            priority: JobPriority::Low,
            library_id: Some(LibraryId::new()),
            source_id: Some(MediaSourceId::new()),
            input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
            summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
            error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
            attempt: 3,
            max_attempts: 4,
            retry_of_job_id: Some(retry_of_job_id),
            next_attempt_at: Some("2026-05-17T00:15:00Z".to_owned()),
            queued_at: "2026-05-17T00:00:00Z".to_owned(),
            started_at: Some("2026-05-17T00:00:01Z".to_owned()),
            completed_at: Some("2026-05-17T00:00:02Z".to_owned()),
        };

        let item = AdminJobListItem::from_job(job);
        let body = serde_json::to_string(&item).unwrap();

        assert!(item.has_input);
        assert!(item.has_summary);
        assert!(item.has_error);
        assert_eq!(item.priority, JobPriority::Low);
        assert_eq!(item.attempt, 3);
        assert_eq!(item.max_attempts, 4);
        assert_eq!(item.retry_of_job_id, Some(retry_of_job_id));
        assert_eq!(
            item.next_attempt_at.as_deref(),
            Some("2026-05-17T00:15:00Z")
        );
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("secret"));
    }

    #[test]
    fn admin_job_cancel_request_response_redacts_raw_payloads_and_errors() {
        let record = JobCancellationRequestRecord {
            job: Job {
                id: JobId::new(),
                kind: JobKind::LibraryScan,
                status: JobStatus::Running,
                resource_class: "disk.scan".to_owned(),
                priority: JobPriority::Normal,
                library_id: Some(LibraryId::new()),
                source_id: Some(MediaSourceId::new()),
                input_json: Some(r#"{"secret":"admin-token"}"#.to_owned()),
                summary_json: Some(r#"{"output_path":"C:\\media\\private.nfo"}"#.to_owned()),
                error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
                attempt: 1,
                max_attempts: 1,
                retry_of_job_id: None,
                next_attempt_at: None,
                queued_at: "2026-05-17T00:00:00Z".to_owned(),
                started_at: Some("2026-05-17T00:00:01Z".to_owned()),
                completed_at: None,
            },
            requested: true,
            terminal: false,
            cancel_requested_at: Some("2026-05-17T00:00:03Z".to_owned()),
        };

        let response = AdminJobCancelRequestResponse::from_record(record);
        let body = serde_json::to_string(&response).unwrap();

        assert!(response.requested);
        assert!(!response.terminal);
        assert!(response.job.has_input);
        assert!(response.job.has_summary);
        assert!(response.job.has_error);
        assert_eq!(response.job.priority, JobPriority::Normal);
        assert_eq!(response.job.attempt, 1);
        assert_eq!(response.job.max_attempts, 1);
        assert_eq!(response.job.retry_of_job_id, None);
        assert_eq!(response.job.next_attempt_at, None);
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
        assert!(!body.contains("error\":\"token"));
    }

    #[test]
    fn vfs_cache_repair_job_response_projects_safe_summary_diagnostics() {
        let summary = AdminVfsCacheRepairJobSummary {
            action: AdminVfsCacheRepairAction::RefreshCache,
            source_scheme: "webdav".to_owned(),
            operation: nako_core::VfsCacheOperation::Stat,
            classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
            failure_class: Some(StorageFailureClass::Unavailable),
            failed_at_ms: 1_000,
            failure_count: 2,
            refreshed_cache_state: Some(AdminVfsCacheRepairCacheState::Fresh),
        };
        let job = Job {
            id: JobId::new(),
            kind: JobKind::VfsCacheRepair,
            status: JobStatus::Succeeded,
            resource_class: "storage.vfs.cache_repair".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(LibraryId::new()),
            source_id: None,
            input_json: Some(
                r#"{"uri_digest":"raw digest must-not-leak","source_uri":"local:///Hidden Movie.mkv"}"#
                    .to_owned(),
            ),
            summary_json: Some(serde_json::to_string(&summary).unwrap()),
            error: None,
            attempt: 1,
            max_attempts: 1,
            retry_of_job_id: None,
            next_attempt_at: None,
            queued_at: "2026-06-07T00:00:00Z".to_owned(),
            started_at: Some("2026-06-07T00:00:01Z".to_owned()),
            completed_at: Some("2026-06-07T00:00:02Z".to_owned()),
        };

        let response = JobResponse::from_job(job);
        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["status"],
            "summary_available"
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["summary"]["action"],
            "refresh_cache"
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["summary"]["source_scheme"],
            "webdav"
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["failure"],
            serde_json::Value::Null
        );
        assert!(!body.contains("raw digest"));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("Hidden Movie"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
    }

    #[test]
    fn vfs_cache_repair_job_list_item_projects_only_safe_failure_diagnostics() {
        let job = Job {
            id: JobId::new(),
            kind: JobKind::VfsCacheRepair,
            status: JobStatus::Failed,
            resource_class: "storage.vfs.cache_repair".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(LibraryId::new()),
            source_id: None,
            input_json: Some(r#"{"uri_digest":"raw digest must-not-leak"}"#.to_owned()),
            summary_json: None,
            error: Some(
                "storage error at local:///Hidden Movie.mkv: token=secret safe-test-etag"
                    .to_owned(),
            ),
            attempt: 1,
            max_attempts: 2,
            retry_of_job_id: None,
            next_attempt_at: None,
            queued_at: "2026-06-07T00:00:00Z".to_owned(),
            started_at: Some("2026-06-07T00:00:01Z".to_owned()),
            completed_at: Some("2026-06-07T00:00:02Z".to_owned()),
        };

        let item = AdminJobListItem::from_job(job);
        let value = serde_json::to_value(&item).unwrap();
        let body = value.to_string();

        assert_eq!(value["diagnostics"]["vfs_cache_repair"]["status"], "failed");
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["failure"]["failure_class"],
            "unknown"
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["failure"]["safe_message"],
            "storage failure"
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["failure"]["retryable"],
            false
        );
        assert_eq!(
            value["diagnostics"]["vfs_cache_repair"]["summary"],
            serde_json::Value::Null
        );
        assert!(!body.contains("raw digest"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("Hidden Movie"));
        assert!(!body.contains("token=secret"));
        assert!(!body.contains("safe-test-etag"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
        assert!(!body.contains("error\":\"storage"));
    }

    #[test]
    fn admin_source_fingerprint_hash_enqueue_request_serializes_safe_fields() {
        let request = AdminSourceFingerprintHashEnqueueRequest {
            library_id: LibraryId::new(),
            source_id: MediaSourceId::new(),
            mode: AdminSourceFingerprintHashMode::Partial,
            partial_prefix_bytes: Some(4096),
            priority: Some(AdminJobPriority::High),
        };

        let value = serde_json::to_value(&request).unwrap();
        let body = value.to_string();

        assert_eq!(value["mode"], "partial");
        assert_eq!(value["partial_prefix_bytes"], 4096);
        assert_eq!(value["priority"], "high");
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("locator"));
        assert!(!body.contains("fingerprint"));
        assert!(!body.contains("hash"));
        assert!(!body.contains("path"));
        assert!(!body.contains("etag"));
        assert!(!body.contains("token"));
    }

    #[test]
    fn admin_source_fingerprint_hash_retry_request_serializes_safe_fields() {
        let request = AdminSourceFingerprintHashRetryRequest {
            max_attempts: Some(3),
            next_attempt_at: Some("2026-06-06T00:00:00Z".to_owned()),
        };

        let value = serde_json::to_value(&request).unwrap();
        let body = value.to_string();

        assert_eq!(value["max_attempts"], 3);
        assert_eq!(value["next_attempt_at"], "2026-06-06T00:00:00Z");
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("locator"));
        assert!(!body.contains("fingerprint"));
        assert!(!body.contains("hash\":\""));
        assert!(!body.contains("path"));
        assert!(!body.contains("etag"));
        assert!(!body.contains("token"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
    }

    #[test]
    fn admin_source_duplicate_reconciliation_plan_serializes_safe_fields() {
        let source_id = MediaSourceId::new();
        let duplicate_source_id = MediaSourceId::new();
        let response = AdminSourceDuplicateReconciliationPlanResponse::from_plan(
            SourceDuplicateReconciliationPlan {
                library_id: LibraryId::new(),
                source_id,
                fingerprint_evidence_kind: SourceFingerprintEvidenceKind::ContentHash,
                confidence_milli: 1_000,
                stale: false,
                candidates: vec![SourceDuplicateReconciliationCandidate {
                    source_id,
                    duplicate_source_id,
                    evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
                    confidence_milli: Some(1_000),
                    stale: false,
                    relationship_id: Some(SourceDuplicateRelationshipId::new()),
                    existing_status: Some(SourceDuplicateRelationshipStatus::Suggested),
                    recommended_action: SourceDuplicateReconciliationAction::PreserveSuggested,
                }],
            },
            PageInfo::new(20, 0, 1),
        );

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["fingerprint_evidence_kind"], "content_hash");
        assert_eq!(value["confidence_milli"], 1_000);
        assert_eq!(
            value["candidates"][0]["evidence_kind"],
            "strong_fingerprint"
        );
        assert_eq!(value["candidates"][0]["existing_status"], "suggested");
        assert_eq!(
            value["candidates"][0]["recommended_action"],
            "preserve_suggested"
        );
        assert_eq!(value["page"]["returned"], 1);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("source_locator"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("path"));
        assert!(!body.contains("etag"));
        assert!(!body.contains("token"));
        assert!(!body.contains("sha256"));
        assert!(!body.contains("fingerprint\":\""));
        assert!(!body.contains("evidence_value"));
        assert!(!body.contains("input_json"));
    }

    #[test]
    fn admin_source_duplicate_reconciliation_apply_serializes_safe_fields() {
        let source_id = MediaSourceId::new();
        let duplicate_source_id = MediaSourceId::new();
        let request = AdminSourceDuplicateReconciliationApplyRequest {
            duplicate_source_id,
            expected_action:
                AdminSourceDuplicateReconciliationApplyExpectedAction::SuggestRelationship,
        };
        let response = AdminSourceDuplicateReconciliationApplyResponse::from_result(
            SourceDuplicateReconciliationApplyResult {
                library_id: LibraryId::new(),
                source_id,
                duplicate_source_id,
                relationship_id: SourceDuplicateRelationshipId::new(),
                relationship_status: SourceDuplicateRelationshipStatus::Suggested,
                applied_action: SourceDuplicateReconciliationAction::SuggestRelationship,
                created: true,
            },
        );

        let request_value = serde_json::to_value(&request).unwrap();
        let response_value = serde_json::to_value(&response).unwrap();
        let body = format!("{}{}", request_value, response_value);

        assert_eq!(request_value["expected_action"], "suggest_relationship");
        assert_eq!(response_value["admin_api_version"], "v1");
        assert_eq!(response_value["relationship_status"], "suggested");
        assert_eq!(response_value["applied_action"], "suggest_relationship");
        assert_eq!(response_value["created"], true);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("source_locator"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("path"));
        assert!(!body.contains("etag"));
        assert!(!body.contains("token"));
        assert!(!body.contains("sha256"));
        assert!(!body.contains("fingerprint"));
        assert!(!body.contains("evidence_value"));
        assert!(!body.contains("input_json"));
    }

    #[test]
    fn admin_outbox_event_list_item_redacts_payload_idempotency_key_and_error() {
        let library_id = LibraryId::new();
        let event = OutboxEventRecord {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library_id),
            library_id: Some(library_id),
            source_id: None,
            idempotency_key: "library_scan:secret-key".to_owned(),
            payload_json: r#"{"secret":"admin-token"}"#.to_owned(),
            status: OutboxEventStatus::Failed,
            attempts: 2,
            next_attempt_at: Some("2026-05-17T00:00:10Z".to_owned()),
            occurred_at: "2026-05-17T00:00:00Z".to_owned(),
            updated_at: "2026-05-17T00:00:05Z".to_owned(),
            last_error: Some("token admin-token failed at C:\\media\\private.nfo".to_owned()),
        };

        let item = AdminOutboxEventListItem::from_record(event);
        let body = serde_json::to_string(&item).unwrap();

        assert!(item.has_payload);
        assert!(item.has_error);
        assert_eq!(item.attempts, 2);
        assert!(!body.contains("secret-key"));
        assert!(!body.contains("admin-token"));
        assert!(!body.contains("private.nfo"));
        assert!(!body.contains("payload_json"));
        assert!(!body.contains("idempotency_key"));
        assert!(!body.contains("last_error"));
    }
}
