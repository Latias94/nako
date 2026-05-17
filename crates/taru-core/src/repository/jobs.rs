use async_trait::async_trait;

use super::PageRequest;
use crate::{
    DomainEventKind, EventId, Job, JobId, JobKind, JobStatus, LibraryId, MediaSourceId, NewJob,
    NewOutboxEvent, OutboxEventRecord, Result,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JobListFilter {
    pub status: Option<JobStatus>,
    pub kind: Option<JobKind>,
    pub resource_class: Option<String>,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job>;

    async fn start_job(&self, id: JobId) -> Result<Job>;

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job>;

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job>;

    async fn fail_unfinished_jobs(&self, error: String) -> Result<u64>;

    async fn get_job(&self, id: JobId) -> Result<Option<Job>>;

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>>;
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

    async fn list_outbox_events(&self, page: PageRequest) -> Result<Vec<OutboxEventRecord>>;
}
