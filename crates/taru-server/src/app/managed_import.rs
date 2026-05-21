use serde::Serialize;
use taru_core::{
    LibraryId, LibraryRepository, ManagedImportArtifactId, ManagedImportArtifactListFilter,
    ManagedImportArtifactRecord, ManagedImportArtifactState, ManagedImportRepository,
    ManagedImportSourceKind, NewManagedImportArtifact, PageRequest, Result, StagingManifestId,
    StagingManifestRepository, TaruError,
};
use taru_db::TaruDatabase;

#[derive(Clone, Debug)]
pub(crate) struct ManagedImportAppService {
    store: TaruDatabase,
}

impl ManagedImportAppService {
    pub(crate) fn new(store: TaruDatabase) -> Self {
        Self { store }
    }

    pub(crate) async fn create_artifact(
        &self,
        request: CreateManagedImportArtifactRequest,
    ) -> Result<ManagedImportArtifactDiagnostic> {
        self.store
            .get_library(request.target_library_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "library",
                id: request.target_library_id.to_string(),
            })?;

        let source_uri = require_non_empty("managed import source_uri", request.source_uri)?;
        let state = request.state.unwrap_or(match request.staging_manifest_id {
            Some(_) => ManagedImportArtifactState::Staged,
            None => ManagedImportArtifactState::Proposed,
        });
        validate_create_state(state)?;

        let staging_manifest = match request.staging_manifest_id {
            Some(id) => Some(
                self.store
                    .get_staging_manifest_record(id)
                    .await?
                    .ok_or_else(|| TaruError::NotFound {
                        entity: "staging_manifest_record",
                        id: id.to_string(),
                    })?,
            ),
            None => None,
        };
        let now_ms = super::current_time_ms()?;
        let artifact = NewManagedImportArtifact {
            id: request.id.unwrap_or_else(ManagedImportArtifactId::new),
            target_library_id: request.target_library_id,
            source_kind: request.source_kind,
            source_uri,
            staging_manifest_id: request.staging_manifest_id,
            artifact_uri: optional_non_empty(request.artifact_uri),
            original_file_name: optional_non_empty(request.original_file_name),
            intended_locator: optional_non_empty(request.intended_locator),
            size_bytes: request.size_bytes.or_else(|| {
                staging_manifest
                    .as_ref()
                    .and_then(|record| record.size_bytes)
            }),
            fingerprint: request
                .fingerprint
                .or_else(|| staging_manifest.and_then(|record| record.fingerprint)),
            state,
            diagnostics_json: optional_non_empty(request.diagnostics_json),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };

        let record = self.store.upsert_managed_import_artifact(artifact).await?;

        Ok(ManagedImportArtifactDiagnostic::from_record(record))
    }

    pub(crate) async fn list_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<ManagedImportArtifactDiagnostics> {
        let page = page.clamped();
        if let Some(library_id) = filter.target_library_id {
            self.store
                .get_library(library_id)
                .await?
                .ok_or_else(|| TaruError::NotFound {
                    entity: "library",
                    id: library_id.to_string(),
                })?;
        }

        let records = self
            .store
            .list_managed_import_artifacts(filter, page)
            .await?;
        let returned = records.len();
        let artifacts = records
            .into_iter()
            .map(ManagedImportArtifactDiagnostic::from_record)
            .collect::<Vec<_>>();

        Ok(ManagedImportArtifactDiagnostics {
            limit: page.limit,
            offset: page.offset,
            returned,
            artifacts,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CreateManagedImportArtifactRequest {
    pub(crate) id: Option<ManagedImportArtifactId>,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: ManagedImportSourceKind,
    pub(crate) source_uri: String,
    pub(crate) staging_manifest_id: Option<StagingManifestId>,
    pub(crate) artifact_uri: Option<String>,
    pub(crate) original_file_name: Option<String>,
    pub(crate) intended_locator: Option<String>,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) fingerprint: Option<String>,
    pub(crate) state: Option<ManagedImportArtifactState>,
    pub(crate) diagnostics_json: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportArtifactDiagnostics {
    pub(crate) limit: u32,
    pub(crate) offset: u64,
    pub(crate) returned: usize,
    pub(crate) artifacts: Vec<ManagedImportArtifactDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportArtifactDiagnostic {
    pub(crate) id: ManagedImportArtifactId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) source_kind: String,
    pub(crate) custom_source_kind: bool,
    pub(crate) source_scheme: Option<String>,
    pub(crate) source_uri_redacted: String,
    pub(crate) staging_manifest_id: Option<StagingManifestId>,
    pub(crate) has_artifact_uri: bool,
    pub(crate) has_original_file_name: bool,
    pub(crate) has_intended_locator: bool,
    pub(crate) size_bytes: Option<u64>,
    pub(crate) has_fingerprint: bool,
    pub(crate) state: ManagedImportArtifactState,
    pub(crate) has_diagnostics: bool,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

impl ManagedImportArtifactDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: ManagedImportArtifactRecord) -> Self {
        let (source_kind, source_kind_key) = record.source_kind.as_parts();
        Self {
            id: record.id,
            target_library_id: record.target_library_id,
            source_kind: source_kind.to_owned(),
            custom_source_kind: !source_kind_key.is_empty(),
            source_scheme: uri_scheme(&record.source_uri).map(str::to_owned),
            source_uri_redacted: redact_uri(&record.source_uri),
            staging_manifest_id: record.staging_manifest_id,
            has_artifact_uri: record.artifact_uri.is_some(),
            has_original_file_name: record.original_file_name.is_some(),
            has_intended_locator: record.intended_locator.is_some(),
            size_bytes: record.size_bytes,
            has_fingerprint: record.fingerprint.is_some(),
            state: record.state,
            has_diagnostics: record.diagnostics_json.is_some(),
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

fn validate_create_state(state: ManagedImportArtifactState) -> Result<()> {
    if matches!(
        state,
        ManagedImportArtifactState::Accepted
            | ManagedImportArtifactState::Applying
            | ManagedImportArtifactState::Promoted
            | ManagedImportArtifactState::CleanupPending
            | ManagedImportArtifactState::Cleaned
    ) {
        return Err(TaruError::InvalidInput {
            message: format!(
                "managed import creation cannot start in mutating lifecycle state: {}",
                state.as_str()
            ),
        });
    }

    Ok(())
}

fn require_non_empty(label: &str, value: String) -> Result<String> {
    optional_non_empty(Some(value)).ok_or_else(|| TaruError::InvalidInput {
        message: format!("{label} cannot be empty"),
    })
}

fn optional_non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn uri_scheme(value: &str) -> Option<&str> {
    value
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.is_empty())
}

fn redact_uri(value: &str) -> String {
    uri_scheme(value)
        .map(|scheme| format!("{scheme}://<redacted>"))
        .unwrap_or_else(|| "<redacted>".to_owned())
}
