use async_trait::async_trait;

use super::PageRequest;
use crate::{
    CancelLeasedJob, CompleteLeasedJob, DomainEventKind, EnqueueJobRetry, EventId, FailLeasedJob,
    Job, JobCancellationRequestRecord, JobId, JobKind, JobLeaseClaimRequest, JobLeaseHeartbeat,
    JobQueuePressureSummary, JobStatus, LeasedJob, LibraryId, MediaSourceId, NewJob,
    NewOutboxEvent, OutboxEventRecord, OutboxEventStatus, RecoverExpiredJobLeases,
    RequestJobCancellation, Result,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JobListFilter {
    pub status: Option<JobStatus>,
    pub kind: Option<JobKind>,
    pub resource_class: Option<String>,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutboxEventListFilter {
    pub kind: Option<DomainEventKind>,
    pub status: Option<OutboxEventStatus>,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn enqueue_job_retry(&self, retry: EnqueueJobRetry) -> Result<Job>;

    async fn start_job(&self, id: JobId) -> Result<Job>;

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job>;

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job>;

    async fn fail_unfinished_jobs(&self, error: String) -> Result<u64>;

    async fn get_job(&self, id: JobId) -> Result<Option<Job>>;

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>>;

    async fn summarize_job_queue_pressure(&self) -> Result<Vec<JobQueuePressureSummary>>;
}

#[async_trait]
pub trait JobLeaseRepository: Send + Sync {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>>;

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob>;

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job>;

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job>;

    async fn request_job_cancellation(
        &self,
        request: RequestJobCancellation,
    ) -> Result<JobCancellationRequestRecord>;

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job>;

    async fn recover_expired_job_leases(&self, recovery: RecoverExpiredJobLeases) -> Result<u64>;
}

#[async_trait]
pub trait EventOutboxRepository: Send + Sync {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord>;

    async fn get_outbox_event(&self, id: EventId) -> Result<Option<OutboxEventRecord>>;

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>>;

    async fn list_outbox_events(
        &self,
        filter: OutboxEventListFilter,
        page: PageRequest,
    ) -> Result<Vec<OutboxEventRecord>>;
}
