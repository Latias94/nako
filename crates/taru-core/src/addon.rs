use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{AddonGrantId, AddonId, AddonTokenId, LibraryId, Result, SecretString, TaruError};

pub const ADDON_TOKEN_RAW_PREFIX: &str = "taru_at_";
pub const ADDON_TOKEN_DISPLAY_PREFIX_LEN: usize = 18;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonStatus {
    Enabled,
    Disabled,
}

impl AddonStatus {
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
                message: format!("unknown addon status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonTokenStatus {
    Active,
    Revoked,
    Rotated,
}

impl AddonTokenStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Revoked => "revoked",
            Self::Rotated => "rotated",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "revoked" => Ok(Self::Revoked),
            "rotated" => Ok(Self::Rotated),
            _ => Err(TaruError::Database {
                message: format!("unknown addon token status stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonPermission {
    MetadataWrite,
    ArtworkWrite,
    SubtitleWrite,
    LibraryFileWrite,
}

impl AddonPermission {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataWrite => "metadata_write",
            Self::ArtworkWrite => "artwork_write",
            Self::SubtitleWrite => "subtitle_write",
            Self::LibraryFileWrite => "library_file_write",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "metadata_write" => Ok(Self::MetadataWrite),
            "artwork_write" => Ok(Self::ArtworkWrite),
            "subtitle_write" => Ok(Self::SubtitleWrite),
            "library_file_write" => Ok(Self::LibraryFileWrite),
            _ => Err(TaruError::Database {
                message: format!("unknown addon permission stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAddonRegistration {
    pub id: AddonId,
    pub manifest_id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub base_url: String,
    pub manifest_json: String,
    pub granted_scopes: Vec<String>,
    pub status: AddonStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRegistrationRecord {
    pub id: AddonId,
    pub manifest_id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub base_url: String,
    pub manifest_json: String,
    pub granted_scopes: Vec<String>,
    pub status: AddonStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAddonToken {
    pub id: AddonTokenId,
    pub addon_id: AddonId,
    pub label: String,
    pub token_prefix: String,
    pub token_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokenRecord {
    pub id: AddonTokenId,
    pub addon_id: AddonId,
    pub label: String,
    pub token_prefix: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub status: AddonTokenStatus,
    pub created_at: String,
    pub rotated_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewAddonGrant {
    pub id: AddonGrantId,
    pub addon_id: AddonId,
    pub permission: AddonPermission,
    pub library_id: Option<LibraryId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonGrantRecord {
    pub id: AddonGrantId,
    pub addon_id: AddonId,
    pub permission: AddonPermission,
    pub library_id: Option<LibraryId>,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddonIssuedToken {
    pub raw_token: SecretString,
    pub token_prefix: String,
    pub token_hash: String,
}

impl AddonIssuedToken {
    #[must_use]
    pub fn generate() -> Self {
        let raw_token = format!(
            "{}{}{}{}{}",
            ADDON_TOKEN_RAW_PREFIX,
            token_uuid_component(),
            token_uuid_component(),
            token_uuid_component(),
            token_uuid_component()
        );
        let token_prefix = addon_token_display_prefix(&raw_token);
        let token_hash = hash_addon_token(&raw_token);

        Self {
            raw_token: SecretString::new(raw_token),
            token_prefix,
            token_hash,
        }
    }
}

#[must_use]
pub fn hash_addon_token(raw_token: &str) -> String {
    let digest = Sha256::digest(raw_token.as_bytes());
    format!("sha256:{}", lowercase_hex(&digest))
}

#[must_use]
pub fn addon_token_display_prefix(raw_token: &str) -> String {
    raw_token
        .chars()
        .take(ADDON_TOKEN_DISPLAY_PREFIX_LEN)
        .collect()
}

fn token_uuid_component() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_addon_token_has_raw_secret_prefix_and_hash() {
        let token = AddonIssuedToken::generate();
        let raw = token.raw_token.expose_secret();

        assert!(raw.starts_with(ADDON_TOKEN_RAW_PREFIX));
        assert_eq!(token.token_prefix, addon_token_display_prefix(raw));
        assert_eq!(token.token_hash, hash_addon_token(raw));
        assert!(token.token_hash.starts_with("sha256:"));
        assert_ne!(token.token_hash, raw);
        assert_eq!(format!("{:?}", token.raw_token), "<redacted>");
    }
}
