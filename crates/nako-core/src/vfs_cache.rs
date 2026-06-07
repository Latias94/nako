use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{LibraryId, NakoError, Result};

pub const VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS: &str = "storage.vfs.cache_repair";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VfsCacheRepairJobAction {
    RefreshCache,
}

impl VfsCacheRepairJobAction {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RefreshCache => "refresh_cache",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "refresh_cache" => Ok(Self::RefreshCache),
            _ => Err(NakoError::Database {
                message: format!("unknown VFS cache repair job action stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VfsCachedObjectKind {
    File,
    Directory,
    Symlink,
    Other,
}

impl VfsCachedObjectKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "file" => Ok(Self::File),
            "directory" => Ok(Self::Directory),
            "symlink" => Ok(Self::Symlink),
            "other" => Ok(Self::Other),
            _ => Err(NakoError::Database {
                message: format!("unknown VFS cached object kind stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VfsCacheOperation {
    Stat,
    List,
}

impl VfsCacheOperation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stat => "stat",
            Self::List => "list",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "stat" => Ok(Self::Stat),
            "list" => Ok(Self::List),
            _ => Err(NakoError::Database {
                message: format!("unknown VFS cache operation stored in database: {value}"),
            }),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCachedObject {
    pub uri: String,
    pub scheme: String,
    pub kind: VfsCachedObjectKind,
    pub len: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub capabilities_bits: u32,
    pub fetched_at_ms: i64,
    pub fresh_until_ms: i64,
}

impl VfsCachedObject {
    #[must_use]
    pub const fn is_fresh_at(&self, now_ms: i64) -> bool {
        self.fresh_until_ms >= now_ms
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCachedListing {
    pub directory: VfsCachedObject,
    pub entries: Vec<VfsCachedObject>,
    pub fetched_at_ms: i64,
    pub fresh_until_ms: i64,
}

impl VfsCachedListing {
    #[must_use]
    pub const fn is_fresh_at(&self, now_ms: i64) -> bool {
        self.fresh_until_ms >= now_ms
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCacheFailureAuthority {
    pub library_id: Option<LibraryId>,
    pub backend_key: Option<String>,
}

impl VfsCacheFailureAuthority {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            library_id: None,
            backend_key: None,
        }
    }

    #[must_use]
    pub fn attributed(library_id: LibraryId, backend_key: impl Into<String>) -> Self {
        Self {
            library_id: Some(library_id),
            backend_key: Some(backend_key.into()),
        }
    }

    #[must_use]
    pub const fn is_present(&self) -> bool {
        self.library_id.is_some() || self.backend_key.is_some()
    }
}

impl Default for VfsCacheFailureAuthority {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewVfsCacheFailure {
    pub uri: String,
    pub scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub error: String,
    pub authority: VfsCacheFailureAuthority,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCacheFailure {
    pub uri: String,
    pub scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub failure_count: u32,
    pub error: String,
    pub authority: VfsCacheFailureAuthority,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCacheRepairJobInput {
    pub action: VfsCacheRepairJobAction,
    pub source_scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub failure_count: u32,
    pub uri_digest: String,
    pub authority: VfsCacheFailureAuthority,
}

impl VfsCacheRepairJobInput {
    pub fn new(
        action: VfsCacheRepairJobAction,
        source_scheme: impl Into<String>,
        operation: VfsCacheOperation,
        failed_at_ms: i64,
        failure_count: u32,
        uri_digest: impl Into<String>,
        authority: VfsCacheFailureAuthority,
    ) -> Result<Self> {
        let source_scheme = source_scheme.into();
        validate_vfs_cache_repair_source_scheme(&source_scheme)?;
        if failure_count == 0 {
            return Err(NakoError::InvalidInput {
                message: "VFS cache repair job failure_count must be greater than zero".to_owned(),
            });
        }

        let uri_digest = uri_digest.into();
        validate_vfs_cache_repair_uri_digest(&uri_digest)?;

        Ok(Self {
            action,
            source_scheme,
            operation,
            failed_at_ms,
            failure_count,
            uri_digest,
            authority,
        })
    }

    pub fn from_failure(failure: &VfsCacheFailure) -> Result<Self> {
        Self::new(
            VfsCacheRepairJobAction::RefreshCache,
            failure.scheme.clone(),
            failure.operation,
            failure.failed_at_ms,
            failure.failure_count,
            vfs_cache_repair_uri_digest(&failure.uri),
            failure.authority.clone(),
        )
    }

    #[must_use]
    pub fn matches_failure(&self, failure: &VfsCacheFailure) -> bool {
        self.action == VfsCacheRepairJobAction::RefreshCache
            && self.source_scheme == failure.scheme
            && self.operation == failure.operation
            && self.failed_at_ms == failure.failed_at_ms
            && self.failure_count == failure.failure_count
            && self.uri_digest == vfs_cache_repair_uri_digest(&failure.uri)
            && self.authority == failure.authority
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCacheSummary {
    pub object_count: u64,
    pub listing_count: u64,
    pub failure_count: u64,
    pub stale_object_count: u64,
    pub stale_listing_count: u64,
    pub last_failure_at_ms: Option<i64>,
}

#[must_use]
pub fn vfs_cache_repair_uri_digest(uri: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"nako:vfs-cache-repair-uri:v1");
    hasher.update(uri.len().to_be_bytes());
    hasher.update(uri.as_bytes());
    format!("sha256:{}", lowercase_hex(&hasher.finalize()))
}

fn validate_vfs_cache_repair_source_scheme(source_scheme: &str) -> Result<()> {
    if source_scheme.is_empty() || source_scheme.trim() != source_scheme {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job source scheme must be non-empty and trimmed".to_owned(),
        });
    }
    if !source_scheme
        .chars()
        .next()
        .is_some_and(|value| value.is_ascii_alphabetic())
        || !source_scheme.chars().all(is_storage_scheme_character)
    {
        return Err(NakoError::InvalidInput {
            message:
                "VFS cache repair job source scheme must contain only scheme-safe ASCII characters"
                    .to_owned(),
        });
    }

    Ok(())
}

fn validate_vfs_cache_repair_uri_digest(uri_digest: &str) -> Result<()> {
    let Some(digest) = uri_digest.strip_prefix("sha256:") else {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job URI digest must be a sha256 digest".to_owned(),
        });
    };
    if digest.len() != 64 || !digest.chars().all(|value| value.is_ascii_hexdigit()) {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job URI digest must be a sha256 digest".to_owned(),
        });
    }

    Ok(())
}

fn is_storage_scheme_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '+' | '-' | '.' | '_')
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
    fn vfs_cache_repair_job_input_from_failure_serializes_without_raw_uri() {
        let library_id = LibraryId::new();
        let failure = VfsCacheFailure {
            uri: "local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret".to_owned(),
            scheme: "local".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_000,
            failure_count: 2,
            error: "raw backend error with /Users/ExampleUser and token=secret".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                library_id,
                format!("library:{library_id}:local"),
            ),
        };

        let input = VfsCacheRepairJobInput::from_failure(&failure).unwrap();
        let serialized = serde_json::to_string(&input).unwrap();
        let round_trip: VfsCacheRepairJobInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(round_trip, input);
        assert!(input.matches_failure(&failure));
        assert_eq!(input.action, VfsCacheRepairJobAction::RefreshCache);
        assert_eq!(input.source_scheme, "local");
        assert_eq!(input.operation, VfsCacheOperation::Stat);
        assert!(input.uri_digest.starts_with("sha256:"));
        assert!(!serialized.contains("Hidden Movie"));
        assert!(!serialized.contains("Secret Path"));
        assert!(!serialized.contains("ExampleUser"));
        assert!(!serialized.contains("token=secret"));
        assert!(!serialized.contains("local:///"));
        assert!(!serialized.contains("raw backend error"));
    }

    #[test]
    fn vfs_cache_repair_job_input_rejects_locator_like_scheme_without_leaking_value() {
        let err = VfsCacheRepairJobInput::new(
            VfsCacheRepairJobAction::RefreshCache,
            "local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv",
            VfsCacheOperation::Stat,
            1_000,
            1,
            vfs_cache_repair_uri_digest("local:///hidden"),
            VfsCacheFailureAuthority::none(),
        )
        .unwrap_err();

        let NakoError::InvalidInput { message } = err else {
            panic!("expected invalid input");
        };
        assert_eq!(
            message,
            "VFS cache repair job source scheme must contain only scheme-safe ASCII characters"
        );
        assert!(!message.contains("Hidden Movie"));
        assert!(!message.contains("Secret Path"));
        assert!(!message.contains("ExampleUser"));
        assert!(!message.contains("local:///"));
    }

    #[test]
    fn vfs_cache_repair_job_input_rejects_invalid_digest_without_leaking_value() {
        let err = VfsCacheRepairJobInput::new(
            VfsCacheRepairJobAction::RefreshCache,
            "webdav",
            VfsCacheOperation::List,
            1_000,
            1,
            "webdav:///Secret Path?token=secret",
            VfsCacheFailureAuthority::none(),
        )
        .unwrap_err();

        let NakoError::InvalidInput { message } = err else {
            panic!("expected invalid input");
        };
        assert_eq!(
            message,
            "VFS cache repair job URI digest must be a sha256 digest"
        );
        assert!(!message.contains("Secret Path"));
        assert!(!message.contains("token=secret"));
        assert!(!message.contains("webdav:///"));
    }

    #[test]
    fn vfs_cache_repair_job_action_round_trips() {
        assert_eq!(
            VfsCacheRepairJobAction::RefreshCache.as_str(),
            "refresh_cache"
        );
        assert_eq!(
            VfsCacheRepairJobAction::parse("refresh_cache").unwrap(),
            VfsCacheRepairJobAction::RefreshCache
        );
    }
}
