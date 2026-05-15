use serde::{Deserialize, Serialize};

use crate::{EventId, Result, TaruError, WebhookDeliveryAttemptId, WebhookEndpointId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEndpointStatus {
    Enabled,
    Disabled,
}

impl WebhookEndpointStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(TaruError::Database {
                message: format!("unknown webhook endpoint status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewWebhookEndpoint {
    pub id: WebhookEndpointId,
    pub name: String,
    pub url: String,
    pub secret_env: Option<String>,
    pub subscribed_event_kinds: Vec<String>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub status: WebhookEndpointStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookEndpointRecord {
    pub id: WebhookEndpointId,
    pub name: String,
    pub url: String,
    pub secret_env: Option<String>,
    pub subscribed_event_kinds: Vec<String>,
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub status: WebhookEndpointStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDeliveryStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl WebhookDeliveryStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(TaruError::Database {
                message: format!("unknown webhook delivery status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewWebhookDeliveryAttempt {
    pub id: WebhookDeliveryAttemptId,
    pub endpoint_id: WebhookEndpointId,
    pub event_id: EventId,
    pub attempt_number: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDeliveryAttemptRecord {
    pub id: WebhookDeliveryAttemptId,
    pub endpoint_id: WebhookEndpointId,
    pub event_id: EventId,
    pub attempt_number: u32,
    pub status: WebhookDeliveryStatus,
    pub http_status: Option<u16>,
    pub error: Option<String>,
    pub requested_at: String,
    pub completed_at: Option<String>,
    pub next_retry_at: Option<String>,
}
