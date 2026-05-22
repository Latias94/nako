use serde::{Deserialize, Serialize};

use crate::{NakoError, Result};

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
pub struct NewVfsCacheFailure {
    pub uri: String,
    pub scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub error: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VfsCacheFailure {
    pub uri: String,
    pub scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub failure_count: u32,
    pub error: String,
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
