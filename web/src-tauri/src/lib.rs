use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tauri::Manager;
use url::Url;

#[derive(Debug, Default)]
struct DesktopShellState {
    server_profile: Mutex<Option<ServerProfile>>,
    profile_path: Option<PathBuf>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerProfile {
    base_url: String,
    source: ServerProfileSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ServerProfileSource {
    Environment,
    LocalProfile,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerProfileInput {
    base_url: String,
}

impl DesktopShellState {
    fn from_profile_path(profile_path: PathBuf) -> Self {
        Self {
            server_profile: Mutex::new(
                read_server_profile(&profile_path).or_else(environment_server_profile),
            ),
            profile_path: Some(profile_path),
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

    fn save_profile(&self, base_url: String) -> Result<DesktopBootstrap, String> {
        let next_profile = ServerProfile {
            base_url,
            source: ServerProfileSource::LocalProfile,
        };

        if let Some(profile_path) = &self.profile_path {
            write_server_profile(profile_path, &next_profile)?;
        }

        {
            let mut profile = self
                .server_profile
                .lock()
                .expect("desktop shell state mutex poisoned");
            *profile = Some(next_profile);
        }

        Ok(self.bootstrap())
    }

    fn clear_profile(&self) -> Result<DesktopBootstrap, String> {
        {
            let mut profile = self
                .server_profile
                .lock()
                .expect("desktop shell state mutex poisoned");
            *profile = None;
        }

        if let Some(profile_path) = &self.profile_path
            && profile_path.exists()
        {
            fs::remove_file(profile_path).map_err(|error| error.to_string())?;
        }

        Ok(self.bootstrap())
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
    state.save_profile(base_url)
}

#[tauri::command]
fn clear_server_profile(
    state: tauri::State<'_, DesktopShellState>,
) -> Result<DesktopBootstrap, String> {
    state.clear_profile()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let profile_path = app.path().app_config_dir()?.join("server-profile.json");
            app.manage(DesktopShellState::from_profile_path(profile_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            clear_server_profile,
            desktop_bootstrap,
            save_server_profile
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Nako desktop shell");
}

fn read_server_profile(profile_path: &Path) -> Option<ServerProfile> {
    let raw = fs::read_to_string(profile_path).ok()?;
    let profile = serde_json::from_str::<ServerProfile>(&raw).ok()?;
    server_profile_from_raw(&profile.base_url, ServerProfileSource::LocalProfile)
}

fn write_server_profile(profile_path: &Path, profile: &ServerProfile) -> Result<(), String> {
    if let Some(parent) = profile_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let raw = serde_json::to_string_pretty(profile).map_err(|error| error.to_string())?;
    fs::write(profile_path, raw).map_err(|error| error.to_string())
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
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        DesktopShellState, ServerProfileSource, normalize_server_url, server_profile_from_raw,
    };

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

    #[test]
    fn persists_local_profile_without_secrets() {
        let profile_path = temp_profile_path();
        let state = DesktopShellState::from_profile_path(profile_path.clone());

        state
            .save_profile("http://127.0.0.1:8096".to_string())
            .unwrap();

        let raw = fs::read_to_string(&profile_path).unwrap();
        assert!(raw.contains("http://127.0.0.1:8096"));
        assert!(!raw.contains("secret"));

        let restored = DesktopShellState::from_profile_path(profile_path.clone()).bootstrap();
        assert_eq!(
            restored.profile.unwrap().source,
            ServerProfileSource::LocalProfile
        );

        state.clear_profile().unwrap();
        assert!(!profile_path.exists());
    }

    fn temp_profile_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nako-desktop-profile-{nonce}.json"))
    }
}
