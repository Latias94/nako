use nako_core::{
    LibraryId, ManagedImportArtifactId, ManagedImportArtifactRecord, ManagedImportArtifactState,
    ManagedImportPromotionApplyId, ManagedImportPromotionApplyRecord,
    ManagedImportPromotionApplyState, ManagedImportPromotionOperationKind, StagingManifestId,
    UserPrincipalId,
};
use serde::Serialize;

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ManagedImportPromotionAcceptanceDiagnostic {
    pub(crate) id: ManagedImportPromotionApplyId,
    pub(crate) artifact_id: ManagedImportArtifactId,
    pub(crate) target_library_id: LibraryId,
    pub(crate) requested_by: UserPrincipalId,
    pub(crate) operation_kind: ManagedImportPromotionOperationKind,
    pub(crate) source_scheme: Option<String>,
    pub(crate) destination_locator: Option<String>,
    pub(crate) state: ManagedImportPromotionApplyState,
    pub(crate) replayed: bool,
    pub(crate) accepted_plan_snapshot: bool,
    pub(crate) accepted_warnings_snapshot: bool,
    pub(crate) has_outcome: bool,
    pub(crate) safe_error_code: Option<String>,
    pub(crate) safe_message: Option<String>,
    pub(crate) has_raw_source_uri: bool,
    pub(crate) has_raw_fingerprint: bool,
    pub(crate) created_at_ms: i64,
    pub(crate) updated_at_ms: i64,
}

impl ManagedImportPromotionAcceptanceDiagnostic {
    #[must_use]
    pub(crate) fn from_record(record: ManagedImportPromotionApplyRecord, replayed: bool) -> Self {
        Self {
            id: record.id,
            artifact_id: record.artifact_id,
            target_library_id: record.target_library_id,
            requested_by: record.requested_by,
            operation_kind: record.operation_kind,
            source_scheme: record
                .source_artifact_uri
                .as_deref()
                .and_then(uri_scheme)
                .map(str::to_owned),
            destination_locator: Some(record.destination_locator),
            state: record.state,
            replayed,
            accepted_plan_snapshot: !record.accepted_plan_json.trim().is_empty(),
            accepted_warnings_snapshot: record.accepted_warnings_json.is_some(),
            has_outcome: record.outcome_json.is_some(),
            safe_error_code: record.safe_error_code,
            safe_message: record.safe_message,
            has_raw_source_uri: false,
            has_raw_fingerprint: false,
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
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
