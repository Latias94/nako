use async_trait::async_trait;

use super::PageRequest;
use crate::{
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRecord, IngestionFailureStatus,
    LibraryId, NewIngestionFailure, Result,
};

#[async_trait]
pub trait IngestionFailureRepository: Send + Sync {
    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord>;

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>>;

    async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        ignored_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>>;

    async fn list_ingestion_failures(
        &self,
        filter: IngestionFailureFilter,
        page: PageRequest,
    ) -> Result<Vec<IngestionFailureRecord>>;

    async fn count_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: IngestionFailureStatus,
    ) -> Result<u64>;
}
