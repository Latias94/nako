use async_trait::async_trait;

use taru_core::Result;
pub use taru_core::{
    DomainEventKind, DomainEventSubject, NewOutboxEvent as DomainEvent, OutboxEventRecord,
    OutboxEventStatus,
};

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
}
