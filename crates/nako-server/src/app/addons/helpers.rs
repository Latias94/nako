use std::{collections::HashMap, env};

use nako_addon_protocol::AddonScope;
use nako_core::{AddonRegistrationRecord, AddonStatus, NakoError, Result, SecretString};
use sha2::{Digest, Sha256};

pub(crate) fn optional_non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

pub(crate) fn uri_scheme(value: &str) -> Option<&str> {
    value
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.is_empty())
}

pub(crate) fn redact_uri(value: &str) -> String {
    uri_scheme(value)
        .map(|scheme| format!("{scheme}://<redacted>"))
        .unwrap_or_else(|| "<redacted>".to_owned())
}

pub(crate) fn fingerprint_key(value: &str) -> String {
    let digest = sha256_hex(value);
    format!("sha256:{}", &digest[..32])
}

pub(crate) fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn addon_surface_url(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

pub(crate) fn stored_granted_scopes(addon: &AddonRegistrationRecord) -> Result<Vec<AddonScope>> {
    addon
        .granted_scopes
        .iter()
        .map(|scope| {
            serde_json::from_value::<AddonScope>(serde_json::Value::String(scope.clone())).map_err(
                |err| NakoError::InvalidInput {
                    message: format!("invalid stored addon scope `{scope}`: {err}"),
                },
            )
        })
        .collect()
}

pub(crate) fn normalize_optional_secret_env(
    value: Option<String>,
    field_name: &'static str,
) -> Result<Option<String>> {
    match value {
        Some(value) => {
            let value = value.trim().to_owned();
            if value.is_empty() {
                return Err(NakoError::InvalidInput {
                    message: format!("{field_name} cannot be empty"),
                });
            }
            if !is_valid_environment_name(&value) {
                return Err(NakoError::InvalidInput {
                    message: format!("{field_name} must be a valid environment variable name"),
                });
            }

            Ok(Some(value))
        }
        None => Ok(None),
    }
}

pub(crate) fn ensure_addon_accepts_runtime_authority(
    addon: &AddonRegistrationRecord,
    operation: &'static str,
) -> Result<()> {
    if addon.status == AddonStatus::Unregistered {
        return Err(NakoError::Conflict {
            message: format!(
                "cannot {operation} for unregistered addon registration {}",
                addon.id
            ),
        });
    }

    Ok(())
}

pub(crate) fn resolve_outbound_task_dispatch_secret(
    addon: &AddonRegistrationRecord,
) -> Result<Option<SecretString>> {
    resolve_outbound_task_dispatch_secret_with(addon, resolve_outbound_task_dispatch_secret_env)
}

fn resolve_outbound_task_dispatch_secret_env(
    secret_env: &str,
) -> std::result::Result<String, String> {
    #[cfg(test)]
    if let Some(secret) = test_outbound_task_dispatch_secret(secret_env) {
        return Ok(secret);
    }

    env::var(secret_env).map_err(|err| err.to_string())
}

pub(crate) fn resolve_outbound_task_dispatch_secret_with<F, E>(
    addon: &AddonRegistrationRecord,
    resolve_env: F,
) -> Result<Option<SecretString>>
where
    F: FnOnce(&str) -> std::result::Result<String, E>,
    E: std::fmt::Display,
{
    let Some(secret_env) = addon.outbound_task_dispatch_secret_env.as_deref() else {
        return Ok(None);
    };
    let secret = resolve_env(secret_env).map_err(|err| NakoError::InvalidInput {
        message: format!(
            "addon {} references unavailable outbound task-dispatch secret environment variable {secret_env}: {err}",
            addon.id
        ),
    })?;
    if secret.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!(
                "addon {} outbound task-dispatch secret environment variable {secret_env} is empty",
                addon.id
            ),
        });
    }

    Ok(Some(SecretString::new(secret)))
}

fn is_valid_environment_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[cfg(test)]
static TEST_OUTBOUND_TASK_DISPATCH_SECRETS: std::sync::OnceLock<
    std::sync::Mutex<HashMap<String, String>>,
> = std::sync::OnceLock::new();

#[cfg(test)]
pub(crate) struct TestOutboundTaskDispatchSecretGuard {
    name: String,
}

#[cfg(test)]
pub(crate) fn set_test_outbound_task_dispatch_secret(
    name: &str,
    value: &str,
) -> TestOutboundTaskDispatchSecretGuard {
    let secrets =
        TEST_OUTBOUND_TASK_DISPATCH_SECRETS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    secrets
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(name.to_owned(), value.to_owned());
    TestOutboundTaskDispatchSecretGuard {
        name: name.to_owned(),
    }
}

#[cfg(test)]
impl Drop for TestOutboundTaskDispatchSecretGuard {
    fn drop(&mut self) {
        if let Some(secrets) = TEST_OUTBOUND_TASK_DISPATCH_SECRETS.get() {
            secrets
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&self.name);
        }
    }
}

#[cfg(test)]
fn test_outbound_task_dispatch_secret(name: &str) -> Option<String> {
    TEST_OUTBOUND_TASK_DISPATCH_SECRETS
        .get()
        .and_then(|secrets| {
            secrets
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(name)
                .cloned()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nako_core::{AddonId, AddonStatus};

    fn addon_registration(
        outbound_task_dispatch_secret_env: Option<&str>,
    ) -> AddonRegistrationRecord {
        AddonRegistrationRecord {
            id: AddonId::new(),
            manifest_id: "example.metadata".to_owned(),
            name: "Example Metadata".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: "{}".to_owned(),
            outbound_task_dispatch_secret_env: outbound_task_dispatch_secret_env.map(str::to_owned),
            granted_scopes: Vec::new(),
            status: AddonStatus::Enabled,
            created_at: "2026-05-24T00:00:00.000Z".to_owned(),
            updated_at: "2026-05-24T00:00:00.000Z".to_owned(),
        }
    }

    #[test]
    fn resolves_outbound_task_dispatch_secret_from_env_reference() {
        let addon = addon_registration(Some("NAKO_ADDON_DISPATCH_SECRET"));
        let resolved = resolve_outbound_task_dispatch_secret_with(&addon, |name| match name {
            "NAKO_ADDON_DISPATCH_SECRET" => Ok("super-secret".to_owned()),
            other => Err(format!("missing {other}")),
        })
        .unwrap();

        let secret = resolved.expect("expected resolved outbound secret");
        assert_eq!(secret.expose_secret(), "super-secret");
    }

    #[test]
    fn missing_outbound_task_dispatch_secret_reports_safe_error() {
        let addon = addon_registration(Some("NAKO_ADDON_DISPATCH_SECRET"));
        let err = resolve_outbound_task_dispatch_secret_with(&addon, |_name| {
            Err(std::env::VarError::NotPresent)
        })
        .unwrap_err();
        let text = err.to_string();

        assert!(text.contains("NAKO_ADDON_DISPATCH_SECRET"));
        assert!(!text.contains("super-secret"));
    }

    #[test]
    fn missing_reference_returns_none() {
        let addon = addon_registration(None);

        assert_eq!(
            resolve_outbound_task_dispatch_secret_with(&addon, |_name| {
                Ok::<String, String>("unused".to_owned())
            })
            .unwrap(),
            None
        );
    }

    #[test]
    fn rejects_invalid_outbound_task_dispatch_secret_env_names() {
        let err = normalize_optional_secret_env(
            Some("not-a-valid env".to_owned()),
            "outbound_task_dispatch_secret_env",
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("must be a valid environment variable name")
        );
    }
}
