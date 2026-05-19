use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use taru_core::{
    ExternalProvider, Library, LibraryId, LibraryOptions, LibraryPreset, MediaItemId, MediaKind,
    MetadataProfile, MetadataRefreshMode, Result, SecretString, TaruError,
};
use taru_transcode::{
    HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationPolicy,
    TranscodeResourceBudget,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaruServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,
    pub database_url: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default = "default_ffprobe_path")]
    pub ffprobe_path: PathBuf,
    #[serde(default = "default_ffmpeg_path")]
    pub ffmpeg_path: PathBuf,
    #[serde(default = "default_scan_concurrency")]
    pub scan_concurrency: usize,
    #[serde(default = "default_probe_concurrency")]
    pub probe_concurrency: usize,
    #[serde(default = "default_metadata_concurrency")]
    pub metadata_concurrency: usize,
    #[serde(default = "default_remux_concurrency")]
    pub remux_concurrency: usize,
    #[serde(default = "default_webhook_concurrency")]
    pub webhook_concurrency: usize,
    #[serde(default = "default_remux_timeout_ms")]
    pub remux_timeout_ms: u64,
    #[serde(default = "default_remux_staging_root")]
    pub remux_staging_root: PathBuf,
    #[serde(default)]
    pub metadata: MetadataConfig,
    #[serde(default)]
    pub transcode: TranscodeConfig,
    #[serde(default)]
    pub staging: StagingConfig,
    #[serde(default)]
    pub playback: PlaybackConfig,
    #[serde(default)]
    pub artwork: ArtworkConfig,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<LocalLibraryConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuthConfig {
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    #[serde(
        default = "default_auth_token_env",
        skip_serializing_if = "Option::is_none"
    )]
    pub token_env: Option<String>,
}

impl AuthConfig {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            token_env: None,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: default_auth_enabled(),
            token_env: default_auth_token_env(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalLibraryConfig {
    pub id: LibraryId,
    pub name: String,
    pub root: PathBuf,
    #[serde(default = "default_library_preset")]
    pub preset: LibraryPreset,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webdav: Option<WebDavLibraryConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebDavLibraryConfig {
    pub root: String,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_env: Option<String>,
    #[serde(default = "default_webdav_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_webdav_max_attempts")]
    pub max_attempts: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StagingConfig {
    #[serde(default = "default_staging_max_bytes")]
    pub max_bytes: u64,
    #[serde(default = "default_staging_retention_ms")]
    pub retention_ms: u64,
    #[serde(default = "default_true")]
    pub cleanup_on_startup: bool,
}

impl Default for StagingConfig {
    fn default() -> Self {
        Self {
            max_bytes: default_staging_max_bytes(),
            retention_ms: default_staging_retention_ms(),
            cleanup_on_startup: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackConfig {
    #[serde(default = "default_remote_stream_concurrency")]
    pub remote_stream_concurrency: usize,
    #[serde(default = "default_remote_stage_concurrency")]
    pub remote_stage_concurrency: usize,
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self {
            remote_stream_concurrency: default_remote_stream_concurrency(),
            remote_stage_concurrency: default_remote_stage_concurrency(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtworkConfig {
    #[serde(default = "default_artwork_artifact_root")]
    pub artifact_root: PathBuf,
    #[serde(default = "default_artwork_fetch_timeout_ms")]
    pub fetch_timeout_ms: u64,
    #[serde(default = "default_artwork_fetch_max_attempts")]
    pub fetch_max_attempts: u32,
    #[serde(default = "default_artwork_fetch_max_bytes")]
    pub fetch_max_bytes: u64,
    #[serde(default = "default_artwork_fetch_concurrency")]
    pub fetch_concurrency: usize,
    #[serde(default = "default_artwork_fetch_user_agent")]
    pub fetch_user_agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetch_proxy: Option<SecretString>,
    #[serde(default = "default_artwork_max_dimension")]
    pub max_width: u32,
    #[serde(default = "default_artwork_max_dimension")]
    pub max_height: u32,
}

impl Default for ArtworkConfig {
    fn default() -> Self {
        Self {
            artifact_root: default_artwork_artifact_root(),
            fetch_timeout_ms: default_artwork_fetch_timeout_ms(),
            fetch_max_attempts: default_artwork_fetch_max_attempts(),
            fetch_max_bytes: default_artwork_fetch_max_bytes(),
            fetch_concurrency: default_artwork_fetch_concurrency(),
            fetch_user_agent: default_artwork_fetch_user_agent(),
            fetch_proxy: None,
            max_width: default_artwork_max_dimension(),
            max_height: default_artwork_max_dimension(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataConfig {
    #[serde(default)]
    pub runtime: MetadataProviderRuntimeConfig,
    #[serde(default = "default_metadata_raw_cache_retention_ms")]
    pub raw_cache_retention_ms: u64,
    #[serde(default)]
    pub maintenance: MetadataMaintenanceConfig,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<MetadataProviderConfig>,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            runtime: MetadataProviderRuntimeConfig::default(),
            raw_cache_retention_ms: default_metadata_raw_cache_retention_ms(),
            maintenance: MetadataMaintenanceConfig::default(),
            providers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenanceConfig {
    #[serde(default = "default_metadata_raw_cache_cleanup_on_startup")]
    pub raw_cache_cleanup_on_startup: bool,
    #[serde(default)]
    pub raw_cache_cleanup_interval_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policies: Vec<MetadataMaintenancePolicyConfig>,
}

impl Default for MetadataMaintenanceConfig {
    fn default() -> Self {
        Self {
            raw_cache_cleanup_on_startup: default_metadata_raw_cache_cleanup_on_startup(),
            raw_cache_cleanup_interval_ms: 0,
            policies: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePolicyConfig {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<LibraryId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_ids: Vec<MediaItemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<ExternalProvider>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_kinds: Vec<MediaKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<MetadataProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_mode: Option<MetadataRefreshMode>,
    #[serde(default)]
    pub force: bool,
    #[serde(default = "default_metadata_maintenance_interval_ms")]
    pub interval_ms: u64,
    #[serde(default)]
    pub initial_delay_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderRuntimeConfig {
    #[serde(default = "default_metadata_provider_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_metadata_provider_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_metadata_provider_min_interval_ms")]
    pub min_interval_ms: u64,
    #[serde(default = "default_metadata_provider_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_metadata_provider_user_agent")]
    pub user_agent: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<SecretString>,
    #[serde(default = "default_metadata_provider_circuit_breaker_failures")]
    pub circuit_breaker_failures: u32,
    #[serde(default = "default_metadata_provider_circuit_breaker_backoff_ms")]
    pub circuit_breaker_backoff_ms: u64,
}

impl Default for MetadataProviderRuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: default_metadata_provider_timeout_ms(),
            max_attempts: default_metadata_provider_max_attempts(),
            min_interval_ms: default_metadata_provider_min_interval_ms(),
            concurrency: default_metadata_provider_concurrency(),
            user_agent: default_metadata_provider_user_agent(),
            proxy: None,
            circuit_breaker_failures: default_metadata_provider_circuit_breaker_failures(),
            circuit_breaker_backoff_ms: default_metadata_provider_circuit_breaker_backoff_ms(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderConfig {
    pub provider: ExternalProvider,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default)]
    pub include_adult: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<MetadataProviderHeaderConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<MetadataProviderRuntimeConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderHeaderConfig {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<SecretString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_env: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TranscodeConfig {
    #[serde(default)]
    pub hardware_acceleration: HardwareAcceleration,
    #[serde(default)]
    pub hardware_fallback: HardwareAccelerationFallback,
    #[serde(default = "default_transcode_cpu_concurrency")]
    pub cpu_concurrency: usize,
    #[serde(default = "default_transcode_gpu_concurrency")]
    pub gpu_concurrency: usize,
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            hardware_acceleration: HardwareAcceleration::None,
            hardware_fallback: HardwareAccelerationFallback::Cpu,
            cpu_concurrency: default_transcode_cpu_concurrency(),
            gpu_concurrency: default_transcode_gpu_concurrency(),
        }
    }
}

impl TranscodeConfig {
    #[must_use]
    pub const fn hardware_policy(self) -> HardwareAccelerationPolicy {
        HardwareAccelerationPolicy {
            requested: self.hardware_acceleration,
            fallback: self.hardware_fallback,
        }
    }

    #[must_use]
    pub const fn resource_budget(self) -> TranscodeResourceBudget {
        TranscodeResourceBudget::new(self.cpu_concurrency, self.gpu_concurrency)
    }
}

pub fn load_config(path: &Path) -> Result<TaruServerConfig> {
    let content = fs::read_to_string(path).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to read config {}: {err}", path.display()),
    })?;

    toml::from_str(&content).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to parse config {}: {err}", path.display()),
    })
}

pub fn example_config() -> Result<String> {
    let config = TaruServerConfig {
        listen_addr: default_listen_addr(),
        database_url: "sqlite://taru.db".to_owned(),
        auth: AuthConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: default_ffmpeg_path(),
        scan_concurrency: default_scan_concurrency(),
        probe_concurrency: default_probe_concurrency(),
        metadata_concurrency: default_metadata_concurrency(),
        remux_concurrency: default_remux_concurrency(),
        webhook_concurrency: default_webhook_concurrency(),
        remux_timeout_ms: default_remux_timeout_ms(),
        remux_staging_root: default_remux_staging_root(),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: PathBuf::from("F:/Media/Movies"),
            preset: default_library_preset(),
            webdav: None,
        }],
    };

    toml::to_string_pretty(&config).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to render example config: {err}"),
    })
}

pub fn default_library_from_config(config: &TaruServerConfig) -> Result<Library> {
    libraries_from_config(config)
        .into_iter()
        .next()
        .ok_or_else(|| TaruError::InvalidInput {
            message: "server config must include at least one library".to_owned(),
        })
}

pub fn libraries_from_config(config: &TaruServerConfig) -> Vec<Library> {
    config
        .libraries
        .iter()
        .map(library_from_library_config)
        .collect()
}

pub fn configured_library_config_for(
    config: &TaruServerConfig,
    library_id: LibraryId,
) -> Result<LocalLibraryConfig> {
    config
        .libraries
        .clone()
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| TaruError::NotFound {
            entity: "library",
            id: library_id.to_string(),
        })
}

pub fn library_from_library_config(config: &LocalLibraryConfig) -> Library {
    Library {
        id: config.id,
        name: config.name.clone(),
        roots: vec![configured_library_root(config)],
        options: LibraryOptions::from_preset(config.preset),
    }
}

fn configured_library_root(library: &LocalLibraryConfig) -> String {
    library
        .webdav
        .as_ref()
        .map(|config| config.root.clone())
        .unwrap_or_else(|| "local:///".to_owned())
}

fn default_listen_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000)
}

const fn default_auth_enabled() -> bool {
    true
}

fn default_auth_token_env() -> Option<String> {
    Some("TARU_ADMIN_TOKEN".to_owned())
}

fn default_ffprobe_path() -> PathBuf {
    PathBuf::from("ffprobe")
}

fn default_ffmpeg_path() -> PathBuf {
    PathBuf::from("ffmpeg")
}

const fn default_scan_concurrency() -> usize {
    1
}

const fn default_probe_concurrency() -> usize {
    2
}

const fn default_metadata_concurrency() -> usize {
    2
}

const fn default_remux_concurrency() -> usize {
    1
}

const fn default_webhook_concurrency() -> usize {
    2
}

const fn default_remux_timeout_ms() -> u64 {
    30 * 60 * 1_000
}

const fn default_staging_max_bytes() -> u64 {
    100 * 1024 * 1024 * 1024
}

const fn default_staging_retention_ms() -> u64 {
    7 * 24 * 60 * 60 * 1_000
}

const fn default_remote_stream_concurrency() -> usize {
    8
}

const fn default_remote_stage_concurrency() -> usize {
    2
}

const fn default_true() -> bool {
    true
}

const fn default_webdav_timeout_ms() -> u64 {
    30_000
}

const fn default_webdav_max_attempts() -> u32 {
    2
}

const fn default_transcode_cpu_concurrency() -> usize {
    1
}

const fn default_transcode_gpu_concurrency() -> usize {
    1
}

fn default_remux_staging_root() -> PathBuf {
    PathBuf::from("taru-cache/remux")
}

fn default_artwork_artifact_root() -> PathBuf {
    PathBuf::from("taru-cache/artwork")
}

const fn default_artwork_fetch_timeout_ms() -> u64 {
    10_000
}

const fn default_artwork_fetch_max_attempts() -> u32 {
    2
}

const fn default_artwork_fetch_max_bytes() -> u64 {
    25 * 1024 * 1024
}

const fn default_artwork_fetch_concurrency() -> usize {
    2
}

fn default_artwork_fetch_user_agent() -> String {
    format!("taru/{}", env!("CARGO_PKG_VERSION"))
}

const fn default_artwork_max_dimension() -> u32 {
    20_000
}

const fn default_metadata_provider_timeout_ms() -> u64 {
    10_000
}

const fn default_metadata_provider_max_attempts() -> u32 {
    2
}

const fn default_metadata_provider_min_interval_ms() -> u64 {
    250
}

const fn default_metadata_provider_concurrency() -> usize {
    1
}

fn default_metadata_provider_user_agent() -> String {
    format!("taru/{}", env!("CARGO_PKG_VERSION"))
}

const fn default_metadata_provider_circuit_breaker_failures() -> u32 {
    5
}

const fn default_metadata_provider_circuit_breaker_backoff_ms() -> u64 {
    60_000
}

const fn default_metadata_raw_cache_retention_ms() -> u64 {
    90 * 24 * 60 * 60 * 1_000
}

const fn default_metadata_raw_cache_cleanup_on_startup() -> bool {
    false
}

const fn default_metadata_maintenance_interval_ms() -> u64 {
    24 * 60 * 60 * 1_000
}

fn default_library_preset() -> LibraryPreset {
    LibraryPreset::Movies
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, path::PathBuf};

    use super::*;

    #[test]
    fn config_round_trips_from_toml() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            listen_addr = "127.0.0.1:4000"
            database_url = "sqlite://taru.db"
            ffprobe_path = "ffprobe"
            ffmpeg_path = "ffmpeg"
            scan_concurrency = 2
            probe_concurrency = 3
            metadata_concurrency = 4
            remux_concurrency = 2
            webhook_concurrency = 3
            remux_timeout_ms = 60000
            remux_staging_root = "F:/Taru/cache/remux"

            [auth]
            enabled = true
            token_env = "TARU_ADMIN_TOKEN"

            [transcode]
            hardware_acceleration = "nvenc"
            hardware_fallback = "fail"
            cpu_concurrency = 3
            gpu_concurrency = 2

            [staging]
            max_bytes = 123456789
            retention_ms = 86400000
            cleanup_on_startup = false

            [playback]
            remote_stream_concurrency = 7
            remote_stage_concurrency = 2

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "anime"

            [libraries.webdav]
            root = "webdav:///Movies"
            base_url = "https://webdav.example.test/dav"
            username = "media"
            password_env = "TARU_WEBDAV_PASSWORD"
            timeout_ms = 10000
            max_attempts = 3

            [metadata.runtime]
            timeout_ms = 7000
            max_attempts = 3
            min_interval_ms = 500
            concurrency = 2
            user_agent = "taru-test/1"
            proxy = "http://127.0.0.1:10809"
            circuit_breaker_failures = 4
            circuit_breaker_backoff_ms = 12345

            [[metadata.providers]]
            provider = "tmdb"
            enabled = true
            token_env = "TMDB_READ_ACCESS_TOKEN"
            language = "zh-CN"
            include_adult = false

            [[metadata.providers]]
            provider = "bangumi"
            enabled = true
            token_env = "BANGUMI_TOKEN"
            api_base_url = "https://api.bgm.tv"
            image_base_url = "https://lain.bgm.tv"
            include_adult = true

            [[metadata.providers.headers]]
            name = "X-Test"
            value_env = "BANGUMI_HEADER"

            [[metadata.providers]]
            provider = "douban"
            enabled = true
            api_key_env = "DOUBAN_API_KEY"
            "#,
        )
        .unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:4000".parse().unwrap());
        assert_eq!(config.database_url, "sqlite://taru.db");
        assert!(config.auth.enabled);
        assert_eq!(config.auth.token_env.as_deref(), Some("TARU_ADMIN_TOKEN"));
        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.ffmpeg_path, PathBuf::from("ffmpeg"));
        assert_eq!(config.scan_concurrency, 2);
        assert_eq!(config.probe_concurrency, 3);
        assert_eq!(config.metadata_concurrency, 4);
        assert_eq!(config.remux_concurrency, 2);
        assert_eq!(config.webhook_concurrency, 3);
        assert_eq!(config.remux_timeout_ms, 60_000);
        assert_eq!(
            config.remux_staging_root,
            PathBuf::from("F:/Taru/cache/remux")
        );
        assert_eq!(
            config.transcode.hardware_acceleration,
            HardwareAcceleration::Nvenc
        );
        assert_eq!(
            config.transcode.hardware_fallback,
            HardwareAccelerationFallback::Fail
        );
        assert_eq!(config.transcode.cpu_concurrency, 3);
        assert_eq!(config.transcode.gpu_concurrency, 2);
        assert_eq!(
            config.transcode.hardware_policy(),
            HardwareAccelerationPolicy {
                requested: HardwareAcceleration::Nvenc,
                fallback: HardwareAccelerationFallback::Fail
            }
        );
        assert_eq!(
            config.transcode.resource_budget(),
            TranscodeResourceBudget::new(3, 2)
        );
        assert_eq!(config.staging.max_bytes, 123_456_789);
        assert_eq!(config.staging.retention_ms, 86_400_000);
        assert!(!config.staging.cleanup_on_startup);
        assert_eq!(config.playback.remote_stream_concurrency, 7);
        assert_eq!(config.playback.remote_stage_concurrency, 2);
        assert_eq!(config.metadata.runtime.timeout_ms, 7_000);
        assert_eq!(config.metadata.runtime.max_attempts, 3);
        assert_eq!(config.metadata.runtime.min_interval_ms, 500);
        assert_eq!(config.metadata.runtime.concurrency, 2);
        assert_eq!(config.metadata.runtime.user_agent, "taru-test/1");
        assert_eq!(
            config
                .metadata
                .runtime
                .proxy
                .as_ref()
                .map(SecretString::expose_secret),
            Some("http://127.0.0.1:10809")
        );
        assert_eq!(config.metadata.runtime.circuit_breaker_failures, 4);
        assert_eq!(config.metadata.runtime.circuit_breaker_backoff_ms, 12_345);
        assert_eq!(config.metadata.providers.len(), 3);
        assert_eq!(
            config.metadata.providers[0].provider,
            taru_core::ExternalProvider::Tmdb
        );
        assert_eq!(
            config.metadata.providers[0].token_env.as_deref(),
            Some("TMDB_READ_ACCESS_TOKEN")
        );
        assert_eq!(
            config.metadata.providers[0].language.as_deref(),
            Some("zh-CN")
        );
        assert_eq!(
            config.metadata.providers[1].provider,
            taru_core::ExternalProvider::Bangumi
        );
        assert_eq!(
            config.metadata.providers[1].token_env.as_deref(),
            Some("BANGUMI_TOKEN")
        );
        assert!(config.metadata.providers[1].include_adult);
        assert_eq!(config.metadata.providers[1].headers[0].name, "X-Test");
        assert_eq!(
            config.metadata.providers[2].api_key_env.as_deref(),
            Some("DOUBAN_API_KEY")
        );
        assert_eq!(config.libraries.len(), 1);
        let library = &config.libraries[0];
        assert_eq!(library.name, "Movies");
        assert_eq!(library.root, PathBuf::from("F:/Media/Movies"));
        assert_eq!(library.preset, LibraryPreset::Anime);
        let webdav = library.webdav.as_ref().unwrap();
        assert_eq!(webdav.root, "webdav:///Movies");
        assert_eq!(webdav.base_url, "https://webdav.example.test/dav");
        assert_eq!(webdav.username.as_deref(), Some("media"));
        assert_eq!(webdav.password_env.as_deref(), Some("TARU_WEBDAV_PASSWORD"));
        assert_eq!(webdav.timeout_ms, 10_000);
        assert_eq!(webdav.max_attempts, 3);
        assert_eq!(
            default_library_from_config(&config).unwrap().roots,
            vec!["webdav:///Movies"]
        );
        assert_eq!(
            default_library_from_config(&config)
                .unwrap()
                .options
                .metadata_profile
                .metadata_providers,
            vec![
                taru_core::ExternalProvider::Bangumi,
                taru_core::ExternalProvider::Tmdb,
                taru_core::ExternalProvider::Douban
            ]
        );
    }

    #[test]
    fn config_supports_multiple_libraries() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "movies"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000002"
            name = "Remote Anime"
            root = "F:/unused"
            preset = "anime"

            [libraries.webdav]
            root = "webdav:///Anime"
            base_url = "https://webdav.example.test/dav"
            username = "media"
            password_env = "TARU_WEBDAV_PASSWORD"
            timeout_ms = 15000
            max_attempts = 4
            "#,
        )
        .unwrap();

        assert_eq!(config.libraries.len(), 2);
        assert_eq!(config.libraries[0].name, "Movies");
        assert_eq!(config.libraries[1].name, "Remote Anime");
        assert_eq!(config.libraries[1].preset, LibraryPreset::Anime);
        assert_eq!(
            config.libraries[1].webdav.as_ref().unwrap().root,
            "webdav:///Anime"
        );
        assert_eq!(
            libraries_from_config(&config)
                .into_iter()
                .map(|library| library.roots[0].clone())
                .collect::<Vec<_>>(),
            vec!["local:///", "webdav:///Anime"]
        );
        assert_eq!(
            configured_library_config_for(&config, config.libraries[1].id)
                .unwrap()
                .webdav
                .unwrap()
                .max_attempts,
            4
        );
    }

    #[test]
    fn default_library_from_multi_library_config_returns_first_configured_library() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "movies"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000002"
            name = "Anime"
            root = "F:/Media/Anime"
            preset = "anime"
            "#,
        )
        .unwrap();

        let library = default_library_from_config(&config).unwrap();

        assert_eq!(library.id, config.libraries[0].id);
        assert_eq!(library.name, "Movies");
        assert_eq!(library.options.preset, LibraryPreset::Movies);
    }

    #[test]
    fn config_uses_default_runtime_settings() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(
            config.listen_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000)
        );
        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.ffmpeg_path, PathBuf::from("ffmpeg"));
        assert_eq!(config.scan_concurrency, 1);
        assert_eq!(config.probe_concurrency, 2);
        assert_eq!(config.metadata_concurrency, 2);
        assert_eq!(config.remux_concurrency, 1);
        assert_eq!(config.webhook_concurrency, 2);
        assert_eq!(config.remux_timeout_ms, 30 * 60 * 1_000);
        assert_eq!(config.remux_staging_root, PathBuf::from("taru-cache/remux"));
        assert_eq!(
            config.transcode.hardware_acceleration,
            HardwareAcceleration::None
        );
        assert_eq!(
            config.transcode.hardware_fallback,
            HardwareAccelerationFallback::Cpu
        );
        assert_eq!(config.transcode.cpu_concurrency, 1);
        assert_eq!(config.transcode.gpu_concurrency, 1);
        assert_eq!(config.staging, StagingConfig::default());
        assert_eq!(config.playback, PlaybackConfig::default());
        assert_eq!(config.auth, AuthConfig::default());
        assert_eq!(
            config.metadata.runtime,
            MetadataProviderRuntimeConfig::default()
        );
        assert!(config.metadata.providers.is_empty());
        let library = &config.libraries[0];
        assert_eq!(library.preset, LibraryPreset::Movies);
        assert!(library.webdav.is_none());
        assert_eq!(
            default_library_from_config(&config).unwrap().roots,
            vec!["local:///"]
        );
    }

    #[test]
    fn config_debug_redacts_literal_runtime_and_header_secrets() {
        let mut config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();
        config.metadata.runtime.proxy = Some("http://user:proxy-secret@127.0.0.1:10809".into());
        config.metadata.providers = vec![MetadataProviderConfig {
            provider: ExternalProvider::Douban,
            enabled: true,
            token_env: None,
            api_key_env: None,
            api_base_url: None,
            image_base_url: None,
            language: None,
            include_adult: false,
            headers: vec![MetadataProviderHeaderConfig {
                name: "X-Test".to_owned(),
                value: Some("literal-header-secret".into()),
                value_env: None,
            }],
            runtime: None,
        }];

        let debug = format!("{config:?}");

        assert!(!debug.contains("proxy-secret"));
        assert!(!debug.contains("literal-header-secret"));
        assert!(debug.contains("<redacted>"));
    }
}
