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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSettingsDocumentKey {
    PlaybackRuntime,
}

impl AdminSettingsDocumentKey {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlaybackRuntime => "playback_runtime",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "playback_runtime" => Some(Self::PlaybackRuntime),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSettingsDocumentRecord {
    pub key: AdminSettingsDocumentKey,
    pub payload_json: String,
    pub source: AdminSettingsSource,
    pub effect: AdminSettingsEffect,
    pub updated_at_ms: i64,
}
