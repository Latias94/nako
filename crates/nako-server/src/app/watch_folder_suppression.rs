use std::sync::Arc;

use nako_core::{LibraryId, NakoError, Result};
use nako_vfs::StorageUri;
use serde::Serialize;
use tokio::sync::Mutex;

use super::current_time_ms;

const DEFAULT_PLANNED_WRITE_SUPPRESSION_TTL_MS: i64 = 5 * 60 * 1_000;
const MAX_PLANNED_WRITE_SUPPRESSION_TTL_MS: i64 = 60 * 60 * 1_000;
const MAX_SAFE_LABEL_LEN: usize = 64;

#[derive(Clone, Debug)]
pub(crate) struct WatchFolderSuppressionAppService {
    inner: Arc<Mutex<WatchFolderSuppressionState>>,
}

#[derive(Debug, Default)]
struct WatchFolderSuppressionState {
    next_token: u64,
    records: Vec<PlannedWatchFolderWriteSuppressionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedWatchFolderWriteSuppressionRecord {
    token: PlannedWatchFolderWriteSuppressionToken,
    target_library_id: LibraryId,
    scope_uri: StorageUri,
    owner: String,
    reason: String,
    expires_at_ms: i64,
    completion: PlannedWatchFolderWriteCompletion,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlannedWatchFolderWriteSuppressionToken(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlannedWatchFolderWriteCompletion {
    SuppressOnly,
    ReconcileScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BeginPlannedWatchFolderWriteSuppressionRequest {
    pub(crate) target_library_id: LibraryId,
    pub(crate) scope_uri: StorageUri,
    pub(crate) owner: String,
    pub(crate) reason: String,
    pub(crate) ttl_ms: Option<i64>,
    pub(crate) completion: PlannedWatchFolderWriteCompletion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PlannedWatchFolderWriteSuppressionDiagnostic {
    #[serde(skip_serializing)]
    pub(crate) token: PlannedWatchFolderWriteSuppressionToken,
    pub(crate) target_library_id: LibraryId,
    pub(crate) scope_scheme: String,
    pub(crate) scope_ref_redacted: String,
    pub(crate) owner: String,
    pub(crate) reason: String,
    pub(crate) expires_at_ms: i64,
    pub(crate) completion: PlannedWatchFolderWriteCompletion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompletePlannedWatchFolderWriteSuppressionDiagnostic {
    pub(crate) suppression: PlannedWatchFolderWriteSuppressionDiagnostic,
    pub(crate) reconciliation_requested: bool,
}

impl WatchFolderSuppressionAppService {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(WatchFolderSuppressionState::default())),
        }
    }

    pub(crate) async fn begin_planned_write_suppression(
        &self,
        request: BeginPlannedWatchFolderWriteSuppressionRequest,
    ) -> Result<PlannedWatchFolderWriteSuppressionDiagnostic> {
        let now_ms = current_time_ms()?;
        let ttl_ms = normalize_ttl_ms(request.ttl_ms)?;
        let owner = normalize_safe_label("planned write owner", request.owner)?;
        let reason = normalize_safe_label("planned write reason", request.reason)?;
        let expires_at_ms = now_ms
            .checked_add(ttl_ms)
            .ok_or_else(|| NakoError::InvalidInput {
                message: "planned write suppression expiry overflowed".to_owned(),
            })?;

        let mut state = self.inner.lock().await;
        state.prune_expired(now_ms);
        state.next_token =
            state
                .next_token
                .checked_add(1)
                .ok_or_else(|| NakoError::InvalidInput {
                    message: "planned write suppression token overflowed".to_owned(),
                })?;
        let record = PlannedWatchFolderWriteSuppressionRecord {
            token: PlannedWatchFolderWriteSuppressionToken(state.next_token),
            target_library_id: request.target_library_id,
            scope_uri: request.scope_uri,
            owner,
            reason,
            expires_at_ms,
            completion: request.completion,
        };
        let diagnostic = record.diagnostic();
        state.records.push(record);

        Ok(diagnostic)
    }

    pub(crate) async fn complete_planned_write_suppression(
        &self,
        token: PlannedWatchFolderWriteSuppressionToken,
    ) -> Result<Option<CompletePlannedWatchFolderWriteSuppressionDiagnostic>> {
        let now_ms = current_time_ms()?;
        let mut state = self.inner.lock().await;
        state.prune_expired(now_ms);

        let Some(index) = state
            .records
            .iter()
            .position(|record| record.token == token)
        else {
            return Ok(None);
        };
        let record = state.records.remove(index);
        let reconciliation_requested =
            record.completion == PlannedWatchFolderWriteCompletion::ReconcileScope;

        Ok(Some(CompletePlannedWatchFolderWriteSuppressionDiagnostic {
            suppression: record.diagnostic(),
            reconciliation_requested,
        }))
    }

    pub(crate) async fn match_suppression(
        &self,
        target_library_id: LibraryId,
        uri: &StorageUri,
    ) -> Result<Option<PlannedWatchFolderWriteSuppressionDiagnostic>> {
        let now_ms = current_time_ms()?;
        let mut state = self.inner.lock().await;
        state.prune_expired(now_ms);

        Ok(state
            .records
            .iter()
            .find(|record| {
                record.target_library_id == target_library_id
                    && scope_contains_uri(&record.scope_uri, uri)
            })
            .map(PlannedWatchFolderWriteSuppressionRecord::diagnostic))
    }

    pub(crate) async fn list_active_for_library(
        &self,
        target_library_id: LibraryId,
    ) -> Result<Vec<PlannedWatchFolderWriteSuppressionDiagnostic>> {
        let now_ms = current_time_ms()?;
        let mut state = self.inner.lock().await;
        state.prune_expired(now_ms);

        Ok(state
            .records
            .iter()
            .filter(|record| record.target_library_id == target_library_id)
            .map(PlannedWatchFolderWriteSuppressionRecord::diagnostic)
            .collect())
    }
}

impl Default for WatchFolderSuppressionAppService {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchFolderSuppressionState {
    fn prune_expired(&mut self, now_ms: i64) {
        self.records.retain(|record| record.expires_at_ms > now_ms);
    }
}

impl PlannedWatchFolderWriteSuppressionRecord {
    fn diagnostic(&self) -> PlannedWatchFolderWriteSuppressionDiagnostic {
        PlannedWatchFolderWriteSuppressionDiagnostic {
            token: self.token,
            target_library_id: self.target_library_id,
            scope_scheme: self.scope_uri.scheme().to_owned(),
            scope_ref_redacted: redact_storage_uri(&self.scope_uri),
            owner: self.owner.clone(),
            reason: self.reason.clone(),
            expires_at_ms: self.expires_at_ms,
            completion: self.completion,
        }
    }
}

fn normalize_ttl_ms(ttl_ms: Option<i64>) -> Result<i64> {
    let ttl_ms = ttl_ms.unwrap_or(DEFAULT_PLANNED_WRITE_SUPPRESSION_TTL_MS);
    if ttl_ms <= 0 {
        return Err(NakoError::InvalidInput {
            message: "planned write suppression ttl_ms must be greater than zero".to_owned(),
        });
    }
    if ttl_ms > MAX_PLANNED_WRITE_SUPPRESSION_TTL_MS {
        return Err(NakoError::InvalidInput {
            message: format!(
                "planned write suppression ttl_ms must be at most {MAX_PLANNED_WRITE_SUPPRESSION_TTL_MS}"
            ),
        });
    }

    Ok(ttl_ms)
}

fn normalize_safe_label(label: &str, value: String) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("{label} cannot be empty"),
        });
    }
    if value.len() > MAX_SAFE_LABEL_LEN {
        return Err(NakoError::InvalidInput {
            message: format!("{label} cannot exceed {MAX_SAFE_LABEL_LEN} characters"),
        });
    }
    if !value
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.'))
    {
        return Err(NakoError::InvalidInput {
            message: format!("{label} must be a safe identifier"),
        });
    }

    Ok(value.to_owned())
}

fn scope_contains_uri(scope: &StorageUri, uri: &StorageUri) -> bool {
    if scope.scheme() != uri.scheme() {
        return false;
    }

    let scope_path = normalized_storage_path(scope);
    if scope_path.is_empty() {
        return true;
    }

    let uri_path = normalized_storage_path(uri);
    uri_path == scope_path
        || uri_path
            .strip_prefix(scope_path.as_str())
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn normalized_storage_path(uri: &StorageUri) -> String {
    uri.path_part()
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn redact_storage_uri(uri: &StorageUri) -> String {
    format!("{}://<redacted>", uri.scheme())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn planned_write_suppression_matches_same_scope_and_descendants() {
        let service = WatchFolderSuppressionAppService::new();
        let library_id = LibraryId::new();
        let diagnostic = service
            .begin_planned_write_suppression(BeginPlannedWatchFolderWriteSuppressionRequest {
                target_library_id: library_id,
                scope_uri: StorageUri::from_parts("local", "Movies").unwrap(),
                owner: "nfo".to_owned(),
                reason: "sidecar_write".to_owned(),
                ttl_ms: Some(60_000),
                completion: PlannedWatchFolderWriteCompletion::SuppressOnly,
            })
            .await
            .unwrap();

        assert_eq!(diagnostic.scope_ref_redacted, "local://<redacted>");
        assert!(
            service
                .match_suppression(
                    library_id,
                    &StorageUri::from_parts("local", "Movies").unwrap()
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            service
                .match_suppression(
                    library_id,
                    &StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap(),
                )
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            service
                .match_suppression(
                    library_id,
                    &StorageUri::from_parts("local", "Movies2/Demo.mkv").unwrap(),
                )
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn planned_write_completion_removes_suppression_and_reports_reconciliation_intent() {
        let service = WatchFolderSuppressionAppService::new();
        let library_id = LibraryId::new();
        let diagnostic = service
            .begin_planned_write_suppression(BeginPlannedWatchFolderWriteSuppressionRequest {
                target_library_id: library_id,
                scope_uri: StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap(),
                owner: "managed_import".to_owned(),
                reason: "library_write".to_owned(),
                ttl_ms: Some(60_000),
                completion: PlannedWatchFolderWriteCompletion::ReconcileScope,
            })
            .await
            .unwrap();
        let completed = service
            .complete_planned_write_suppression(diagnostic.token)
            .await
            .unwrap()
            .unwrap();

        assert!(completed.reconciliation_requested);
        assert!(
            service
                .match_suppression(
                    library_id,
                    &StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap(),
                )
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn planned_write_suppression_rejects_unsafe_labels_and_ttl() {
        let service = WatchFolderSuppressionAppService::new();
        let library_id = LibraryId::new();
        let scope_uri = StorageUri::from_parts("local", "Movies/Demo.mkv").unwrap();

        for (owner, reason, ttl_ms, expected) in [
            (
                "",
                "sidecar_write",
                Some(60_000),
                "planned write owner cannot be empty",
            ),
            (
                "managed import",
                "sidecar_write",
                Some(60_000),
                "planned write owner must be a safe identifier",
            ),
            (
                "managed_import",
                "wrote local:///Movies/Demo.mkv",
                Some(60_000),
                "planned write reason must be a safe identifier",
            ),
            (
                "managed_import",
                "sidecar_write",
                Some(0),
                "planned write suppression ttl_ms must be greater than zero",
            ),
            (
                "managed_import",
                "sidecar_write",
                Some(MAX_PLANNED_WRITE_SUPPRESSION_TTL_MS + 1),
                "planned write suppression ttl_ms must be at most 3600000",
            ),
        ] {
            let err = service
                .begin_planned_write_suppression(BeginPlannedWatchFolderWriteSuppressionRequest {
                    target_library_id: library_id,
                    scope_uri: scope_uri.clone(),
                    owner: owner.to_owned(),
                    reason: reason.to_owned(),
                    ttl_ms,
                    completion: PlannedWatchFolderWriteCompletion::SuppressOnly,
                })
                .await
                .unwrap_err();

            assert_eq!(err.to_string(), format!("invalid input: {expected}"));
            let body = err.to_string();
            assert!(!body.contains("local:///"));
            assert!(!body.contains("Demo.mkv"));
        }
    }
}
