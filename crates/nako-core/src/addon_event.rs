use serde::{Deserialize, Serialize};

use crate::{
    AddonEventDeliveryAttemptId, AddonId, AddonRoutingPlanStatus, AddonRoutingPlanTarget, EventId,
    NakoError, Result,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonEventDeliveryStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl AddonEventDeliveryStatus {
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
            _ => Err(NakoError::Database {
                message: format!("unknown addon event delivery status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAddonEventDeliveryAttempt {
    pub id: AddonEventDeliveryAttemptId,
    pub addon_id: AddonId,
    pub event_id: EventId,
    pub declaration_id: String,
    pub attempt_number: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClaimAddonEventDeliveryAttempt {
    pub id: AddonEventDeliveryAttemptId,
    pub addon_id: AddonId,
    pub event_id: EventId,
    pub declaration_id: String,
    pub max_attempts: u32,
    pub now: String,
    pub lease_expires_at: String,
    pub forced_replay: bool,
    pub replay_reason_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventDeliveryAttemptRecord {
    pub id: AddonEventDeliveryAttemptId,
    pub addon_id: AddonId,
    pub event_id: EventId,
    pub declaration_id: String,
    pub attempt_number: u32,
    pub status: AddonEventDeliveryStatus,
    pub http_status: Option<u16>,
    pub error: Option<String>,
    pub requested_at: String,
    pub completed_at: Option<String>,
    pub next_retry_at: Option<String>,
    pub lease_expires_at: Option<String>,
    pub forced_replay: bool,
    pub replay_reason_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventSchedulerWorkRecord {
    pub addon_id: AddonId,
    pub event_id: EventId,
    pub declaration_id: String,
    pub event_kind: String,
    pub routing_plan_status: AddonRoutingPlanStatus,
    pub routing_plan_target: AddonRoutingPlanTarget,
    pub routing_plan_safe_reason_code: Option<String>,
    pub attempt_count: u32,
    pub next_attempt_number: u32,
    pub latest_attempt_status: Option<AddonEventDeliveryStatus>,
    pub latest_http_status: Option<u16>,
    pub latest_next_retry_at: Option<String>,
    pub latest_lease_expires_at: Option<String>,
    pub has_succeeded: bool,
    pub has_in_flight: bool,
}
