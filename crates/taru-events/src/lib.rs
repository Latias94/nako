use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taru_core::{EventId, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainEventKind {
    LibraryScanned,
    ItemIndexed,
    MetadataUpdated,
    PlaybackStarted,
    AutomationCompleted,
    AddonCalled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DomainEvent {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: String,
    pub occurred_at: String,
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<()>;
}
