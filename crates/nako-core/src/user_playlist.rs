use serde::{Deserialize, Serialize};

use crate::{MediaItemId, UserPlaylistId, UserPrincipalId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserPlaylistVisibility {
    Private,
}

impl UserPlaylistVisibility {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "private" => Some(Self::Private),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistRecord {
    pub id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub name: String,
    pub visibility: UserPlaylistVisibility,
    pub item_count: u32,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistItemRecord {
    pub playlist_id: UserPlaylistId,
    pub item_id: MediaItemId,
    pub position: u32,
    pub added_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewUserPlaylist {
    pub id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub name: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistNameUpdate {
    pub playlist_id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub name: String,
    pub expected_version: Option<u64>,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistItemWrite {
    pub playlist_id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub position: Option<u32>,
    pub expected_version: Option<u64>,
    pub added_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistItemRemoval {
    pub playlist_id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub item_id: MediaItemId,
    pub expected_version: Option<u64>,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UserPlaylistReorder {
    pub playlist_id: UserPlaylistId,
    pub principal_id: UserPrincipalId,
    pub item_ids: Vec<MediaItemId>,
    pub expected_version: Option<u64>,
    pub updated_at_ms: i64,
}
