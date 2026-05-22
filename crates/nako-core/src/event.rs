use serde::{Deserialize, Serialize};

use crate::{
    EventId, JobId, LibraryId, MediaItemId, MediaSourceId, NakoError, Result, TranscodeSessionId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainEventKind {
    LibraryScanned,
    ItemMetadataRefreshed,
    MetadataMaintenanceCompleted,
    NfoImported,
    NfoExported,
    PlaybackSessionFinished,
}

impl DomainEventKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LibraryScanned => "library.scanned",
            Self::ItemMetadataRefreshed => "item.metadata_refreshed",
            Self::MetadataMaintenanceCompleted => "metadata.maintenance_completed",
            Self::NfoImported => "nfo.imported",
            Self::NfoExported => "nfo.exported",
            Self::PlaybackSessionFinished => "playback.session_finished",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "library.scanned" => Ok(Self::LibraryScanned),
            "item.metadata_refreshed" => Ok(Self::ItemMetadataRefreshed),
            "metadata.maintenance_completed" => Ok(Self::MetadataMaintenanceCompleted),
            "nfo.imported" => Ok(Self::NfoImported),
            "nfo.exported" => Ok(Self::NfoExported),
            "playback.session_finished" => Ok(Self::PlaybackSessionFinished),
            _ => Err(NakoError::Database {
                message: format!("unknown event kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
pub enum DomainEventSubject {
    Library(LibraryId),
    Item(MediaItemId),
    Source(MediaSourceId),
    Job(JobId),
    PlaybackSession(TranscodeSessionId),
}

impl DomainEventSubject {
    #[must_use]
    pub const fn kind(self) -> &'static str {
        match self {
            Self::Library(_) => "library",
            Self::Item(_) => "item",
            Self::Source(_) => "source",
            Self::Job(_) => "job",
            Self::PlaybackSession(_) => "playback_session",
        }
    }

    #[must_use]
    pub fn id(self) -> String {
        match self {
            Self::Library(id) => id.to_string(),
            Self::Item(id) => id.to_string(),
            Self::Source(id) => id.to_string(),
            Self::Job(id) => id.to_string(),
            Self::PlaybackSession(id) => id.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxEventStatus {
    Pending,
    Dispatching,
    Delivered,
    Failed,
}

impl OutboxEventStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatching => "dispatching",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "dispatching" => Ok(Self::Dispatching),
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            _ => Err(NakoError::Database {
                message: format!("unknown outbox event status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewOutboxEvent {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: DomainEventSubject,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub idempotency_key: String,
    pub payload_json: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutboxEventRecord {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: DomainEventSubject,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub idempotency_key: String,
    pub payload_json: String,
    pub status: OutboxEventStatus,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub occurred_at: String,
    pub updated_at: String,
    pub next_attempt_at: Option<String>,
}
