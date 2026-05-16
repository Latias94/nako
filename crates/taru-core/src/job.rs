use serde::{Deserialize, Serialize};

use crate::{JobId, LibraryId, MediaSourceId, Result, TaruError};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    LibraryScan,
    LibraryProbe,
    MetadataRefresh,
    MetadataMaintenance,
    NfoImport,
    NfoExport,
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
}

impl JobStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
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
