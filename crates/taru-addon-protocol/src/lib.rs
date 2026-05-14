use serde::{Deserialize, Serialize};
use taru_core::AddonId;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonManifest {
    pub id: AddonId,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub resources: Vec<AddonResource>,
    pub auth: AddonAuth,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonResource {
    Catalog,
    Metadata,
    Image,
    Stream,
    Subtitle,
    Recommendation,
    Automation,
    Webhook,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonAuth {
    #[default]
    None,
    Bearer,
    SharedSecret,
}
