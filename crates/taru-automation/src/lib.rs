use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use taru_core::{JobId, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProviderDescriptor {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<AutomationCapability>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationCapability {
    Recommendation,
    MetadataCleanup,
    Summary,
    Classification,
    NotificationText,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AutomationRequest {
    pub job_id: JobId,
    pub provider_id: String,
    pub capability: AutomationCapability,
    pub input: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AutomationOutcome {
    pub job_id: JobId,
    pub output: Value,
    pub accepted_into_canonical_metadata: bool,
}

#[async_trait]
pub trait AutomationProvider: Send + Sync {
    fn descriptor(&self) -> AutomationProviderDescriptor;

    async fn run(&self, request: AutomationRequest) -> Result<AutomationOutcome>;
}
