use nako_client_protocol::PageInfo;
use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateState, LibraryId,
    ManagedImportArtifactId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAcquisitionIntakeCandidateListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub candidates: Vec<AdminAcquisitionIntakeCandidateDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAcquisitionIntakeCandidateDiagnostic {
    pub id: AcquisitionIntakeCandidateId,
    pub target_library_id: LibraryId,
    pub source_kind: String,
    pub custom_source_kind: bool,
    pub source_scheme: Option<String>,
    pub source_ref_redacted: String,
    pub source_key_fingerprint: String,
    pub has_display_name: bool,
    pub has_intended_locator: bool,
    pub size_bytes: Option<u64>,
    pub has_fingerprint: bool,
    pub managed_import_artifact_id: Option<ManagedImportArtifactId>,
    pub state: AcquisitionIntakeCandidateState,
    pub has_diagnostics: bool,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryRequest {
    pub target_library_id: LibraryId,
    pub root_uri: Option<String>,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub target_library_id: LibraryId,
    pub root_scheme: Option<String>,
    pub root_ref_redacted: String,
    pub ready_candidates: u64,
    pub inspecting_candidates: u64,
    pub blocked_candidates: u64,
    pub incomplete_candidates: u64,
    pub unsupported_candidates: u64,
    pub suppressed_candidates: u64,
    pub recorded_candidates: u64,
    pub newly_ready_candidates: u64,
    pub active_suppressions: Vec<AdminWatchFolderSuppression>,
    pub failures: Vec<AdminWatchFolderDiscoveryFailure>,
    pub writes_library: bool,
    pub managed_import_artifacts_created: bool,
    pub promotion_apply: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderSuppression {
    pub target_library_id: LibraryId,
    pub scope_scheme: String,
    pub scope_ref_redacted: String,
    pub owner: String,
    pub reason: String,
    pub expires_at_ms: i64,
    pub completion: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderDiscoveryFailure {
    pub ref_redacted: String,
    pub safe_message: String,
}

#[cfg(test)]
mod tests {
    use crate::{admin::ADMIN_API_VERSION, public_client::API_VERSION};

    use super::*;

    #[test]
    fn acquisition_intake_diagnostics_use_redacted_refs_not_raw_sources() {
        let diagnostic = AdminAcquisitionIntakeCandidateDiagnostic {
            id: AcquisitionIntakeCandidateId::new(),
            target_library_id: LibraryId::new(),
            source_kind: "watch_folder".to_owned(),
            custom_source_kind: false,
            source_scheme: Some("local".to_owned()),
            source_ref_redacted: "local://<redacted>".to_owned(),
            source_key_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
            has_display_name: true,
            has_intended_locator: true,
            size_bytes: Some(42),
            has_fingerprint: true,
            managed_import_artifact_id: Some(ManagedImportArtifactId::new()),
            state: AcquisitionIntakeCandidateState::Ready,
            has_diagnostics: true,
            first_seen_at_ms: 1_000,
            last_seen_at_ms: 1_100,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
        };
        let response = AdminAcquisitionIntakeCandidateListResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            candidates: vec![diagnostic],
            page: PageInfo {
                limit: 10,
                offset: 0,
                returned: 1,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["candidates"][0]["source_kind"], "watch_folder");
        assert_eq!(value["candidates"][0]["source_scheme"], "local");
        assert_eq!(
            value["candidates"][0]["source_ref_redacted"],
            "local://<redacted>"
        );
        assert_eq!(value["candidates"][0]["state"], "ready");
        assert_eq!(value["page"]["returned"], 1);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("\"intended_locator\""));
        assert!(!body.contains("\"display_name\""));
        assert!(!body.contains("diagnostics_json"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("token"));
        assert!(!body.contains("private"));
    }

    #[test]
    fn watch_folder_discovery_response_redacts_root_and_failures() {
        let response = AdminWatchFolderDiscoveryResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            target_library_id: LibraryId::new(),
            root_scheme: Some("local".to_owned()),
            root_ref_redacted: "local://<redacted>".to_owned(),
            ready_candidates: 2,
            inspecting_candidates: 0,
            blocked_candidates: 1,
            incomplete_candidates: 1,
            unsupported_candidates: 0,
            suppressed_candidates: 1,
            recorded_candidates: 3,
            newly_ready_candidates: 2,
            active_suppressions: vec![AdminWatchFolderSuppression {
                target_library_id: LibraryId::new(),
                scope_scheme: "local".to_owned(),
                scope_ref_redacted: "local://<redacted>".to_owned(),
                owner: "nfo".to_owned(),
                reason: "sidecar_write".to_owned(),
                expires_at_ms: 1_000,
                completion: "suppress_only".to_owned(),
            }],
            failures: vec![AdminWatchFolderDiscoveryFailure {
                ref_redacted: "local://<redacted>".to_owned(),
                safe_message: "storage error: NotFound".to_owned(),
            }],
            writes_library: false,
            managed_import_artifacts_created: false,
            promotion_apply: false,
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["root_ref_redacted"], "local://<redacted>");
        assert_eq!(value["ready_candidates"], 2);
        assert_eq!(value["inspecting_candidates"], 0);
        assert_eq!(value["newly_ready_candidates"], 2);
        assert_eq!(value["suppressed_candidates"], 1);
        assert_eq!(
            value["active_suppressions"][0]["scope_ref_redacted"],
            "local://<redacted>"
        );
        assert_eq!(value["active_suppressions"][0]["owner"], "nfo");
        assert_eq!(value["active_suppressions"][0]["reason"], "sidecar_write");
        assert_eq!(
            value["active_suppressions"][0]["completion"],
            "suppress_only"
        );
        assert_eq!(value["failures"][0]["ref_redacted"], "local://<redacted>");
        assert_eq!(value["writes_library"], false);
        assert_eq!(value["managed_import_artifacts_created"], false);
        assert_eq!(value["promotion_apply"], false);
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("uri_redacted"));
        assert!(!body.contains("scope_uri"));
        assert!(!body.contains("token"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("token"));
        assert!(!body.contains("C:\\"));
    }
}
