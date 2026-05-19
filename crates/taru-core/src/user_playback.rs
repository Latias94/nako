use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{MediaItemId, MediaSourceId, Result, TaruError};

pub const LOCAL_ADMIN_PRINCIPAL_ID: &str = "local-admin";

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct UserPrincipalId(String);

impl UserPrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "user principal id cannot be empty".to_owned(),
            });
        }
        if trimmed.len() != value.len() {
            return Err(TaruError::InvalidInput {
                message: "user principal id cannot contain leading or trailing whitespace"
                    .to_owned(),
            });
        }
        if value.chars().any(char::is_control) {
            return Err(TaruError::InvalidInput {
                message: "user principal id cannot contain control characters".to_owned(),
            });
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn local_admin() -> Self {
        Self(LOCAL_ADMIN_PRINCIPAL_ID.to_owned())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserPrincipalId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for UserPrincipalId {
    type Err = TaruError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserPlaybackState {
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub source_id: Option<MediaSourceId>,
    pub resume_position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub watched: bool,
    pub watched_at_ms: Option<i64>,
    pub last_played_at_ms: Option<i64>,
    pub updated_at_ms: i64,
    pub version: u64,
}

impl UserPlaybackState {
    #[must_use]
    pub fn progress_percent(&self) -> Option<f32> {
        let position = self.resume_position_ms?;
        let duration = self.duration_ms?;
        if duration == 0 {
            return None;
        }

        Some((position as f32 / duration as f32).clamp(0.0, 1.0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserPlaybackStateWrite {
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub source_id: Option<MediaSourceId>,
    pub resume_position_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub watched: bool,
    pub watched_at_ms: Option<i64>,
    pub last_played_at_ms: Option<i64>,
    pub updated_at_ms: i64,
}
