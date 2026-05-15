use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use taru_core::{Library, LibraryId, LibraryOptions, LibraryPreset, Result, TaruError};
use taru_transcode::{
    HardwareAcceleration, HardwareAccelerationFallback, HardwareAccelerationPolicy,
    TranscodeResourceBudget,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaruServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,
    pub database_url: String,
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
    pub library: LocalLibraryConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalLibraryConfig {
    pub id: LibraryId,
    pub name: String,
    pub root: PathBuf,
    #[serde(default = "default_library_preset")]
    pub preset: LibraryPreset,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataConfig {
    #[serde(default)]
    pub tmdb: TmdbMetadataConfig,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            tmdb: TmdbMetadataConfig::default(),
        }
    }
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TmdbMetadataConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_tmdb_access_token_env")]
    pub access_token_env: String,
    #[serde(default = "default_tmdb_api_base_url")]
    pub api_base_url: String,
    #[serde(default = "default_tmdb_image_base_url")]
    pub image_base_url: String,
    #[serde(default = "default_tmdb_language")]
    pub language: String,
    #[serde(default)]
    pub include_adult: bool,
}

impl Default for TmdbMetadataConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            access_token_env: default_tmdb_access_token_env(),
            api_base_url: default_tmdb_api_base_url(),
            image_base_url: default_tmdb_image_base_url(),
            language: default_tmdb_language(),
            include_adult: false,
        }
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
        library: LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: PathBuf::from("F:/Media/Movies"),
            preset: default_library_preset(),
        },
    };

    toml::to_string_pretty(&config).map_err(|err| TaruError::InvalidInput {
        message: format!("failed to render example config: {err}"),
    })
}

pub fn library_from_config(config: &TaruServerConfig) -> Library {
    Library {
        id: config.library.id,
        name: config.library.name.clone(),
        roots: vec!["local:///".to_owned()],
        options: LibraryOptions::from_preset(config.library.preset),
    }
}

fn default_listen_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000)
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

const fn default_transcode_cpu_concurrency() -> usize {
    1
}

const fn default_transcode_gpu_concurrency() -> usize {
    1
}

fn default_remux_staging_root() -> PathBuf {
    PathBuf::from("taru-cache/remux")
}

fn default_tmdb_access_token_env() -> String {
    "TMDB_READ_ACCESS_TOKEN".to_owned()
}

fn default_tmdb_api_base_url() -> String {
    "https://api.themoviedb.org/3".to_owned()
}

fn default_tmdb_image_base_url() -> String {
    "https://image.tmdb.org/t/p/original".to_owned()
}

fn default_tmdb_language() -> String {
    "en-US".to_owned()
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

            [transcode]
            hardware_acceleration = "nvenc"
            hardware_fallback = "fail"
            cpu_concurrency = 3
            gpu_concurrency = 2

            [library]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            preset = "anime"

            [metadata.tmdb]
            enabled = true
            access_token_env = "TMDB_READ_ACCESS_TOKEN"
            language = "zh-CN"
            include_adult = false
            "#,
        )
        .unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:4000".parse().unwrap());
        assert_eq!(config.database_url, "sqlite://taru.db");
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
        assert!(config.metadata.tmdb.enabled);
        assert_eq!(
            config.metadata.tmdb.access_token_env,
            "TMDB_READ_ACCESS_TOKEN"
        );
        assert_eq!(config.metadata.tmdb.language, "zh-CN");
        assert_eq!(config.library.name, "Movies");
        assert_eq!(config.library.root, PathBuf::from("F:/Media/Movies"));
        assert_eq!(config.library.preset, LibraryPreset::Anime);
        assert_eq!(library_from_config(&config).roots, vec!["local:///"]);
        assert_eq!(
            library_from_config(&config)
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
    fn config_uses_default_runtime_settings() {
        let config = toml::from_str::<TaruServerConfig>(
            r#"
            database_url = "sqlite://taru.db"

            [library]
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
        assert!(!config.metadata.tmdb.enabled);
        assert_eq!(config.library.preset, LibraryPreset::Movies);
        assert_eq!(
            config.metadata.tmdb.access_token_env,
            "TMDB_READ_ACCESS_TOKEN"
        );
    }
}
