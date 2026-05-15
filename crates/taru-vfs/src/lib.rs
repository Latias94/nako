use std::{fmt, path::PathBuf};

use async_trait::async_trait;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use taru_core::{Result, TaruError};

mod local;
mod webdav;

pub use local::LocalFsBackend;
pub use webdav::{
    EnvWebDavSecretResolver, WebDavBackend, WebDavBackendConfig, WebDavSecretResolver,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct StorageUri(String);

impl StorageUri {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();

        let Some((scheme, _path)) = value.split_once("://") else {
            return Err(TaruError::InvalidInput {
                message: format!("storage uri must include a scheme: {value}"),
            });
        };

        if scheme.is_empty() {
            return Err(TaruError::InvalidInput {
                message: format!("storage uri scheme cannot be empty: {value}"),
            });
        }

        Ok(Self(value))
    }

    pub fn from_parts(scheme: &str, path: &str) -> Result<Self> {
        if scheme.is_empty() {
            return Err(TaruError::InvalidInput {
                message: "storage uri scheme cannot be empty".to_owned(),
            });
        }

        let path = path.trim_start_matches(['/', '\\']);

        if path.is_empty() {
            Self::parse(format!("{scheme}:///"))
        } else {
            Self::parse(format!("{scheme}:///{path}"))
        }
    }

    #[must_use]
    pub fn scheme(&self) -> &str {
        self.0
            .split_once("://")
            .map(|(scheme, _path)| scheme)
            .unwrap_or("")
    }

    #[must_use]
    pub fn path_part(&self) -> &str {
        self.0
            .split_once("://")
            .map(|(_scheme, path)| path)
            .unwrap_or("")
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StorageUri {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct StorageCapabilities: u32 {
        const SEEKABLE = 1 << 0;
        const RANGE_READABLE = 1 << 1;
        const WATCHABLE = 1 << 2;
        const LINKABLE = 1 << 3;
        const WRITABLE = 1 << 4;
        const EXPENSIVE_LISTING = 1 << 5;
        const RATE_LIMITED = 1 << 6;
        const REMOTE_LATENCY = 1 << 7;
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObjectMetadata {
    pub uri: StorageUri,
    pub kind: ObjectKind,
    pub len: Option<u64>,
    pub modified_at: Option<String>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub capabilities: StorageCapabilities,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ByteRange {
    pub offset: u64,
    pub length: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VirtualFile {
    pub uri: StorageUri,
    pub range: Option<ByteRange>,
    pub local_path_hint: Option<PathBuf>,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn scheme(&self) -> &'static str;

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata>;

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>>;

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile>;

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String>;

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_uri_requires_scheme() {
        assert!(StorageUri::parse("local://library/movie.mkv").is_ok());
        assert!(StorageUri::parse("library/movie.mkv").is_err());
    }

    #[test]
    fn storage_uri_builds_root_and_relative_forms() {
        assert_eq!(
            StorageUri::from_parts("local", "").unwrap().as_str(),
            "local:///"
        );
        assert_eq!(
            StorageUri::from_parts("local", "/movies/demo.mkv")
                .unwrap()
                .as_str(),
            "local:///movies/demo.mkv"
        );
    }
}
