use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    Router,
    http::{Method, StatusCode as AxumStatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use taru_api::EnqueueMetadataMaintenanceRequest;
use taru_core::{
    CanonicalMetadata, DomainEventKind, DomainEventSubject, EventOutboxRepository, JobId, JobKind,
    JobStatus, LibraryId, MediaItem, MediaItemId, MediaKind, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, MetadataField, MetadataRefreshMode, MetadataRepository, MetadataSource,
    NewStagingManifestRecord, NewTranscodeSession, PageRequest, ProviderRawResponse,
    StagingManifestId, StagingManifestRepository, StagingPurpose, StagingState,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRepository,
    TranscodeSessionState,
};
use taru_core::{ExternalProvider, MetadataMatchKind, MetadataProviderAttemptStatus};
use taru_library::{LibraryScanRequest, LibraryScanner};
use taru_metadata::MetadataRefreshSummary;
use taru_streaming::{ClientPlaybackCapabilities, DirectPlayRangeRequest};
use taru_transcode::{HardwareAcceleration, HardwareAccelerationFallback, RemuxContainer};
use taru_vfs::{
    ByteRange, ObjectKind, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile,
    StorageBackend, StorageCapabilities, StorageUri, VirtualFile,
};
use tokio::{net::TcpListener, sync::Notify};

use super::playback::{
    HlsSourceDisposition, HlsStagingPolicy, RemuxRequestKey, RemuxSourceDisposition,
    RemuxStagingPolicy, source_path_for_ffmpeg_with_backend,
};
use super::staging::ManifestRecordingStorageBackend;
use super::*;
use crate::config::{
    LocalLibraryConfig, MetadataConfig, MetadataMaintenanceConfig, MetadataMaintenancePolicyConfig,
    MetadataProviderConfig, PlaybackConfig, StagingConfig, TranscodeConfig, WebDavLibraryConfig,
};

mod metadata;
mod nfo;
mod playback;
mod staging;
mod startup;
mod storage;

fn remote_media_source(locator: &str) -> MediaSource {
    MediaSource {
        id: MediaSourceId::new(),
        library_id: LibraryId::new(),
        item_id: MediaItemId::new(),
        locator: locator.to_owned(),
        file_name: "Demo.mkv".to_owned(),
        size_bytes: Some(12),
        fingerprint: Some("remote-fingerprint".to_owned()),
    }
}

fn staging_manifest_record(
    id: StagingManifestId,
    local_path: &Path,
    expires_at_ms: Option<i64>,
    active_leases: u32,
) -> NewStagingManifestRecord {
    NewStagingManifestRecord {
        id,
        source_uri: "webdav:///Movies/Demo.mkv".to_owned(),
        source_scheme: "webdav".to_owned(),
        purpose: StagingPurpose::ProbeInput,
        local_path: local_path.display().to_string(),
        size_bytes: Some(3),
        etag: Some("etag-staged".to_owned()),
        fingerprint: Some("fingerprint-staged".to_owned()),
        state: StagingState::Ready,
        created_at_ms: 1,
        updated_at_ms: 1,
        last_accessed_at_ms: 1,
        expires_at_ms,
        active_leases,
        validation_error: None,
    }
}

struct RemotePlaybackBackend {
    bytes: Vec<u8>,
    local_path_hint: Option<PathBuf>,
}

#[derive(Clone)]
struct ConcurrentStageControl {
    in_flight: Arc<AtomicUsize>,
    max_in_flight: Arc<AtomicUsize>,
    released: Arc<AtomicBool>,
    both_entered: Arc<Notify>,
    release_notify: Arc<Notify>,
}

impl ConcurrentStageControl {
    fn new() -> Self {
        Self {
            in_flight: Arc::new(AtomicUsize::new(0)),
            max_in_flight: Arc::new(AtomicUsize::new(0)),
            released: Arc::new(AtomicBool::new(false)),
            both_entered: Arc::new(Notify::new()),
            release_notify: Arc::new(Notify::new()),
        }
    }

    fn release(&self) {
        self.released.store(true, Ordering::SeqCst);
        self.release_notify.notify_waiters();
    }
}

struct ConcurrentStageBackend {
    bytes: Vec<u8>,
    control: ConcurrentStageControl,
}

struct FailingStageBackend {
    len: u64,
    fingerprint: String,
}

#[async_trait]
impl StorageBackend for FailingStageBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.len),
            modified_at: None,
            etag: Some("etag-failing".to_owned()),
            fingerprint: Some(self.fingerprint.clone()),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        Err(TaruError::Storage {
            uri: request.uri.to_string(),
            message: "intentional staging failure".to_owned(),
        })
    }
}

#[async_trait]
impl StorageBackend for ConcurrentStageBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.bytes.len() as u64),
            modified_at: None,
            etag: Some("etag-concurrent".to_owned()),
            fingerprint: Some(format!("fingerprint-{}", uri.path_part())),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: None,
        })
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let current = self.control.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.control
            .max_in_flight
            .fetch_max(current, Ordering::SeqCst);
        if current == 2 {
            self.control.both_entered.notify_waiters();
        }

        while !self.control.released.load(Ordering::SeqCst) {
            self.control.release_notify.notified().await;
        }

        self.control.in_flight.fetch_sub(1, Ordering::SeqCst);
        let path = taru_vfs::deterministic_stage_path(
            &request.root,
            &request.uri,
            Some(&format!("fingerprint-{}", request.uri.path_part())),
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| TaruError::Storage {
                    uri: parent.display().to_string(),
                    message: format!("failed to create test staging directory: {err}"),
                })?;
        }
        tokio::fs::write(&path, &self.bytes)
            .await
            .map_err(|err| TaruError::Storage {
                uri: path.display().to_string(),
                message: format!("failed to write test staging file: {err}"),
            })?;

        Ok(StagedFile {
            uri: request.uri,
            path,
            len: Some(self.bytes.len() as u64),
            etag: Some("etag-concurrent".to_owned()),
            fingerprint: Some("fingerprint-concurrent".to_owned()),
            reused: false,
        })
    }
}

#[async_trait]
impl StorageBackend for RemotePlaybackBackend {
    fn scheme(&self) -> &'static str {
        "webdav"
    }

    async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
        Ok(ObjectMetadata {
            uri: uri.clone(),
            kind: ObjectKind::File,
            len: Some(self.bytes.len() as u64),
            modified_at: None,
            etag: Some("etag-remote".to_owned()),
            fingerprint: Some("remote-fingerprint".to_owned()),
            capabilities: StorageCapabilities::RANGE_READABLE | StorageCapabilities::REMOTE_LATENCY,
            cache: None,
        })
    }

    async fn list(&self, _uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
        Ok(Vec::new())
    }

    async fn open_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<VirtualFile> {
        Ok(VirtualFile {
            uri: uri.clone(),
            range,
            local_path_hint: self.local_path_hint.clone(),
        })
    }

    async fn read_range(&self, _uri: &StorageUri, _range: Option<ByteRange>) -> Result<ReadRange> {
        panic!("direct play should use stream_range instead of read_range");
    }

    async fn stream_range(&self, uri: &StorageUri, range: Option<ByteRange>) -> Result<ReadStream> {
        let bytes = match range {
            Some(range) => {
                let start = range.offset as usize;
                let end = range
                    .length
                    .map(|length| start + length as usize)
                    .unwrap_or(self.bytes.len());
                self.bytes[start..end].to_vec()
            }
            None => self.bytes.clone(),
        };

        Ok(ReadStream::from_bytes(uri.clone(), range, bytes))
    }

    async fn read_to_string(&self, _uri: &StorageUri) -> Result<String> {
        Ok(String::new())
    }

    async fn write_string(&self, _uri: &StorageUri, _content: &str) -> Result<()> {
        Err(TaruError::Unsupported("test backend is read-only"))
    }

    async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
        let path = taru_vfs::deterministic_stage_path(
            &request.root,
            &request.uri,
            Some("remote-fingerprint"),
        )?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| TaruError::Storage {
                    uri: parent.display().to_string(),
                    message: format!("failed to create test staging directory: {err}"),
                })?;
        }
        tokio::fs::write(&path, &self.bytes)
            .await
            .map_err(|err| TaruError::Storage {
                uri: path.display().to_string(),
                message: format!("failed to write test staging file: {err}"),
            })?;

        Ok(StagedFile {
            uri: request.uri,
            path,
            len: Some(self.bytes.len() as u64),
            etag: Some("etag-remote".to_owned()),
            fingerprint: Some("remote-fingerprint".to_owned()),
            reused: false,
        })
    }
}

struct MockWebDavServer {
    addr: std::net::SocketAddr,
}

impl MockWebDavServer {
    async fn start() -> Self {
        let router = Router::new().route("/{*path}", any(mock_webdav_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self { addr }
    }

    fn base_url(&self) -> String {
        format!("http://{}/dav", self.addr)
    }
}

async fn mock_webdav_handler(method: Method, uri: axum::http::Uri) -> Response {
    let path = uri.path();
    if method.as_str() == "PROPFIND" {
        if path.ends_with("/Movies/") || path.ends_with("/Movies") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[
                    MockWebDavFixture {
                        href: "/dav/Movies/",
                        collection: true,
                        len: None,
                        etag: None,
                    },
                    MockWebDavFixture {
                        href: "/dav/Movies/Demo.mkv",
                        collection: false,
                        len: Some(4),
                        etag: Some("etag-demo"),
                    },
                    MockWebDavFixture {
                        href: "/dav/Movies/Demo.nfo",
                        collection: false,
                        len: Some(40),
                        etag: Some("etag-demo-nfo"),
                    },
                ]),
            )
                .into_response();
        }

        if path.ends_with("/Movies/Demo.mkv") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[MockWebDavFixture {
                    href: "/dav/Movies/Demo.mkv",
                    collection: false,
                    len: Some(4),
                    etag: Some("etag-demo"),
                }]),
            )
                .into_response();
        }

        if path.ends_with("/Movies/Demo.nfo") {
            return (
                AxumStatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                mock_multistatus(&[MockWebDavFixture {
                    href: "/dav/Movies/Demo.nfo",
                    collection: false,
                    len: Some(40),
                    etag: Some("etag-demo-nfo"),
                }]),
            )
                .into_response();
        }
    }

    if method == Method::GET && path.ends_with("/Movies/Demo.mkv") {
        return (AxumStatusCode::OK, [(header::CONTENT_LENGTH, "4")], "demo").into_response();
    }
    if method == Method::GET && path.ends_with("/Movies/Demo.nfo") {
        return (
            AxumStatusCode::OK,
            [(header::CONTENT_LENGTH, "40")],
            "<movie><title>Remote NFO</title></movie>",
        )
            .into_response();
    }

    AxumStatusCode::NOT_FOUND.into_response()
}

struct MockWebDavFixture {
    href: &'static str,
    collection: bool,
    len: Option<u64>,
    etag: Option<&'static str>,
}

fn mock_multistatus(fixtures: &[MockWebDavFixture]) -> String {
    let responses = fixtures
            .iter()
            .map(|fixture| {
                let resourcetype = if fixture.collection {
                    "<D:resourcetype><D:collection/></D:resourcetype>"
                } else {
                    "<D:resourcetype/>"
                };
                let len = fixture
                    .len
                    .map(|len| format!("<D:getcontentlength>{len}</D:getcontentlength>"))
                    .unwrap_or_default();
                let etag = fixture
                    .etag
                    .map(|etag| format!("<D:getetag>\"{etag}\"</D:getetag>"))
                    .unwrap_or_default();
                format!(
                    r#"<D:response><D:href>{}</D:href><D:propstat><D:prop>{resourcetype}{len}{etag}</D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>"#,
                    fixture.href
                )
            })
            .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">{responses}</D:multistatus>"#
    )
}

fn fake_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let content =
            "#!/bin/sh\nfor arg do out=\"$arg\"; done\nprintf remuxed > \"$out\"\nexit 0\n";
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
        content.push_str("exit /b 0\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_failing_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let content = "#!/bin/sh\necho remux failed >&2\nexit 7\n";
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("echo remux failed 1>&2\r\n");
        content.push_str("exit /b 7\r\n");
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    hls_ffmpeg_script(root, name, true)
}

fn fake_failing_hls_ffmpeg_script(root: &Path, name: &str) -> PathBuf {
    hls_ffmpeg_script(root, name, false)
}

fn hls_ffmpeg_script(root: &Path, name: &str, success: bool) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        content.push_str("for arg do out=\"$arg\"; done\n");
        content.push_str("dir=$(dirname \"$out\")\n");
        content.push_str("mkdir -p \"$dir\"\n");
        if success {
            content.push_str(
                "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
            );
            content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
            content.push_str("exit 0\n");
        } else {
            content.push_str("printf partial > \"$out\"\n");
            content.push_str("printf hls-failed >&2\n");
            content.push_str("exit 42\n");
        }
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
        content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
        if success {
            content.push_str(">\"%out%\" echo #EXTM3U\r\n");
            content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
            content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
            content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
            content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
            content.push_str("exit /b 0\r\n");
        } else {
            content.push_str("<nul set /p dummy=partial>\"%out%\"\r\n");
            content.push_str("echo hls-failed 1>&2\r\n");
            content.push_str("exit /b 42\r\n");
        }
        fs::write(&path, content).unwrap();
        path
    }
}

async fn remux_app_with_source(
    ffmpeg_path: PathBuf,
) -> (tempfile::TempDir, TaruApp, SqliteStore, MediaSource) {
    remux_app_with_source_and_transcode(ffmpeg_path, TranscodeConfig::default()).await
}

async fn remux_app_with_source_and_transcode(
    ffmpeg_path: PathBuf,
    transcode: TranscodeConfig,
) -> (tempfile::TempDir, TaruApp, SqliteStore, MediaSource) {
    let temp = tempfile::tempdir().unwrap();
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = TaruServerConfig {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path,
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: staging_root,
        metadata: MetadataConfig::default(),
        transcode,
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: taru_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = SqliteStore::connect_in_memory().await.unwrap();
    let app = TaruApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(
            source.id,
            &MediaProbeResult {
                duration_ms: Some(1_000),
                container: Some("matroska,webm".to_owned()),
                bit_rate: None,
                streams: vec![
                    MediaStreamInfo {
                        index: 0,
                        kind: MediaStreamKind::Video,
                        codec: Some("h264".to_owned()),
                        language: None,
                        duration_ms: None,
                        bit_rate: None,
                        width: Some(1920),
                        height: Some(1080),
                        channels: None,
                        sample_rate: None,
                    },
                    MediaStreamInfo {
                        index: 1,
                        kind: MediaStreamKind::Audio,
                        codec: Some("aac".to_owned()),
                        language: None,
                        duration_ms: None,
                        bit_rate: None,
                        width: None,
                        height: None,
                        channels: Some(2),
                        sample_rate: Some(48_000),
                    },
                ],
            },
        )
        .await
        .unwrap();

    (temp, app, store, source)
}
