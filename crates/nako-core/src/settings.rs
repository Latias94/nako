use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataRawCacheSettings {
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSettingsSource {
    Configured,
    Admin,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSettingsEffect {
    Active,
    RequiresRestart,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataRawCacheSettingsRecord {
    pub settings: AdminMetadataRawCacheSettings,
    pub source: AdminSettingsSource,
    pub effect: AdminSettingsEffect,
    pub updated_at_ms: i64,
}
