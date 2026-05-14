use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use taru_core::{Library, LibraryId, Result, TaruError};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TaruServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,
    pub database_url: String,
    #[serde(default = "default_ffprobe_path")]
    pub ffprobe_path: PathBuf,
    #[serde(default = "default_scan_concurrency")]
    pub scan_concurrency: usize,
    #[serde(default = "default_probe_concurrency")]
    pub probe_concurrency: usize,
    pub library: LocalLibraryConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LocalLibraryConfig {
    pub id: LibraryId,
    pub name: String,
    pub root: PathBuf,
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
        scan_concurrency: default_scan_concurrency(),
        probe_concurrency: default_probe_concurrency(),
        library: LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: PathBuf::from("F:/Media/Movies"),
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
    }
}

fn default_listen_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000)
}

fn default_ffprobe_path() -> PathBuf {
    PathBuf::from("ffprobe")
}

const fn default_scan_concurrency() -> usize {
    1
}

const fn default_probe_concurrency() -> usize {
    2
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
            scan_concurrency = 2
            probe_concurrency = 3

            [library]
            id = "018f0000-0000-7000-8000-000000000001"
            name = "Movies"
            root = "F:/Media/Movies"
            "#,
        )
        .unwrap();

        assert_eq!(config.listen_addr, "127.0.0.1:4000".parse().unwrap());
        assert_eq!(config.database_url, "sqlite://taru.db");
        assert_eq!(config.ffprobe_path, PathBuf::from("ffprobe"));
        assert_eq!(config.scan_concurrency, 2);
        assert_eq!(config.probe_concurrency, 3);
        assert_eq!(config.library.name, "Movies");
        assert_eq!(config.library.root, PathBuf::from("F:/Media/Movies"));
        assert_eq!(library_from_config(&config).roots, vec!["local:///"]);
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
        assert_eq!(config.scan_concurrency, 1);
        assert_eq!(config.probe_concurrency, 2);
    }
}
