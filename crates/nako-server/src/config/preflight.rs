use std::{collections::HashSet, fs, io, path::Path};

use nako_db::DatabaseBackendKind;
use serde::Serialize;

use super::{LocalLibraryConfig, NakoServerConfig, NetworkAccessConfig, NetworkExposureMode};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConfigPreflightOptions {
    pub create_dirs: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigPreflightStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigPreflightReport {
    pub status: ConfigPreflightStatus,
    pub checks: Vec<ConfigPreflightCheck>,
}

impl ConfigPreflightReport {
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.status == ConfigPreflightStatus::Fail
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigPreflightCheck {
    pub id: String,
    pub status: ConfigPreflightStatus,
    pub summary: String,
    pub detail: String,
}

pub fn preflight_config(
    config: &NakoServerConfig,
    options: ConfigPreflightOptions,
) -> ConfigPreflightReport {
    preflight_config_with_env(config, options, |name| std::env::var(name).ok())
}

pub fn render_config_preflight_text(report: &ConfigPreflightReport) -> String {
    let mut output = format!("Nako config preflight: {}\n", report.status.as_label());
    for check in &report.checks {
        output.push_str(&format!(
            "- [{}] {}: {}",
            check.status.as_label(),
            check.id,
            check.summary
        ));
        if !check.detail.is_empty() {
            output.push_str(&format!(" ({})", check.detail));
        }
        output.push('\n');
    }
    output
}

pub(super) fn preflight_config_with_env(
    config: &NakoServerConfig,
    options: ConfigPreflightOptions,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> ConfigPreflightReport {
    let mut checks = Vec::new();

    checks.push(check_database_backend_url(config, &env_lookup));
    checks.extend(check_bind_and_auth(config, &env_lookup));
    checks.extend(check_network_access_policy(config, &env_lookup));
    checks.extend(check_runtime_directories(config, options));
    checks.extend(check_libraries(config));

    ConfigPreflightReport {
        status: checks
            .iter()
            .fold(ConfigPreflightStatus::Pass, |status, check| {
                status.max(check.status)
            }),
        checks,
    }
}

fn check_database_backend_url(
    config: &NakoServerConfig,
    env_lookup: &impl Fn(&str) -> Option<String>,
) -> ConfigPreflightCheck {
    let (database_url, source_detail) = match effective_database_url(config, env_lookup) {
        Ok(value) => value,
        Err(check) => return check,
    };
    let scheme = database_url_scheme(&database_url);
    let expected = match config.database_backend {
        DatabaseBackendKind::Sqlite => "sqlite",
        DatabaseBackendKind::Postgres => "postgres or postgresql",
    };
    let backend = config.database_backend.to_string();

    if database_url.contains("${") {
        return preflight_check(
            "database.backend_url",
            ConfigPreflightStatus::Fail,
            "database_url contains an unresolved template marker",
            format!(
                "database_backend={backend}; url_scheme={scheme}; render secrets before Nako reads the config"
            ),
        );
    }

    let scheme_matches = match config.database_backend {
        DatabaseBackendKind::Sqlite => scheme == "sqlite",
        DatabaseBackendKind::Postgres => scheme == "postgres" || scheme == "postgresql",
    };

    if !scheme_matches {
        return preflight_check(
            "database.backend_url",
            ConfigPreflightStatus::Fail,
            "database_backend and database_url scheme do not match",
            format!(
                "database_backend={backend}; expected_url_scheme={expected}; url_scheme={scheme}"
            ),
        );
    }

    if config.database_backend == DatabaseBackendKind::Sqlite
        && database_url.trim() == "sqlite::memory:"
    {
        return preflight_check(
            "database.backend_url",
            ConfigPreflightStatus::Fail,
            "sqlite::memory: is not valid for a packaged self-hosted server",
            "use an on-disk sqlite path for durable state",
        );
    }

    preflight_check(
        "database.backend_url",
        ConfigPreflightStatus::Pass,
        "database backend and URL scheme are compatible",
        format!("database_backend={backend}; {source_detail}; url_scheme={scheme}"),
    )
}

fn effective_database_url(
    config: &NakoServerConfig,
    env_lookup: &impl Fn(&str) -> Option<String>,
) -> std::result::Result<(String, String), ConfigPreflightCheck> {
    if let Some(env_name) = config.database_url_env.as_deref() {
        return env_lookup(env_name)
            .filter(|value| !value.trim().is_empty())
            .map(|value| (value, format!("database_url_env={env_name}")))
            .ok_or_else(|| {
                preflight_check(
                    "database.backend_url",
                    ConfigPreflightStatus::Fail,
                    "database_url_env is configured but the environment variable is missing or empty",
                    format!("database_url_env={env_name}"),
                )
            });
    }

    if config.database_url.trim().is_empty() {
        return Err(preflight_check(
            "database.backend_url",
            ConfigPreflightStatus::Fail,
            "database_url or database_url_env must be configured",
            "set an inline database_url or point database_url_env at a secret environment variable",
        ));
    }

    Ok((
        config.database_url.clone(),
        "database_url=inline".to_owned(),
    ))
}

fn check_bind_and_auth(
    config: &NakoServerConfig,
    env_lookup: &impl Fn(&str) -> Option<String>,
) -> Vec<ConfigPreflightCheck> {
    let mut checks = Vec::new();
    let listen_ip = config.listen_addr.ip();

    checks.push(if listen_ip.is_unspecified() && !config.auth.enabled {
        preflight_check(
            "network.bind",
            ConfigPreflightStatus::Fail,
            "public bind with disabled auth is unsafe",
            "bind to loopback or enable bearer auth before exposing the server",
        )
    } else if listen_ip.is_unspecified() {
        preflight_check(
            "network.bind",
            ConfigPreflightStatus::Warn,
            "server is bound on all interfaces",
            "use only behind a trusted reverse proxy, VPN, tunnel, or private network",
        )
    } else if !config.auth.enabled {
        preflight_check(
            "network.bind",
            ConfigPreflightStatus::Warn,
            "auth is disabled",
            "loopback-only development use is acceptable; enable auth before broader exposure",
        )
    } else {
        preflight_check(
            "network.bind",
            ConfigPreflightStatus::Pass,
            "network bind/auth shape is conservative",
            "auth enabled and bind is not all-interfaces",
        )
    });

    checks.push(if !config.auth.enabled {
        preflight_check(
            "auth.token",
            ConfigPreflightStatus::Warn,
            "bearer auth is disabled",
            "do not expose this instance outside a trusted local boundary",
        )
    } else if let Some(token_env) = config.auth.token_env.as_deref() {
        match env_lookup(token_env).filter(|value| !value.trim().is_empty()) {
            Some(_) => preflight_check(
                "auth.token",
                ConfigPreflightStatus::Pass,
                "bearer token environment variable is present",
                format!("token_env={token_env}"),
            ),
            None => preflight_check(
                "auth.token",
                ConfigPreflightStatus::Fail,
                "auth is enabled but the bearer token environment variable is missing or empty",
                format!("set token_env={token_env} before starting Nako"),
            ),
        }
    } else {
        preflight_check(
            "auth.token",
            ConfigPreflightStatus::Fail,
            "auth is enabled but no token_env is configured",
            "set [auth].token_env to an environment variable containing the admin bearer token",
        )
    });

    checks
}

fn check_network_access_policy(
    config: &NakoServerConfig,
    env_lookup: &impl Fn(&str) -> Option<String>,
) -> Vec<ConfigPreflightCheck> {
    let mut checks = Vec::new();
    let network = &config.network;

    checks.push(check_network_access_mode(config));
    checks.push(check_trusted_proxy_policy(network));
    checks.push(check_origin_policy(network));
    checks.push(check_tunnel_provider_policy(network, env_lookup));

    checks
}

fn check_network_access_mode(config: &NakoServerConfig) -> ConfigPreflightCheck {
    let network = &config.network;
    let has_external_base_url = has_https_url(network.external_base_url.as_deref());
    let listen_ip = config.listen_addr.ip();

    match network.exposure_mode {
        NetworkExposureMode::LocalOnly => {
            if network.external_base_url.is_some()
                || network.trusted_proxy_headers
                || !network.trusted_proxy_sources.is_empty()
                || !network.tunnel_providers.is_empty()
            {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Warn,
                    "local-only network mode ignores remote access settings",
                    "remove external_base_url, trusted proxy, and tunnel settings or choose a remote exposure_mode",
                );
            }
            preflight_check(
                "network.access",
                ConfigPreflightStatus::Pass,
                "network access mode is local-only",
                "remote access policy disabled",
            )
        }
        NetworkExposureMode::PrivateNetwork => {
            if !config.auth.enabled {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "private-network exposure requires bearer auth",
                    "enable [auth] before exposing Nako beyond loopback",
                );
            }
            preflight_check(
                "network.access",
                ConfigPreflightStatus::Pass,
                "private-network exposure policy is explicit",
                if listen_ip.is_unspecified() {
                    "auth enabled; bind is all-interfaces"
                } else {
                    "auth enabled"
                },
            )
        }
        NetworkExposureMode::ReverseProxy => {
            if !config.auth.enabled {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "reverse-proxy exposure requires bearer auth",
                    "enable [auth] before trusting a reverse proxy",
                );
            }
            if !has_external_base_url {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "reverse-proxy exposure requires external_base_url",
                    "set [network].external_base_url to the https:// URL served by the proxy",
                );
            }
            preflight_check(
                "network.access",
                ConfigPreflightStatus::Pass,
                "reverse-proxy exposure policy is explicit",
                "external_base_url is configured; raw URL redacted",
            )
        }
        NetworkExposureMode::TunnelProvider => {
            if !config.auth.enabled {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "tunnel-provider exposure requires bearer auth",
                    "enable [auth] before exposing Nako through a tunnel",
                );
            }
            if !has_external_base_url {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "tunnel-provider exposure requires external_base_url",
                    "set [network].external_base_url to the public https:// tunnel URL",
                );
            }
            if network.tunnel_providers.is_empty() {
                return preflight_check(
                    "network.access",
                    ConfigPreflightStatus::Fail,
                    "tunnel-provider exposure requires at least one provider declaration",
                    "add [[network.tunnel_providers]] with a token_env reference",
                );
            }
            preflight_check(
                "network.access",
                ConfigPreflightStatus::Pass,
                "tunnel-provider exposure policy is explicit",
                "external_base_url and provider declarations are configured; raw URLs redacted",
            )
        }
    }
}

fn check_trusted_proxy_policy(network: &NetworkAccessConfig) -> ConfigPreflightCheck {
    if network.trusted_proxy_headers && network.trusted_proxy_sources.is_empty() {
        return preflight_check(
            "network.proxy",
            ConfigPreflightStatus::Fail,
            "trusted proxy headers require trusted proxy sources",
            "configure trusted_proxy_sources before accepting X-Forwarded-* headers",
        );
    }

    if !network.trusted_proxy_headers && !network.trusted_proxy_sources.is_empty() {
        return preflight_check(
            "network.proxy",
            ConfigPreflightStatus::Warn,
            "trusted proxy sources are configured but forwarded headers are disabled",
            "enable trusted_proxy_headers only after the proxy source boundary is reviewed",
        );
    }

    if network.trusted_proxy_headers {
        preflight_check(
            "network.proxy",
            ConfigPreflightStatus::Pass,
            "trusted proxy header policy is explicit",
            format!(
                "trusted_proxy_source_count={}",
                network.trusted_proxy_sources.len()
            ),
        )
    } else {
        preflight_check(
            "network.proxy",
            ConfigPreflightStatus::Pass,
            "forwarded headers are not trusted",
            "default-deny proxy header policy",
        )
    }
}

fn check_origin_policy(network: &NetworkAccessConfig) -> ConfigPreflightCheck {
    if network
        .allowed_origins
        .iter()
        .any(|origin| origin.trim() == "*")
    {
        return preflight_check(
            "network.origins",
            ConfigPreflightStatus::Fail,
            "wildcard browser origins are not allowed",
            "configure explicit https:// origins for browser clients",
        );
    }

    if network
        .allowed_origins
        .iter()
        .any(|origin| !has_http_origin(Some(origin)))
    {
        return preflight_check(
            "network.origins",
            ConfigPreflightStatus::Fail,
            "allowed origins must be HTTP(S) origins",
            "remove blank, wildcard, non-HTTP, path-bearing, query-bearing, or credential-bearing origins",
        );
    }

    if network.allowed_origins.is_empty() {
        preflight_check(
            "network.origins",
            ConfigPreflightStatus::Pass,
            "no browser origins are configured",
            "CORS remains default-deny until origins are explicitly configured",
        )
    } else {
        preflight_check(
            "network.origins",
            ConfigPreflightStatus::Pass,
            "browser origin policy is explicit",
            format!("allowed_origin_count={}", network.allowed_origins.len()),
        )
    }
}

fn check_tunnel_provider_policy(
    network: &NetworkAccessConfig,
    env_lookup: &impl Fn(&str) -> Option<String>,
) -> ConfigPreflightCheck {
    if network.tunnel_providers.is_empty() {
        return preflight_check(
            "network.tunnel_providers",
            ConfigPreflightStatus::Pass,
            "no tunnel providers are configured",
            "built-in tunnel runtime is not enabled",
        );
    }

    for provider in &network.tunnel_providers {
        if provider.id.trim().is_empty() {
            return preflight_check(
                "network.tunnel_providers",
                ConfigPreflightStatus::Fail,
                "tunnel provider IDs must be non-empty",
                "assign a stable redaction-safe provider id",
            );
        }
        if !has_https_url(provider.public_url.as_deref()) {
            return preflight_check(
                "network.tunnel_providers",
                ConfigPreflightStatus::Fail,
                "tunnel providers require HTTPS public_url",
                format!("provider_id={}", provider.id),
            );
        }
        match provider.token_env.as_deref() {
            Some(token_env) => {
                if env_lookup(token_env)
                    .filter(|value| !value.trim().is_empty())
                    .is_none()
                {
                    return preflight_check(
                        "network.tunnel_providers",
                        ConfigPreflightStatus::Fail,
                        "tunnel provider token environment variable is missing or empty",
                        format!("provider_id={}; token_env={token_env}", provider.id),
                    );
                }
            }
            None => {
                return preflight_check(
                    "network.tunnel_providers",
                    ConfigPreflightStatus::Fail,
                    "tunnel providers require token_env",
                    format!("provider_id={}", provider.id),
                );
            }
        }
    }

    preflight_check(
        "network.tunnel_providers",
        ConfigPreflightStatus::Pass,
        "tunnel provider declarations are configured",
        format!("provider_count={}", network.tunnel_providers.len()),
    )
}

fn has_https_url(value: Option<&str>) -> bool {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some_and(|value| value.starts_with("https://"))
}

fn has_http_origin(value: Option<&str>) -> bool {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };

    !rest.is_empty()
        && !rest.contains('@')
        && !rest.contains('/')
        && !rest.contains('\\')
        && !rest.contains('?')
        && !rest.contains('#')
        && !rest.contains(',')
        && !rest.contains(';')
        && !rest.chars().any(char::is_whitespace)
}

fn check_runtime_directories(
    config: &NakoServerConfig,
    options: ConfigPreflightOptions,
) -> Vec<ConfigPreflightCheck> {
    vec![
        check_runtime_directory(
            "paths.remux_staging_root",
            &config.remux_staging_root,
            options,
        ),
        check_runtime_directory(
            "paths.artwork_artifact_root",
            &config.artwork.artifact_root,
            options,
        ),
    ]
}

fn check_runtime_directory(
    id: &str,
    path: &Path,
    options: ConfigPreflightOptions,
) -> ConfigPreflightCheck {
    if options.create_dirs {
        return match create_and_probe_directory(path) {
            Ok(()) => preflight_check(
                id,
                ConfigPreflightStatus::Pass,
                "directory exists and accepted a write probe",
                "created missing directories if needed",
            ),
            Err(err) => preflight_check(
                id,
                ConfigPreflightStatus::Fail,
                "directory create/write probe failed",
                format!("io_kind={:?}", err.kind()),
            ),
        };
    }

    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => preflight_check(
            id,
            ConfigPreflightStatus::Pass,
            "directory exists",
            "write probe skipped; rerun with --create-dirs to verify writability",
        ),
        Ok(_) => preflight_check(
            id,
            ConfigPreflightStatus::Fail,
            "configured path exists but is not a directory",
            "choose a directory outside media library roots",
        ),
        Err(err) if err.kind() == io::ErrorKind::NotFound => preflight_check(
            id,
            ConfigPreflightStatus::Warn,
            "directory does not exist yet",
            "rerun config-check with --create-dirs to create and verify it",
        ),
        Err(err) => preflight_check(
            id,
            ConfigPreflightStatus::Fail,
            "directory metadata check failed",
            format!("io_kind={:?}", err.kind()),
        ),
    }
}

fn create_and_probe_directory(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "configured path is not a directory",
        ));
    }

    let probe_path = path.join(format!(".nako-preflight-{}.tmp", std::process::id()));
    fs::write(&probe_path, b"nako-preflight")?;
    fs::remove_file(probe_path)?;
    Ok(())
}

fn check_libraries(config: &NakoServerConfig) -> Vec<ConfigPreflightCheck> {
    if config.libraries.is_empty() {
        return vec![preflight_check(
            "libraries.configured",
            ConfigPreflightStatus::Fail,
            "no media libraries are configured",
            "add at least one [[libraries]] entry before starting Nako",
        )];
    }

    let mut checks = vec![preflight_check(
        "libraries.configured",
        ConfigPreflightStatus::Pass,
        "one or more media libraries are configured",
        format!("library_count={}", config.libraries.len()),
    )];

    checks.push(check_unique_library_ids(&config.libraries));
    checks.push(check_unique_library_roots(&config.libraries));

    for library in &config.libraries {
        checks.push(check_library(library));
    }

    checks
}

fn check_unique_library_ids(libraries: &[LocalLibraryConfig]) -> ConfigPreflightCheck {
    let mut seen_ids = HashSet::new();
    let has_duplicate = libraries
        .iter()
        .any(|library| !seen_ids.insert(library.id.to_string()));

    if has_duplicate {
        preflight_check(
            "libraries.unique_ids",
            ConfigPreflightStatus::Fail,
            "configured media library IDs must be unique",
            "assign a stable unique ID to each [[libraries]] entry",
        )
    } else {
        preflight_check(
            "libraries.unique_ids",
            ConfigPreflightStatus::Pass,
            "configured media library IDs are unique",
            format!("library_count={}", libraries.len()),
        )
    }
}

fn check_unique_library_roots(libraries: &[LocalLibraryConfig]) -> ConfigPreflightCheck {
    let mut seen_roots = HashSet::new();
    let has_duplicate = libraries.iter().any(|library| {
        let root_key = library
            .webdav
            .as_ref()
            .map(|webdav| format!("webdav:{}", webdav.root))
            .unwrap_or_else(|| format!("local:{}", library.root.display()));
        !seen_roots.insert(root_key)
    });

    if has_duplicate {
        preflight_check(
            "libraries.unique_roots",
            ConfigPreflightStatus::Fail,
            "configured media library roots must be unique",
            "split libraries by distinct storage roots to avoid scan ownership ambiguity",
        )
    } else {
        preflight_check(
            "libraries.unique_roots",
            ConfigPreflightStatus::Pass,
            "configured media library roots are unique",
            format!("library_count={}", libraries.len()),
        )
    }
}

fn check_library(library: &LocalLibraryConfig) -> ConfigPreflightCheck {
    let id = format!("libraries.{}", library.id);

    if let Some(webdav) = library.webdav.as_ref() {
        if webdav.root.trim().is_empty() || !webdav.root.starts_with("webdav://") {
            return preflight_check(
                id,
                ConfigPreflightStatus::Fail,
                "WebDAV library root must use the webdav:// scheme",
                format!("library_name={}", library.name),
            );
        }
        if webdav.base_url.trim().is_empty()
            || !(webdav.base_url.starts_with("https://") || webdav.base_url.starts_with("http://"))
        {
            return preflight_check(
                id,
                ConfigPreflightStatus::Fail,
                "WebDAV library base_url must be an HTTP(S) URL",
                format!("library_name={}", library.name),
            );
        }
        return preflight_check(
            id,
            ConfigPreflightStatus::Pass,
            "WebDAV library has static connection settings",
            format!("library_name={}; network probe skipped", library.name),
        );
    }

    match fs::metadata(&library.root) {
        Ok(metadata) if metadata.is_dir() => preflight_check(
            id,
            ConfigPreflightStatus::Pass,
            "local media library root exists",
            format!("library_name={}", library.name),
        ),
        Ok(_) => preflight_check(
            id,
            ConfigPreflightStatus::Fail,
            "local media library root is not a directory",
            format!("library_name={}", library.name),
        ),
        Err(err) if err.kind() == io::ErrorKind::NotFound => preflight_check(
            id,
            ConfigPreflightStatus::Fail,
            "local media library root does not exist",
            format!(
                "library_name={}; mount or create the media library before starting Nako",
                library.name
            ),
        ),
        Err(err) => preflight_check(
            id,
            ConfigPreflightStatus::Fail,
            "local media library root metadata check failed",
            format!("library_name={}; io_kind={:?}", library.name, err.kind()),
        ),
    }
}

fn preflight_check(
    id: impl Into<String>,
    status: ConfigPreflightStatus,
    summary: impl Into<String>,
    detail: impl Into<String>,
) -> ConfigPreflightCheck {
    ConfigPreflightCheck {
        id: id.into(),
        status,
        summary: summary.into(),
        detail: detail.into(),
    }
}

fn database_url_scheme(database_url: &str) -> String {
    database_url
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.is_empty())
        .unwrap_or("unknown")
        .to_ascii_lowercase()
}

impl ConfigPreflightStatus {
    const fn rank(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Warn => 1,
            Self::Fail => 2,
        }
    }

    const fn as_label(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }

    const fn max(self, other: Self) -> Self {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}
