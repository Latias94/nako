use std::{
    collections::BTreeMap,
    fmt, fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use nako_core::{
    ExternalProvider, Library, LibraryId, LibraryOptions, LibraryPreset, MediaItemId, MediaKind,
    MetadataProfile, MetadataRefreshMode, NakoError, Result, SecretString,
};
use nako_db::DatabaseBackendKind;
use nako_transcode::{
    HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationPolicy,
    TranscodeResourceBudget,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NakoServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,
    #[serde(default)]
    pub database_backend: DatabaseBackendKind,
    #[serde(default)]
    pub database_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database_url_env: Option<String>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub network: NetworkAccessConfig,
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

#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct NetworkAccessConfig {
    #[serde(default)]
    pub exposure_mode: NetworkExposureMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_base_url: Option<String>,
    #[serde(default)]
    pub trusted_proxy_headers: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_proxy_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tunnel_providers: Vec<TunnelProviderConfig>,
}

impl fmt::Debug for NetworkAccessConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NetworkAccessConfig")
            .field("exposure_mode", &self.exposure_mode)
            .field(
                "external_base_url",
                &self.external_base_url.as_ref().map(|_| "<redacted-url>"),
            )
            .field("trusted_proxy_headers", &self.trusted_proxy_headers)
            .field(
                "trusted_proxy_source_count",
                &self.trusted_proxy_sources.len(),
            )
            .field("allowed_origin_count", &self.allowed_origins.len())
            .field("tunnel_provider_count", &self.tunnel_providers.len())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkExposureMode {
    #[default]
    LocalOnly,
    PrivateNetwork,
    ReverseProxy,
    TunnelProvider,
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct TunnelProviderConfig {
    pub id: String,
    #[serde(default)]
    pub kind: TunnelProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env: Option<String>,
}

impl fmt::Debug for TunnelProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TunnelProviderConfig")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field(
                "public_url",
                &self.public_url.as_ref().map(|_| "<redacted-url>"),
            )
            .field("token_env", &self.token_env)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelProviderKind {
    #[default]
    External,
    CloudflareTunnel,
    TailscaleFunnel,
    Ngrok,
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
    #[serde(default)]
    pub ingest_worker_enabled: bool,
    #[serde(default = "default_artwork_ingest_worker_idle_ms")]
    pub ingest_worker_idle_ms: u64,
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
            ingest_worker_enabled: false,
            ingest_worker_idle_ms: default_artwork_ingest_worker_idle_ms(),
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub library_profiles: BTreeMap<LibraryId, MetadataProfile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<MetadataProviderConfig>,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            runtime: MetadataProviderRuntimeConfig::default(),
            raw_cache_retention_ms: default_metadata_raw_cache_retention_ms(),
            maintenance: MetadataMaintenanceConfig::default(),
            library_profiles: BTreeMap::new(),
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

pub fn load_config(path: &Path) -> Result<NakoServerConfig> {
    let content = fs::read_to_string(path).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to read config {}: {err}", path.display()),
    })?;

    toml::from_str(&content).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to parse config {}: {err}", path.display()),
    })
}

pub fn resolve_database_url(config: &NakoServerConfig) -> Result<String> {
    resolve_database_url_with_env(config, |name| std::env::var(name).ok())
}

fn resolve_database_url_with_env(
    config: &NakoServerConfig,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> Result<String> {
    if let Some(env_name) = config.database_url_env.as_deref() {
        return env_lookup(env_name)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| NakoError::InvalidInput {
                message: format!("database_url_env={env_name} is missing or empty"),
            });
    }

    if config.database_url.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "database_url or database_url_env must be configured".to_owned(),
        });
    }

    Ok(config.database_url.clone())
}

mod preflight;
#[cfg(test)]
use preflight::ConfigPreflightStatus;
#[cfg(test)]
use preflight::preflight_config_with_env;
pub use preflight::{ConfigPreflightOptions, preflight_config, render_config_preflight_text};

pub fn example_config() -> Result<String> {
    let config = NakoServerConfig {
        listen_addr: default_listen_addr(),
        database_backend: DatabaseBackendKind::Sqlite,
        database_url: "sqlite://nako.db".to_owned(),
        database_url_env: None,
        auth: AuthConfig::default(),
        network: NetworkAccessConfig::default(),
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

    toml::to_string_pretty(&config).map_err(|err| NakoError::InvalidInput {
        message: format!("failed to render example config: {err}"),
    })
}

pub fn default_library_from_config(config: &NakoServerConfig) -> Result<Library> {
    libraries_from_config(config)
        .into_iter()
        .next()
        .ok_or_else(|| NakoError::InvalidInput {
            message: "server config must include at least one library".to_owned(),
        })
}

pub fn libraries_from_config(config: &NakoServerConfig) -> Vec<Library> {
    config
        .libraries
        .iter()
        .map(|library| library_from_server_config(config, library))
        .collect()
}

pub fn configured_library_config_for(
    config: &NakoServerConfig,
    library_id: LibraryId,
) -> Result<LocalLibraryConfig> {
    config
        .libraries
        .clone()
        .into_iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| NakoError::NotFound {
            entity: "library",
            id: library_id.to_string(),
        })
}

pub fn library_from_server_config(
    config: &NakoServerConfig,
    library: &LocalLibraryConfig,
) -> Library {
    let mut options = LibraryOptions::from_preset(library.preset);
    if let Some(profile) = config.metadata.library_profiles.get(&library.id) {
        options.metadata_profile = profile.clone();
    }

    Library {
        id: library.id,
        name: library.name.clone(),
        roots: vec![configured_library_root(library)],
        options,
    }
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
    Some("NAKO_ADMIN_TOKEN".to_owned())
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
    PathBuf::from("nako-cache/remux")
}

fn default_artwork_artifact_root() -> PathBuf {
    PathBuf::from("nako-cache/artwork")
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

const fn default_artwork_ingest_worker_idle_ms() -> u64 {
    1_000
}

fn default_artwork_fetch_user_agent() -> String {
    format!("nako/{}", env!("CARGO_PKG_VERSION"))
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
    format!("nako/{}", env!("CARGO_PKG_VERSION"))
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
    use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

    use super::*;

    #[test]
    fn config_round_trips_from_toml() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            listen_addr = "127.0.0.1:4000"
            database_backend = "sqlite"
            database_url = "sqlite://nako.db"
            ffprobe_path = "ffprobe"
            ffmpeg_path = "ffmpeg"
            scan_concurrency = 2
            probe_concurrency = 3
            metadata_concurrency = 4
            remux_concurrency = 2
            webhook_concurrency = 3
            remux_timeout_ms = 60000
            remux_staging_root = "F:/Nako/cache/remux"

            [auth]
            enabled = true
            token_env = "NAKO_ADMIN_TOKEN"

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

            [artwork]
            ingest_worker_enabled = true
            ingest_worker_idle_ms = 250

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "anime"

            [libraries.webdav]
            root = "webdav:///Movies"
            base_url = "https://webdav.example.test/dav"
            username = "media"
            password_env = "NAKO_WEBDAV_PASSWORD"
            timeout_ms = 10000
            max_attempts = 3

            [metadata.runtime]
            timeout_ms = 7000
            max_attempts = 3
            min_interval_ms = 500
            concurrency = 2
            user_agent = "nako-test/1"
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
        assert_eq!(config.database_backend, DatabaseBackendKind::Sqlite);
        assert_eq!(config.database_url, "sqlite://nako.db");
        assert!(config.auth.enabled);
        assert_eq!(config.auth.token_env.as_deref(), Some("NAKO_ADMIN_TOKEN"));
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
            PathBuf::from("F:/Nako/cache/remux")
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
        assert!(config.artwork.ingest_worker_enabled);
        assert_eq!(config.artwork.ingest_worker_idle_ms, 250);
        assert_eq!(config.metadata.runtime.timeout_ms, 7_000);
        assert_eq!(config.metadata.runtime.max_attempts, 3);
        assert_eq!(config.metadata.runtime.min_interval_ms, 500);
        assert_eq!(config.metadata.runtime.concurrency, 2);
        assert_eq!(config.metadata.runtime.user_agent, "nako-test/1");
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
            nako_core::ExternalProvider::Tmdb
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
            nako_core::ExternalProvider::Bangumi
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
        assert_eq!(webdav.password_env.as_deref(), Some("NAKO_WEBDAV_PASSWORD"));
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
                nako_core::ExternalProvider::Bangumi,
                nako_core::ExternalProvider::Tmdb,
                nako_core::ExternalProvider::Douban
            ]
        );
    }

    #[test]
    fn config_supports_multiple_libraries() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

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
            password_env = "NAKO_WEBDAV_PASSWORD"
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
    fn config_applies_library_metadata_profile_overrides() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "movies"

            [metadata.library_profiles.018f0000-0000-7000-8000-000000000001]
            item_kinds = ["movie", "collection", "extra"]
            local_readers = []
            metadata_providers = ["douban"]
            image_providers = []
            language = "zh-CN"
            refresh_mode = "missing_only"
            local_metadata_policy = "disabled"

            [metadata.library_profiles.018f0000-0000-7000-8000-000000000001.scan]
            enabled = false
            "#,
        )
        .unwrap();

        let library = default_library_from_config(&config).unwrap();
        let profile = library.options.metadata_profile;

        assert!(profile.local_readers.is_empty());
        assert_eq!(profile.metadata_providers, vec![ExternalProvider::Douban]);
        assert_eq!(profile.language.as_deref(), Some("zh-CN"));
        assert_eq!(profile.refresh_mode, MetadataRefreshMode::MissingOnly);
        assert_eq!(
            profile.local_metadata_policy,
            nako_core::LocalMetadataPolicy::Disabled
        );
        assert_eq!(profile.scan, nako_core::MetadataScanPolicy::disabled());
        assert!(!profile.scan_acquisition_plan().local_nfo_import);
    }

    #[test]
    fn config_applies_library_metadata_addon_scrape_policy() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "movies"

            [metadata.library_profiles.018f0000-0000-7000-8000-000000000001]
            item_kinds = ["movie", "collection", "extra"]
            local_readers = ["nfo"]
            metadata_providers = ["douban"]
            image_providers = []
            language = "zh-CN"
            refresh_mode = "missing_only"
            local_metadata_policy = "local_first"

            [metadata.library_profiles.018f0000-0000-7000-8000-000000000001.scan]
            enabled = true
            addon_scrape = true
            "#,
        )
        .unwrap();

        let library = default_library_from_config(&config).unwrap();
        let profile = library.options.metadata_profile;

        assert!(profile.scan.enabled);
        assert!(profile.scan.addon_scrape);
        assert!(profile.scan_acquisition_plan().local_nfo_import);
        assert!(profile.scan_acquisition_plan().addon_scrape);
    }

    #[test]
    fn default_library_from_multi_library_config_returns_first_configured_library() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

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
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

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
        assert_eq!(config.database_backend, DatabaseBackendKind::Sqlite);
        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.ffmpeg_path, PathBuf::from("ffmpeg"));
        assert_eq!(config.scan_concurrency, 1);
        assert_eq!(config.probe_concurrency, 2);
        assert_eq!(config.metadata_concurrency, 2);
        assert_eq!(config.remux_concurrency, 1);
        assert_eq!(config.webhook_concurrency, 2);
        assert_eq!(config.remux_timeout_ms, 30 * 60 * 1_000);
        assert_eq!(config.remux_staging_root, PathBuf::from("nako-cache/remux"));
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
        assert!(!config.artwork.ingest_worker_enabled);
        assert_eq!(config.artwork.ingest_worker_idle_ms, 1_000);
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
    fn config_accepts_network_access_policy() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"
            [network]
            exposure_mode = "reverse_proxy"
            external_base_url = "https://nako.example.test"
            trusted_proxy_headers = true
            trusted_proxy_sources = ["127.0.0.1", "10.0.0.0/8"]
            allowed_origins = ["https://app.example.test"]

            [[network.tunnel_providers]]
            id = "cloudflared"
            kind = "cloudflare_tunnel"
            public_url = "https://nako.example.test"
            token_env = "NAKO_TUNNEL_TOKEN"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(
            config.network.exposure_mode,
            NetworkExposureMode::ReverseProxy
        );
        assert_eq!(
            config.network.external_base_url.as_deref(),
            Some("https://nako.example.test")
        );
        assert!(config.network.trusted_proxy_headers);
        assert_eq!(
            config.network.trusted_proxy_sources,
            vec!["127.0.0.1".to_owned(), "10.0.0.0/8".to_owned()]
        );
        assert_eq!(
            config.network.allowed_origins,
            vec!["https://app.example.test".to_owned()]
        );
        assert!(matches!(
            config.network.trusted_proxy_sources[1].parse::<std::net::IpAddr>(),
            Err(_)
        ));
        assert_eq!(config.network.tunnel_providers.len(), 1);
        assert_eq!(config.network.tunnel_providers[0].id, "cloudflared");
        assert_eq!(
            config.network.tunnel_providers[0].kind,
            TunnelProviderKind::CloudflareTunnel
        );
        assert_eq!(
            config.network.tunnel_providers[0].public_url.as_deref(),
            Some("https://nako.example.test")
        );
        assert_eq!(
            config.network.tunnel_providers[0].token_env.as_deref(),
            Some("NAKO_TUNNEL_TOKEN")
        );
    }

    #[test]
    fn config_accepts_explicit_postgres_backend_without_inferring_from_url() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_backend = "postgres"
            database_url = "postgres://nako:secret@db.example.test/nako"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(config.database_backend, DatabaseBackendKind::Postgres);
        assert_eq!(
            config.database_url,
            "postgres://nako:secret@db.example.test/nako"
        );
    }

    #[test]
    fn config_accepts_database_url_env_without_inline_database_url() {
        let config = toml::from_str::<NakoServerConfig>(
            r#"
            database_backend = "postgres"
            database_url_env = "NAKO_DATABASE_URL"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(config.database_backend, DatabaseBackendKind::Postgres);
        assert!(config.database_url.is_empty());
        assert_eq!(
            config.database_url_env.as_deref(),
            Some("NAKO_DATABASE_URL")
        );
    }

    #[test]
    fn config_debug_redacts_literal_runtime_and_header_secrets() {
        let mut config = toml::from_str::<NakoServerConfig>(
            r#"
            database_url = "sqlite://nako.db"

            [[libraries]]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();
        config.metadata.runtime.proxy = Some("http://user:proxy-secret@127.0.0.1:10809".into());
        config.network.external_base_url =
            Some("https://user:network-secret@nako.example.test/path?token=url-secret".to_owned());
        config.network.allowed_origins = vec!["https://operator-secret.example.test".to_owned()];
        config.network.tunnel_providers = vec![TunnelProviderConfig {
            id: "cloudflared".to_owned(),
            kind: TunnelProviderKind::CloudflareTunnel,
            public_url: Some(
                "https://user:tunnel-url-secret@tunnel.example.test/path?token=secret".to_owned(),
            ),
            token_env: Some("NAKO_TUNNEL_TOKEN".to_owned()),
        }];
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
        assert!(!debug.contains("network-secret"));
        assert!(!debug.contains("url-secret"));
        assert!(!debug.contains("operator-secret"));
        assert!(!debug.contains("tunnel-url-secret"));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn config_preflight_accepts_operator_safe_sqlite_config() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        let remux_root = temp.path().join("cache").join("remux");
        let artwork_root = temp.path().join("data").join("artwork");
        fs::create_dir_all(&media_root).unwrap();

        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.remux_staging_root = remux_root.clone();
        config.artwork.artifact_root = artwork_root.clone();

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions { create_dirs: true },
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );

        assert_eq!(report.status, ConfigPreflightStatus::Pass);
        assert!(remux_root.is_dir());
        assert!(artwork_root.is_dir());
        assert!(report.checks.iter().any(|check| {
            check.id == "database.backend_url" && check.status == ConfigPreflightStatus::Pass
        }));
        assert!(report.checks.iter().any(|check| {
            check.id == "auth.token" && check.status == ConfigPreflightStatus::Pass
        }));
    }

    #[test]
    fn config_preflight_rejects_database_backend_url_mismatch_without_leaking_secret_url() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.database_backend = DatabaseBackendKind::Sqlite;
        config.database_url = "postgres://nako:super-secret@db.internal/nako".to_owned();

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );
        let json = serde_json::to_string(&report).unwrap();
        let text = render_config_preflight_text(&report);

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(json.contains("database_backend"));
        assert!(json.contains("url_scheme=postgres"));
        assert!(!json.contains("super-secret"));
        assert!(!json.contains("db.internal"));
        assert!(!text.contains("super-secret"));
        assert!(!text.contains("db.internal"));
    }

    #[test]
    fn config_preflight_rejects_public_bind_without_auth() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.listen_addr = "0.0.0.0:3000".parse().unwrap();
        config.auth = AuthConfig::disabled();

        let report =
            preflight_config_with_env(&config, ConfigPreflightOptions::default(), fake_env([]));

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.bind"
                && check.status == ConfigPreflightStatus::Fail
                && check.summary.contains("public bind")
        }));
    }

    #[test]
    fn config_preflight_rejects_enabled_auth_without_token_env_value() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let config = minimal_config(temp.path().join("nako.db"), media_root);

        let report =
            preflight_config_with_env(&config, ConfigPreflightOptions::default(), fake_env([]));

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "auth.token" && check.status == ConfigPreflightStatus::Fail
        }));
    }

    #[test]
    fn config_preflight_rejects_missing_local_library_without_leaking_path() {
        let temp = tempfile::tempdir().unwrap();
        let missing_root = temp.path().join("missing").join("movies");
        let config = minimal_config(temp.path().join("nako.db"), missing_root.clone());

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id.starts_with("libraries.")
                && check.status == ConfigPreflightStatus::Fail
                && check.summary.contains("does not exist")
        }));
        assert!(!json.contains(missing_root.to_string_lossy().as_ref()));
    }

    #[test]
    fn config_preflight_rejects_duplicate_library_roots_before_startup() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root.clone());
        config.libraries.push(LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies Duplicate".to_owned(),
            root: media_root,
            preset: LibraryPreset::Movies,
            webdav: None,
        });

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "libraries.unique_roots" && check.status == ConfigPreflightStatus::Fail
        }));
    }

    #[test]
    fn config_preflight_fails_unresolved_database_template_without_leaking_password_marker() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.database_backend = DatabaseBackendKind::Postgres;
        config.database_url = "postgres://nako:${NAKO_POSTGRES_PASSWORD}@db/nako".to_owned();

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(json.contains("unresolved template"));
        assert!(!json.contains("NAKO_POSTGRES_PASSWORD"));
        assert!(!json.contains("${"));
    }

    #[test]
    fn config_preflight_accepts_database_url_env_without_leaking_value() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("unused.db"), media_root);
        config.database_backend = DatabaseBackendKind::Postgres;
        config.database_url.clear();
        config.database_url_env = Some("NAKO_DATABASE_URL".to_owned());

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([
                ("NAKO_ADMIN_TOKEN", "secret"),
                (
                    "NAKO_DATABASE_URL",
                    "postgres://nako:db-secret@postgres/nako",
                ),
            ]),
        );
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.status, ConfigPreflightStatus::Warn);
        assert!(json.contains("database_url_env=NAKO_DATABASE_URL"));
        assert!(!json.contains("db-secret"));
        assert!(!json.contains("postgres/nako"));
    }

    #[test]
    fn config_preflight_rejects_reverse_proxy_policy_without_external_base_url() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.listen_addr = "0.0.0.0:3000".parse().unwrap();
        config.network.exposure_mode = NetworkExposureMode::ReverseProxy;
        config.network.trusted_proxy_headers = true;
        config.network.trusted_proxy_sources = vec!["127.0.0.1".to_owned()];

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.access"
                && check.status == ConfigPreflightStatus::Fail
                && check.summary.contains("external_base_url")
        }));
    }

    #[test]
    fn config_preflight_rejects_non_tls_reverse_proxy_external_base_url() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.network.exposure_mode = NetworkExposureMode::ReverseProxy;
        config.network.external_base_url = Some("http://nako.example.test".to_owned());

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.access"
                && check.status == ConfigPreflightStatus::Fail
                && check.detail.contains("https://")
        }));
    }

    #[test]
    fn config_preflight_rejects_trusted_proxy_headers_without_trusted_sources() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.network.exposure_mode = NetworkExposureMode::ReverseProxy;
        config.network.external_base_url = Some("https://nako.example.test".to_owned());
        config.network.trusted_proxy_headers = true;

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.proxy"
                && check.status == ConfigPreflightStatus::Fail
                && check.summary.contains("trusted proxy")
        }));
    }

    #[test]
    fn config_preflight_rejects_path_bearing_browser_origins_without_echoing_them() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.network.allowed_origins =
            vec!["https://user:secret@app.example.test/path?token=origin-secret".to_owned()];

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([("NAKO_ADMIN_TOKEN", "secret")]),
        );
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.status, ConfigPreflightStatus::Fail);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.origins" && check.status == ConfigPreflightStatus::Fail
        }));
        assert!(!json.contains("origin-secret"));
        assert!(!json.contains("user:secret"));
    }

    #[test]
    fn config_preflight_accepts_reverse_proxy_policy_without_leaking_tunnel_secret() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("nako.db"), media_root);
        config.listen_addr = "0.0.0.0:3000".parse().unwrap();
        config.network.exposure_mode = NetworkExposureMode::ReverseProxy;
        config.network.external_base_url = Some("https://nako.example.test".to_owned());
        config.network.trusted_proxy_headers = true;
        config.network.trusted_proxy_sources = vec!["127.0.0.1".to_owned()];
        config.network.allowed_origins = vec!["https://app.example.test".to_owned()];
        config.network.tunnel_providers = vec![TunnelProviderConfig {
            id: "cloudflared".to_owned(),
            kind: TunnelProviderKind::CloudflareTunnel,
            public_url: Some("https://nako.example.test".to_owned()),
            token_env: Some("NAKO_TUNNEL_TOKEN".to_owned()),
        }];

        let report = preflight_config_with_env(
            &config,
            ConfigPreflightOptions::default(),
            fake_env([
                ("NAKO_ADMIN_TOKEN", "admin-secret"),
                ("NAKO_TUNNEL_TOKEN", "tunnel-secret"),
            ]),
        );
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(report.status, ConfigPreflightStatus::Warn);
        assert!(report.checks.iter().any(|check| {
            check.id == "network.access" && check.status == ConfigPreflightStatus::Pass
        }));
        assert!(report.checks.iter().any(|check| {
            check.id == "network.proxy" && check.status == ConfigPreflightStatus::Pass
        }));
        assert!(report.checks.iter().any(|check| {
            check.id == "network.tunnel_providers" && check.status == ConfigPreflightStatus::Pass
        }));
        assert!(!json.contains("admin-secret"));
        assert!(!json.contains("tunnel-secret"));
    }

    #[test]
    fn resolve_database_url_prefers_secret_environment_variable() {
        let temp = tempfile::tempdir().unwrap();
        let media_root = temp.path().join("media");
        fs::create_dir_all(&media_root).unwrap();
        let mut config = minimal_config(temp.path().join("unused.db"), media_root);
        config.database_url = "sqlite://should-not-be-used.db".to_owned();
        config.database_url_env = Some("NAKO_DATABASE_URL".to_owned());

        let database_url = resolve_database_url_with_env(
            &config,
            fake_env([("NAKO_DATABASE_URL", "sqlite://from-env.db")]),
        )
        .unwrap();

        assert_eq!(database_url, "sqlite://from-env.db");
    }

    fn minimal_config(database_path: PathBuf, media_root: PathBuf) -> NakoServerConfig {
        NakoServerConfig {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_backend: DatabaseBackendKind::Sqlite,
            database_url: format!("sqlite://{}", database_path.display()),
            database_url_env: None,
            auth: AuthConfig::default(),
            network: NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: default_scan_concurrency(),
            probe_concurrency: default_probe_concurrency(),
            metadata_concurrency: default_metadata_concurrency(),
            remux_concurrency: default_remux_concurrency(),
            webhook_concurrency: default_webhook_concurrency(),
            remux_timeout_ms: default_remux_timeout_ms(),
            remux_staging_root: PathBuf::from("nako-cache/remux"),
            metadata: MetadataConfig::default(),
            transcode: TranscodeConfig::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: ArtworkConfig::default(),
            libraries: vec![LocalLibraryConfig {
                id: LibraryId::new(),
                name: "Movies".to_owned(),
                root: media_root,
                preset: LibraryPreset::Movies,
                webdav: None,
            }],
        }
    }

    fn fake_env<const N: usize>(
        entries: [(&'static str, &'static str); N],
    ) -> impl Fn(&str) -> Option<String> {
        let values: HashMap<&'static str, &'static str> = entries.into_iter().collect();
        move |name| values.get(name).map(|value| (*value).to_owned())
    }
}
