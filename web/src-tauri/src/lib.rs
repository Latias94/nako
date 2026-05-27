use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Default)]
struct DesktopShellState {
    server_profile: Mutex<Option<ServerProfile>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopBootstrap {
    runtime: &'static str,
    profile: Option<ServerProfile>,
    native_playback: NativePlaybackStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativePlaybackStatus {
    available: bool,
    reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerProfile {
    base_url: String,
    source: ServerProfileSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ServerProfileSource {
    Environment,
    Session,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerProfileInput {
    base_url: String,
}

impl DesktopShellState {
    fn from_environment() -> Self {
        Self {
            server_profile: Mutex::new(environment_server_profile()),
        }
    }

    fn bootstrap(&self) -> DesktopBootstrap {
        DesktopBootstrap {
            runtime: "tauri_desktop",
            profile: self
                .server_profile
                .lock()
                .expect("desktop shell state mutex poisoned")
                .clone(),
            native_playback: NativePlaybackStatus {
                available: false,
                reason: "native_playback_core_not_integrated",
            },
        }
    }

    fn save_profile(&self, base_url: String) -> DesktopBootstrap {
        let mut profile = self
            .server_profile
            .lock()
            .expect("desktop shell state mutex poisoned");
        *profile = Some(ServerProfile {
            base_url,
            source: ServerProfileSource::Session,
        });

        self.bootstrap()
    }

    fn clear_profile(&self) -> DesktopBootstrap {
        let mut profile = self
            .server_profile
            .lock()
            .expect("desktop shell state mutex poisoned");
        *profile = None;

        self.bootstrap()
    }
}

#[tauri::command]
fn desktop_bootstrap(state: tauri::State<'_, DesktopShellState>) -> DesktopBootstrap {
    state.bootstrap()
}

#[tauri::command]
fn save_server_profile(
    input: ServerProfileInput,
    state: tauri::State<'_, DesktopShellState>,
) -> Result<DesktopBootstrap, String> {
    let base_url = normalize_server_url(&input.base_url)?;
    Ok(state.save_profile(base_url))
}

#[tauri::command]
fn clear_server_profile(state: tauri::State<'_, DesktopShellState>) -> DesktopBootstrap {
    state.clear_profile()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DesktopShellState::from_environment())
        .invoke_handler(tauri::generate_handler![
            clear_server_profile,
            desktop_bootstrap,
            save_server_profile
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Nako desktop shell");
}

fn environment_server_profile() -> Option<ServerProfile> {
    let raw = std::env::var("NAKO_SERVER_URL").ok()?;
    server_profile_from_raw(&raw, ServerProfileSource::Environment)
}

fn server_profile_from_raw(raw: &str, source: ServerProfileSource) -> Option<ServerProfile> {
    let base_url = normalize_server_url(raw).ok()?;

    Some(ServerProfile { base_url, source })
}

fn normalize_server_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("server_url_required".to_string());
    }

    let parsed = Url::parse(trimmed).map_err(|_| "server_url_invalid".to_string())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("server_url_must_use_http_or_https".to_string()),
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("server_url_must_not_include_credentials".to_string());
    }

    if parsed.query().is_some() {
        return Err("server_url_must_not_include_query".to_string());
    }

    if parsed.fragment().is_some() {
        return Err("server_url_must_not_include_fragment".to_string());
    }

    Ok(parsed.to_string().trim_end_matches('/').to_string())
}

#[cfg(test)]
mod tests {
    use super::{ServerProfileSource, normalize_server_url, server_profile_from_raw};

    #[test]
    fn normalizes_http_server_urls() {
        assert_eq!(
            normalize_server_url("  http://127.0.0.1:7833/  ").unwrap(),
            "http://127.0.0.1:7833"
        );
        assert_eq!(
            normalize_server_url("https://nako.example/base/").unwrap(),
            "https://nako.example/base"
        );
    }

    #[test]
    fn rejects_unsafe_server_urls() {
        assert_eq!(normalize_server_url("").unwrap_err(), "server_url_required");
        assert_eq!(
            normalize_server_url("file:///library").unwrap_err(),
            "server_url_must_use_http_or_https"
        );
        assert_eq!(
            normalize_server_url("https://user:secret@nako.example").unwrap_err(),
            "server_url_must_not_include_credentials"
        );
        assert_eq!(
            normalize_server_url("https://nako.example?token=secret").unwrap_err(),
            "server_url_must_not_include_query"
        );
    }

    #[test]
    fn ignores_invalid_environment_profile_value() {
        assert!(
            server_profile_from_raw("file:///not-a-server", ServerProfileSource::Environment)
                .is_none()
        );
    }
}
