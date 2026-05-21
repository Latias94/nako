use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use bitflags::bitflags;
use bytes::Bytes;
use futures_util::{StreamExt, stream::BoxStream};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use taru_core::{Result, StorageErrorKind, TaruError};

mod cache;
mod local;
mod webdav;

pub use cache::{CachedStorageBackend, VfsCacheOptions};
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
    pub cache: Option<ObjectCacheStatus>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectCacheState {
    Fresh,
    StaleFallback,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObjectCacheStatus {
    pub state: ObjectCacheState,
    pub fetched_at_ms: i64,
    pub fresh_until_ms: i64,
    pub last_failed_at_ms: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObjectListing {
    pub entries: Vec<ObjectMetadata>,
    pub cache: Option<ObjectCacheStatus>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadRange {
    pub uri: StorageUri,
    pub range: Option<ByteRange>,
    pub bytes: Vec<u8>,
}

pub type ByteStream = BoxStream<'static, Result<Bytes>>;

pub struct ReadStream {
    pub uri: StorageUri,
    pub range: Option<ByteRange>,
    pub body: ByteStream,
}

impl ReadStream {
    #[must_use]
    pub fn new(uri: StorageUri, range: Option<ByteRange>, body: ByteStream) -> Self {
        Self { uri, range, body }
    }

    #[must_use]
    pub fn from_bytes(uri: StorageUri, range: Option<ByteRange>, bytes: Vec<u8>) -> Self {
        Self {
            uri,
            range,
            body: futures_util::stream::once(async move { Ok(Bytes::from(bytes)) }).boxed(),
        }
    }
}

impl fmt::Debug for ReadStream {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReadStream")
            .field("uri", &self.uri)
            .field("range", &self.range)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRequest {
    pub uri: StorageUri,
    pub root: PathBuf,
}

impl StageRequest {
    pub fn new(uri: StorageUri, root: impl Into<PathBuf>) -> Self {
        Self {
            uri,
            root: root.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedFile {
    pub uri: StorageUri,
    pub path: PathBuf,
    pub len: Option<u64>,
    pub etag: Option<String>,
    pub fingerprint: Option<String>,
    pub reused: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageWriteMode {
    Direct,
    AtomicReplace,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackupMode {
    #[default]
    None,
    ExistingFile,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackupPolicy {
    pub mode: StorageBackupMode,
    pub retention: StorageBackupRetention,
}

impl StorageBackupPolicy {
    #[must_use]
    pub fn none() -> Self {
        Self {
            mode: StorageBackupMode::None,
            retention: StorageBackupRetention::default(),
        }
    }

    #[must_use]
    pub fn existing_file() -> Self {
        Self {
            mode: StorageBackupMode::ExistingFile,
            retention: StorageBackupRetention::default(),
        }
    }

    #[must_use]
    pub fn keep_latest(mut self, count: usize) -> Self {
        self.retention.keep_latest = Some(count);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackupRetention {
    pub keep_latest: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackupReport {
    pub original_uri: StorageUri,
    pub backup_uri: StorageUri,
    #[serde(default)]
    pub pruned_backups: Vec<StorageUri>,
    #[serde(default)]
    pub prune_failures: Vec<StorageBackupPruneFailure>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackupPruneFailure {
    pub uri: StorageUri,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageWriteRequest {
    pub uri: StorageUri,
    pub content: String,
    pub mode: StorageWriteMode,
    pub backup: StorageBackupPolicy,
}

impl StorageWriteRequest {
    #[must_use]
    pub fn direct(uri: StorageUri, content: impl Into<String>) -> Self {
        Self {
            uri,
            content: content.into(),
            mode: StorageWriteMode::Direct,
            backup: StorageBackupPolicy::none(),
        }
    }

    #[must_use]
    pub fn atomic_replace(uri: StorageUri, content: impl Into<String>) -> Self {
        Self {
            uri,
            content: content.into(),
            mode: StorageWriteMode::AtomicReplace,
            backup: StorageBackupPolicy::none(),
        }
    }

    #[must_use]
    pub fn with_backup(mut self, backup: StorageBackupMode) -> Self {
        self.backup = StorageBackupPolicy {
            mode: backup,
            retention: StorageBackupRetention::default(),
        };
        self
    }

    #[must_use]
    pub fn with_backup_policy(mut self, backup: StorageBackupPolicy) -> Self {
        self.backup = backup;
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageWriteReport {
    pub uri: StorageUri,
    pub mode: StorageWriteMode,
    pub atomic: bool,
    pub backup: Option<StorageBackupReport>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageLinkKind {
    Hard,
    Soft,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageLinkPlanRequest {
    pub source_uri: StorageUri,
    pub target_uri: StorageUri,
    pub kind: StorageLinkKind,
}

impl StorageLinkPlanRequest {
    #[must_use]
    pub fn new(source_uri: StorageUri, target_uri: StorageUri, kind: StorageLinkKind) -> Self {
        Self {
            source_uri,
            target_uri,
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageLinkPlanStatus {
    Ready,
    Unsupported,
    SourceMissing,
    SourceNotFile,
    TargetParentMissing,
    TargetParentNotDirectory,
    TargetExists,
    SecurityViolation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageLinkPlan {
    pub source_uri: StorageUri,
    pub target_uri: StorageUri,
    pub kind: StorageLinkKind,
    pub status: StorageLinkPlanStatus,
    pub can_apply: bool,
    pub source: Option<ObjectMetadata>,
    pub target: Option<ObjectMetadata>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageApplyKind {
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageApplyRequest {
    pub source_uri: StorageUri,
    pub target_uri: StorageUri,
    pub kind: StorageApplyKind,
}

impl StorageApplyRequest {
    #[must_use]
    pub fn new(source_uri: StorageUri, target_uri: StorageUri, kind: StorageApplyKind) -> Self {
        Self {
            source_uri,
            target_uri,
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageApplyStatus {
    Applied,
    Unsupported,
    SourceMissing,
    SourceNotFile,
    TargetParentMissing,
    TargetParentNotDirectory,
    TargetExists,
    SecurityViolation,
    ApplyFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageApplyObject {
    pub uri: StorageUri,
    pub kind: ObjectKind,
    pub len: Option<u64>,
    pub etag: Option<String>,
    pub fingerprint_available: bool,
    pub capabilities: StorageCapabilities,
}

impl StorageApplyObject {
    #[must_use]
    pub fn from_metadata(metadata: ObjectMetadata) -> Self {
        Self {
            uri: metadata.uri,
            kind: metadata.kind,
            len: metadata.len,
            etag: metadata.etag,
            fingerprint_available: metadata.fingerprint.is_some(),
            capabilities: metadata.capabilities,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageApplyReport {
    pub source_uri: StorageUri,
    pub target_uri: StorageUri,
    pub kind: StorageApplyKind,
    pub status: StorageApplyStatus,
    pub applied: bool,
    pub target_created: bool,
    pub source: Option<StorageApplyObject>,
    pub target: Option<StorageApplyObject>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageCleanupRequest {
    pub target_uri: StorageUri,
}

impl StorageCleanupRequest {
    #[must_use]
    pub fn new(target_uri: StorageUri) -> Self {
        Self { target_uri }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageCleanupStatus {
    Cleaned,
    Unsupported,
    TargetMissing,
    TargetNotFile,
    SecurityViolation,
    CleanupFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageCleanupReport {
    pub target_uri: StorageUri,
    pub status: StorageCleanupStatus,
    pub cleaned: bool,
    pub target: Option<StorageApplyObject>,
    pub message: String,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn scheme(&self) -> &'static str;

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata>;

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>>;

    async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
        Ok(ObjectListing {
            entries: self.list(uri).await?,
            cache: None,
        })
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile>;

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        let _ = (uri, range);
        Err(TaruError::Unsupported(
            "storage backend does not support in-process range reads",
        ))
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        let _ = (uri, range);
        Err(TaruError::Unsupported(
            "storage backend does not support streaming range reads",
        ))
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String>;

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()>;

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        if request.backup.mode != StorageBackupMode::None {
            return Err(TaruError::Unsupported(
                "storage backend does not support backup writes",
            ));
        }

        match request.mode {
            StorageWriteMode::Direct => {
                self.write_string(&request.uri, &request.content).await?;
                Ok(StorageWriteReport {
                    uri: request.uri,
                    mode: StorageWriteMode::Direct,
                    atomic: false,
                    backup: None,
                })
            }
            StorageWriteMode::AtomicReplace => Err(TaruError::Unsupported(
                "storage backend does not support atomic replace writes",
            )),
        }
    }

    async fn plan_link(&self, request: StorageLinkPlanRequest) -> Result<StorageLinkPlan> {
        Ok(StorageLinkPlan {
            source_uri: request.source_uri,
            target_uri: request.target_uri,
            kind: request.kind,
            status: StorageLinkPlanStatus::Unsupported,
            can_apply: false,
            source: None,
            target: None,
            message: "storage backend does not support link planning".to_owned(),
        })
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<StorageApplyReport> {
        Ok(StorageApplyReport {
            source_uri: request.source_uri,
            target_uri: request.target_uri,
            kind: request.kind,
            status: StorageApplyStatus::Unsupported,
            applied: false,
            target_created: false,
            source: None,
            target: None,
            message: "storage backend does not support storage apply".to_owned(),
        })
    }

    async fn cleanup(&self, request: StorageCleanupRequest) -> Result<StorageCleanupReport> {
        Ok(StorageCleanupReport {
            target_uri: request.target_uri,
            status: StorageCleanupStatus::Unsupported,
            cleaned: false,
            target: None,
            message: "storage backend does not support storage cleanup".to_owned(),
        })
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let _ = request;
        Err(TaruError::Unsupported(
            "storage backend does not support local staging",
        ))
    }
}

#[async_trait]
impl<B> StorageBackend for Box<B>
where
    B: StorageBackend + ?Sized,
{
    fn scheme(&self) -> &'static str {
        self.as_ref().scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.as_ref().stat(uri).await
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        self.as_ref().list(uri).await
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
        self.as_ref().list_with_status(uri).await
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        self.as_ref().open_range(uri, range).await
    }

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        self.as_ref().read_range(uri, range).await
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        self.as_ref().stream_range(uri, range).await
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        self.as_ref().read_to_string(uri).await
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        self.as_ref().write_string(uri, content).await
    }

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        self.as_ref().write(request).await
    }

    async fn plan_link(&self, request: StorageLinkPlanRequest) -> Result<StorageLinkPlan> {
        self.as_ref().plan_link(request).await
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<StorageApplyReport> {
        self.as_ref().apply(request).await
    }

    async fn cleanup(&self, request: StorageCleanupRequest) -> Result<StorageCleanupReport> {
        self.as_ref().cleanup(request).await
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        self.as_ref().stage(request).await
    }
}

#[async_trait]
impl<B> StorageBackend for Arc<B>
where
    B: StorageBackend + ?Sized,
{
    fn scheme(&self) -> &'static str {
        self.as_ref().scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        self.as_ref().stat(uri).await
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        self.as_ref().list(uri).await
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<ObjectListing> {
        self.as_ref().list_with_status(uri).await
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        self.as_ref().open_range(uri, range).await
    }

    async fn read_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadRange> {
        self.as_ref().read_range(uri, range).await
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        self.as_ref().stream_range(uri, range).await
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        self.as_ref().read_to_string(uri).await
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        self.as_ref().write_string(uri, content).await
    }

    async fn write(&self, request: StorageWriteRequest) -> Result<StorageWriteReport> {
        self.as_ref().write(request).await
    }

    async fn plan_link(&self, request: StorageLinkPlanRequest) -> Result<StorageLinkPlan> {
        self.as_ref().plan_link(request).await
    }

    async fn apply(&self, request: StorageApplyRequest) -> Result<StorageApplyReport> {
        self.as_ref().apply(request).await
    }

    async fn cleanup(&self, request: StorageCleanupRequest) -> Result<StorageCleanupReport> {
        self.as_ref().cleanup(request).await
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        self.as_ref().stage(request).await
    }
}

pub fn deterministic_stage_path(
    root: &Path,
    uri: &StorageUri,
    fingerprint: Option<&str>,
) -> Result<PathBuf> {
    if root.as_os_str().is_empty() {
        return Err(TaruError::InvalidInput {
            message: "staging root cannot be empty".to_owned(),
        });
    }
    if root
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(TaruError::InvalidInput {
            message: "staging root must not contain relative path components".to_owned(),
        });
    }

    let mut hasher = Sha256::new();
    hasher.update(uri.as_str().as_bytes());
    hasher.update(b"\n");
    if let Some(fingerprint) = fingerprint {
        hasher.update(fingerprint.as_bytes());
    }
    let digest = hex_encode(&hasher.finalize());
    let extension = uri
        .path_part()
        .rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_stem, extension)| sanitize_extension(extension))
        .filter(|extension| !extension.is_empty());
    let file_name = match extension {
        Some(extension) => format!("{digest}.{extension}"),
        None => digest.clone(),
    };
    let path = root.join(uri.scheme()).join(&digest[0..2]).join(file_name);

    if !path.starts_with(root) {
        return Err(TaruError::storage(
            root.display().to_string(),
            StorageErrorKind::SecurityViolation,
            "staging path escaped staging root",
        ));
    }

    Ok(path)
}

fn sanitize_extension(extension: &str) -> String {
    extension
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .take(16)
        .collect::<String>()
        .to_ascii_lowercase()
}

fn hex_encode(bytes: &[u8]) -> String {
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
