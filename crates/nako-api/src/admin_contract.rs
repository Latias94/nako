use crate::admin::ADMIN_API_VERSION;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminContractRoute {
    pub key: &'static str,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminContractRouteExclusion {
    pub path: String,
    pub reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AdminRouteExclusionSuffix {
    suffix: &'static str,
    reason: &'static str,
}

const ADMIN_ROUTE_SUFFIXES: [(&str, &str); 89] = [
    ("overview", "overview"),
    ("accessSummary", "access/summary"),
    ("accessUsers", "access/users"),
    (
        "accessUserLocalPassword",
        "access/users/{user_id}/local-password",
    ),
    ("accessUserRoles", "access/users/{user_id}/roles"),
    ("accessUserStatus", "access/users/{user_id}/status"),
    ("accessLibraryPolicies", "access/library-policies"),
    ("addons", "addons"),
    ("addonCatalogSources", "addons/catalog/sources"),
    ("addonCatalogEntries", "addons/catalog/entries"),
    (
        "addonCatalogResolve",
        "addons/catalog/entries/{entry_id}/resolve",
    ),
    ("addonDetail", "addons/:addon_id"),
    ("addonStatus", "addons/:addon_id/status"),
    ("addonUnregister", "addons/:addon_id/unregister"),
    ("addonHealthCheck", "addons/:addon_id/health-check"),
    ("addonSurfaces", "addons/:addon_id/surfaces"),
    ("addonInstallGuide", "addons/:addon_id/install-guide"),
    ("addonManagerPlan", "addons/:addon_id/manager-plan"),
    (
        "addonResourceCallDiagnostic",
        "addons/:addon_id/diagnostics/resource-call",
    ),
    (
        "addonResourceSearchDiagnostic",
        "addons/:addon_id/diagnostics/resource-search",
    ),
    ("addonResourceSearch", "addons/:addon_id/resource-search"),
    (
        "addonResourceSearchSelection",
        "addons/:addon_id/resource-search/{search_id}/selections/{selection_id}/intake-candidate",
    ),
    (
        "addonResourceSearchSelectionLinkCheck",
        "addons/:addon_id/resource-search/{search_id}/selections/{selection_id}/link-check",
    ),
    ("addonSubtitleSearch", "addons/:addon_id/subtitle-search"),
    (
        "addonSubtitleSearchSelection",
        "addons/:addon_id/subtitle-search/{search_id}/selections/{selection_id}/selected-reference",
    ),
    (
        "addonSubtitleImportPlan",
        "addons/:addon_id/subtitle-search/{search_id}/selections/{selection_id}/import-plan",
    ),
    (
        "addonSubtitleImportApply",
        "addons/:addon_id/subtitle-search/{search_id}/selections/{selection_id}/import-apply",
    ),
    (
        "acquisitionIntakeCandidates",
        "acquisition/intake/candidates",
    ),
    (
        "acquisitionIntakeWatchFolderDiscovery",
        "acquisition/intake/watch-folder-discovery",
    ),
    (
        "generatedArtifactProposals",
        "automation/generated-artifacts/proposals",
    ),
    (
        "generatedArtifactApplyOutcomes",
        "automation/generated-artifact-apply-outcomes",
    ),
    (
        "generatedArtifactApplyOutcome",
        "automation/generated-artifact-apply-outcomes/{outcome_id}",
    ),
    (
        "generatedArtifactApplyRecovery",
        "automation/generated-artifact-apply-recovery",
    ),
    (
        "generatedArtifactReviewPlan",
        "automation/generated-artifacts/{artifact_id}/review-plan",
    ),
    (
        "generatedArtifactReview",
        "automation/generated-artifacts/{artifact_id}/review",
    ),
    (
        "generatedArtifactMetadataBulkApplyPlan",
        "automation/generated-artifacts/metadata-apply-plan",
    ),
    (
        "generatedArtifactMetadataBulkApplyBatches",
        "automation/generated-artifacts/metadata-apply-batches",
    ),
    (
        "generatedArtifactMetadataBulkApplyBatch",
        "automation/generated-artifacts/metadata-apply-batches/{batch_id}",
    ),
    (
        "generatedArtifactMetadataApplyPlan",
        "automation/generated-artifacts/{artifact_id}/metadata-apply-plan",
    ),
    (
        "generatedArtifactMetadataApply",
        "automation/generated-artifacts/{artifact_id}/metadata-apply",
    ),
    ("itemArtworkGallery", "items/{item_id}/artwork"),
    ("itemArtworkSelect", "items/{item_id}/artwork/{kind}/select"),
    (
        "itemArtworkSelection",
        "items/{item_id}/artwork/{kind}/selection",
    ),
    ("catalogGovernanceItems", "catalog/governance/items"),
    (
        "catalogGovernanceItemDetail",
        "catalog/governance/items/{item_id}",
    ),
    (
        "catalogGovernanceProviderMappingReviewPlan",
        "catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan",
    ),
    (
        "catalogGovernanceProviderMappingReview",
        "catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review",
    ),
    ("metadataCandidateReviews", "metadata/candidate-reviews"),
    (
        "metadataCandidateReviewBatchApplicationPlan",
        "metadata/candidate-reviews/batch-application-plan",
    ),
    (
        "metadataCandidateReviewBatchApply",
        "metadata/candidate-reviews/batch-apply",
    ),
    (
        "metadataCandidateReviewBatches",
        "metadata/candidate-reviews/batches",
    ),
    (
        "metadataCandidateReviewBatch",
        "metadata/candidate-reviews/batches/{batch_id}",
    ),
    (
        "metadataCandidateReviewsForItem",
        "metadata/items/{item_id}/candidate-reviews",
    ),
    (
        "metadataCandidateReview",
        "metadata/candidate-reviews/{review_id}",
    ),
    (
        "metadataCandidateReviewApply",
        "metadata/candidate-reviews/{review_id}/apply",
    ),
    (
        "metadataCandidateReviewRelatedHierarchyApplicationPlan",
        "metadata/candidate-reviews/{review_id}/related-hierarchy/application-plan",
    ),
    (
        "metadataCandidateReviewRelatedHierarchyApply",
        "metadata/candidate-reviews/{review_id}/related-hierarchy/apply",
    ),
    ("events", "events"),
    ("jobs", "jobs"),
    ("sourceFingerprintHashes", "source-fingerprint-hashes"),
    (
        "sourceFingerprintHashJobRetry",
        "source-fingerprint-hashes/jobs/{job_id}/retry",
    ),
    (
        "sourceDuplicateReconciliationPlan",
        "libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan",
    ),
    (
        "sourceDuplicateReconciliationApply",
        "libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply",
    ),
    (
        "libraryMetadataProfile",
        "libraries/{library_id}/metadata-profile",
    ),
    ("libraryScan", "libraries/{library_id}/scan"),
    ("libraryNfoImport", "libraries/{library_id}/nfo/import"),
    ("libraryNfoExport", "libraries/{library_id}/nfo/export"),
    ("playbackSessions", "playback/sessions"),
    ("playbackRuntime", "playback/runtime"),
    ("playbackRenderers", "playback/renderers"),
    ("playbackSupport", "playback/support"),
    (
        "addonRuntimeReadiness",
        "addons/{addon_id}/runtime-readiness",
    ),
    ("addonRoutingPlans", "addons/{addon_id}/routing-plans"),
    ("storageBackends", "storage/backends"),
    (
        "storageBackendCircuitBreakerReset",
        "storage/backends/{backend_key}/circuit-breaker/reset",
    ),
    ("storageStaging", "storage/staging"),
    (
        "storageVfsCacheRepairRemediationPlan",
        "storage/vfs-cache/repair/remediation-plan",
    ),
    (
        "storageVfsCacheRepairAutomationPlan",
        "storage/vfs-cache/repair/automation/plan",
    ),
    (
        "storageVfsCacheRepairAutomationJobs",
        "storage/vfs-cache/repair/automation/jobs",
    ),
    (
        "storageVfsCacheRepairTargets",
        "storage/vfs-cache/repair/targets",
    ),
    (
        "storageVfsCacheRepairTargetPreview",
        "storage/vfs-cache/repair/targets/{target_ref}/preview",
    ),
    (
        "storageVfsCacheRepairTargetRefreshCache",
        "storage/vfs-cache/repair/targets/{target_ref}/refresh-cache",
    ),
    (
        "storageVfsCacheRepairTargetEnqueue",
        "storage/vfs-cache/repair/targets/{target_ref}/jobs",
    ),
    (
        "storageVfsCacheRepairJobExecute",
        "storage/vfs-cache/repair/jobs/{job_id}/execute",
    ),
    (
        "storageVfsCacheRepairJobRetry",
        "storage/vfs-cache/repair/jobs/{job_id}/retry",
    ),
    (
        "storageVfsCacheRepairActionPlan",
        "storage/vfs-cache/repair/action-plan",
    ),
    (
        "storageVfsCacheRepairRefreshCache",
        "storage/vfs-cache/repair/refresh-cache",
    ),
    ("systemConfig", "system/config"),
    ("settingsMetadataRawCache", "settings/metadata/raw-cache"),
];

const ADMIN_ROUTE_EXCLUSION_SUFFIXES: [AdminRouteExclusionSuffix; 25] = [
    AdminRouteExclusionSuffix {
        suffix: "access/invitations",
        reason: "Invitation lifecycle routes are implemented for Admin operators but are not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "access/invitations/{invitation_id}/revoke",
        reason: "Invitation lifecycle routes are implemented for Admin operators but are not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/install-guide-preview",
        reason: "Install guide preview is a planning helper and does not have a stable generated Admin Web route key yet.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/task-runs",
        reason: "Addon task-run operator workflows are implemented server-side but are not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/task-runs/{job_id}",
        reason: "Addon task-run operator workflows are implemented server-side but are not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/task-runs/{job_id}/retry",
        reason: "Addon task-run operator workflows are implemented server-side but are not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/tokens",
        reason: "Addon credential routes are derived from addonDetail in Admin Web until credential route keys are stabilized.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/tokens/{token_id}/rotate",
        reason: "Addon credential routes are derived from addonDetail in Admin Web until credential route keys are stabilized.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/tokens/{token_id}/revoke",
        reason: "Addon credential routes are derived from addonDetail in Admin Web until credential route keys are stabilized.",
    },
    AdminRouteExclusionSuffix {
        suffix: "addons/{addon_id}/grants",
        reason: "Addon grant routes are derived from addonDetail in Admin Web until grant route keys are stabilized.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/candidates/{candidate_id}/accept",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/ingests/process-next",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/ingests/{ingest_id}/requeue",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/{artifact_id}/publish",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/lifecycle",
        reason: "Managed artwork maintenance diagnostics are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/storage-drift",
        reason: "Managed artwork maintenance diagnostics are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/remediation-plan",
        reason: "Managed artwork maintenance diagnostics are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/remediate-stray-files",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "artwork/artifacts/cleanup",
        reason: "Managed artwork maintenance commands are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "events/{event_id}/addon-event-attempts",
        reason: "Addon event delivery control routes are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "events/{event_id}/addon-event-scheduler/work",
        reason: "Addon event delivery control routes are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "events/{event_id}/addon-events/deliver",
        reason: "Addon event delivery control routes are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "events/{event_id}/addon-events/replay",
        reason: "Addon event delivery control routes are server-side operator workflows not generated as Admin Web route constants in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "jobs/{job_id}/cancel",
        reason: "Job cancellation is an Admin command endpoint and is not generated as an Admin Web route constant in this slice.",
    },
    AdminRouteExclusionSuffix {
        suffix: "settings/playback/runtime",
        reason: "Playback runtime settings mutation is not generated as an Admin Web route constant until the settings UI owns this workflow.",
    },
];

#[must_use]
pub fn admin_typescript_contract() -> String {
    let mut output = String::new();
    output.push_str(
        "// Generated by `cargo run -p nako-api --example emit-admin-typescript-contract`.\n",
    );
    output.push_str("// Do not edit generated output by hand.\n\n");
    output.push_str(&format!(
        "export const NAKO_ADMIN_API_VERSION = \"{}\" as const;\n\n",
        ADMIN_API_VERSION
    ));
    output.push_str("export const NAKO_ADMIN_ROUTES = {\n");
    for (key, suffix) in ADMIN_ROUTE_SUFFIXES {
        output.push_str(&format!("  {key}: \"{}\",\n", admin_route_path(suffix)));
    }
    output.push_str("} as const;\n\n");
    output.push_str(CONTRACT_BODY);
    output
}

#[must_use]
pub fn admin_contract_routes() -> Vec<AdminContractRoute> {
    ADMIN_ROUTE_SUFFIXES
        .iter()
        .map(|(key, suffix)| AdminContractRoute {
            key,
            path: admin_route_path(suffix),
        })
        .collect()
}

#[must_use]
pub fn admin_contract_route_exclusions() -> Vec<AdminContractRouteExclusion> {
    ADMIN_ROUTE_EXCLUSION_SUFFIXES
        .iter()
        .map(|exclusion| AdminContractRouteExclusion {
            path: admin_route_path(exclusion.suffix),
            reason: exclusion.reason,
        })
        .collect()
}

#[must_use]
pub fn normalize_admin_route_path(path: &str) -> String {
    let mut normalized = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != ':' {
            normalized.push(ch);
            continue;
        }

        let mut parameter = String::new();
        while let Some(next) =
            chars.next_if(|next| next.is_ascii_alphanumeric() || *next == '_' || *next == '-')
        {
            parameter.push(next);
        }

        if parameter.is_empty() {
            normalized.push(ch);
        } else {
            normalized.push('{');
            normalized.push_str(&parameter);
            normalized.push('}');
        }
    }

    normalized
}

fn admin_route_path(suffix: &str) -> String {
    format!("/admin/{ADMIN_API_VERSION}/{suffix}")
}

const CONTRACT_BODY: &str = r#"export type AdminApiRouteKey = keyof typeof NAKO_ADMIN_ROUTES;

export interface AdminPageQuery {
  limit?: number;
  offset?: number;
}

export interface AdminCatalogGovernanceItemsQuery extends AdminPageQuery {
  library_id?: string;
  max_confidence_milli?: number;
}

export type AdminSettingsSource = "configured" | "admin";

export type AdminSettingsEffect = "active" | "requires_restart";

export type AdminAccessMode = "single_admin";

export type AdminAccessPrincipalKind = "local_admin";

export type AdminAccessCapabilityState = "active" | "planned";

export type AdminLibraryAccessLevel = "manage";

export type AdminLibraryAccessReason = "single_admin_mode";

export type AdminUserStatus = "active" | "disabled";

export type AdminUserRole = "administrator" | "library_manager" | "viewer";

export type AdminLibraryAccessPolicyLevel = "none" | "browse" | "play" | "manage";

export type AdminLibraryAccessPolicyScope =
  | {
      scope: "user";
      user_id: string;
    }
  | {
      scope: "role";
      role: AdminUserRole;
    };

export interface AdminOutboxEventsQuery extends AdminPageQuery {
  kind?: string;
  status?: string;
  library_id?: string;
  source_id?: string;
}

export interface AdminJobsQuery extends AdminPageQuery {
  status?: string;
  kind?: string;
  resource_class?: string;
  library_id?: string;
  source_id?: string;
}

export type AdminSourceFingerprintHashMode = "full" | "partial";

export type AdminJobPriority = "low" | "normal" | "high";

export type AdminJobStatus =
  | "queued"
  | "running"
  | "succeeded"
  | "failed"
  | "cancelled";

export interface AdminSourceFingerprintHashEnqueueRequest {
  library_id: string;
  source_id: string;
  mode: AdminSourceFingerprintHashMode;
  partial_prefix_bytes?: number | null;
  priority?: AdminJobPriority | null;
}

export interface AdminSourceFingerprintHashRetryRequest {
  max_attempts?: number | null;
  next_attempt_at?: string | null;
}

export interface AdminSourceDuplicateReconciliationPlanQuery extends AdminPageQuery {}

export type AdminSourceDuplicateReconciliationApplyExpectedAction = "suggest_relationship";

export interface AdminSourceDuplicateReconciliationApplyRequest {
  duplicate_source_id: string;
  expected_action: AdminSourceDuplicateReconciliationApplyExpectedAction;
}

export type AdminSourceFingerprintEvidenceKind =
  | "content_hash"
  | "backend_fingerprint"
  | "size_etag"
  | "size_modified_time"
  | "locator_only";

export type AdminSourceDuplicateEvidenceKind =
  | "strong_fingerprint"
  | "size_and_etag"
  | "path_evidence"
  | "filesystem_link"
  | "manual"
  | { other: string };

export type AdminSourceDuplicateRelationshipStatus =
  | "suggested"
  | "confirmed"
  | "rejected";

export type AdminSourceDuplicateReconciliationAction =
  | "suggest_relationship"
  | "preserve_suggested"
  | "preserve_confirmed"
  | "preserve_rejected"
  | "refresh_source_fingerprint";

export interface AdminSourceDuplicateReconciliationCandidate {
  source_id: string;
  duplicate_source_id: string;
  evidence_kind: AdminSourceDuplicateEvidenceKind;
  confidence_milli: number | null;
  stale: boolean;
  relationship_id: string | null;
  existing_status: AdminSourceDuplicateRelationshipStatus | null;
  recommended_action: AdminSourceDuplicateReconciliationAction;
}

export interface AdminSourceDuplicateReconciliationPlanResponse {
  admin_api_version: string;
  library_id: string;
  source_id: string;
  fingerprint_evidence_kind: AdminSourceFingerprintEvidenceKind;
  confidence_milli: number;
  stale: boolean;
  candidates: AdminSourceDuplicateReconciliationCandidate[];
  page: PageInfo;
}

export interface AdminSourceDuplicateReconciliationApplyResponse {
  admin_api_version: string;
  library_id: string;
  source_id: string;
  duplicate_source_id: string;
  relationship_id: string;
  relationship_status: AdminSourceDuplicateRelationshipStatus;
  applied_action: AdminSourceDuplicateReconciliationAction;
  created: boolean;
}

export interface AdminPlaybackSessionsQuery extends AdminPageQuery {
  principal_id?: string;
  source_id?: string;
  state?: string;
}

export interface AdminPlaybackSupportQuery {
  session_id?: string;
  source_id?: string;
}

export interface AdminAcquisitionIntakeCandidatesQuery extends AdminPageQuery {
  library_id?: string;
  state?: string;
  source_kind?: string;
  managed_import_artifact_id?: string;
}

export interface AdminWatchFolderDiscoveryRequest {
  target_library_id: string;
  root_uri?: string;
  max_depth?: number;
}

export interface AdminWatchFolderSuppression {
  target_library_id: string;
  scope_scheme: string;
  scope_ref_redacted: string;
  owner: string;
  reason: string;
  expires_at_ms: number;
  completion: string;
}

export interface AdminStorageStagingQuery extends AdminPageQuery {
  purpose?: string;
  state?: string;
}

export interface AdminStorageBackendsQuery extends AdminPageQuery {}

export type AdminStorageStagingPressureStatus =
  | "disabled"
  | "healthy"
  | "elevated"
  | "critical"
  | "exhausted";

export type AdminStorageStagingAttributionKind =
  | "attributed"
  | "ambiguous"
  | "unknown";

export type StorageBackendKind =
  | "local"
  | "webdav";

export interface AdminStorageStagingPressureSummary {
  status: AdminStorageStagingPressureStatus;
  used_ratio_milli: number | null;
  total_records: number;
  in_flight_records: number;
  failed_records: number;
  unknown_size_records: number;
  active_leases: number;
  ffmpeg_input_records: number;
  probe_input_records: number;
}

export interface AdminStorageStagingPolicySlice {
  backend_key: string;
  library_id: string | null;
  library_name: string | null;
  backend_kind: StorageBackendKind | null;
  source_scheme: string;
  configured_max_bytes: number;
  used_manifest_bytes: number;
  pressure: AdminStorageStagingPressureSummary;
}

export interface AdminStorageStagingPurposeStateSummary {
  purpose: string;
  state: string;
  record_count: number;
  used_manifest_bytes: number;
  active_leases: number;
  unknown_size_records: number;
}

export interface AdminStorageStagingCleanupPurposeStateSummary {
  purpose: string;
  state: string;
  record_count: number;
  cleanup_candidate_bytes: number;
  active_leases: number;
  unknown_size_records: number;
}

export type StorageBackendHealthStatus =
  | "healthy"
  | "recovering"
  | "unavailable";

export type StorageCircuitBreakerState =
  | "closed"
  | "half_open"
  | "open";

export type StorageFailureClass =
  | "timeout"
  | "unavailable"
  | "permission"
  | "rate_limited"
  | "stale_cache"
  | "partial_read"
  | "budget"
  | "security"
  | "unknown";

export type VfsCacheOperation = "stat" | "list";

export type AdminVfsCacheRepairClassification =
  | "healthy"
  | "repairable_stale_fallback"
  | "retryable_refresh_failure"
  | "operator_action_required"
  | "unknown_failure";

export type AdminVfsCacheRepairAction =
  | "none"
  | "refresh_cache"
  | "fix_backend_configuration"
  | "inspect_failure";

export type AdminVfsCacheRepairActionPlanStatus =
  | "no_action"
  | "executable"
  | "plan_only";

export type AdminVfsCacheRepairActionPlanReason =
  | "no_repair_diagnostic"
  | "no_action_required"
  | "refresh_cache_executable"
  | "target_scoped_execution_unavailable"
  | "backend_configuration_required"
  | "manual_failure_inspection_required";

export interface AdminVfsCacheRepairActionReadiness {
  status: AdminVfsCacheRepairActionPlanStatus;
  api_executable: boolean;
  reasons: AdminVfsCacheRepairActionPlanReason[];
}

export interface AdminVfsCacheRepairActionBoundary {
  refreshes_vfs_cache: boolean;
  changes_backend_configuration: boolean;
  requires_manual_failure_inspection: boolean;
  deletes_cache_entries: boolean;
  writes_library_files: boolean;
  starts_durable_job: boolean;
}

export interface AdminVfsCacheRepairExecutableAction {
  method: "POST";
  route_key: AdminApiRouteKey;
  route_path: string;
}

export interface AdminVfsCacheRepairTarget {
  target_ref: string;
  scheme: string;
  operation: VfsCacheOperation;
  failed_at_ms: number;
  failure_count: number;
  classification: AdminVfsCacheRepairClassification;
  recommended_action: AdminVfsCacheRepairAction;
  failure_class: StorageFailureClass | null;
  retryable: boolean;
  safe_message: string | null;
}

export interface AdminVfsCacheRepairRemediationPlanBoundary {
  read_only: boolean;
  refreshes_vfs_cache: boolean;
  changes_backend_configuration: boolean;
  deletes_cache_entries: boolean;
  writes_library_files: boolean;
  starts_durable_job: boolean;
}

export interface AdminVfsCacheRepairClassificationCount {
  classification: AdminVfsCacheRepairClassification;
  count: number;
}

export interface AdminVfsCacheRepairRemediationActionGroup {
  action: AdminVfsCacheRepairAction;
  count: number;
  status: AdminVfsCacheRepairActionPlanStatus;
  readiness: AdminVfsCacheRepairActionReadiness;
  boundary: AdminVfsCacheRepairActionBoundary;
  executable_action: AdminVfsCacheRepairExecutableAction | null;
  sample_targets: AdminVfsCacheRepairTarget[];
}

export interface AdminVfsCacheRepairAutomationPolicyRequest {
  enabled: boolean;
}

export type AdminVfsCacheRepairAutomationBlockReason =
  | "policy_disabled"
  | "backend_configuration_required"
  | "manual_failure_inspection_required"
  | "no_action_required";

export interface AdminVfsCacheRepairAutomationBoundary {
  reads_repair_targets: boolean;
  may_start_durable_jobs: boolean;
  refreshes_vfs_cache: boolean;
  changes_backend_configuration: boolean;
  deletes_cache_entries: boolean;
  writes_library_files: boolean;
}

export interface AdminVfsCacheRepairAutomationEligibleTarget {
  target: AdminVfsCacheRepairTarget;
}

export interface AdminVfsCacheRepairAutomationBlockedTarget {
  target: AdminVfsCacheRepairTarget;
  reason: AdminVfsCacheRepairAutomationBlockReason;
}

export interface AdminVfsCacheRepairAutomationPolicyReport {
  enabled: boolean;
  total_unresolved_targets: number;
  eligible_targets: AdminVfsCacheRepairAutomationEligibleTarget[];
  blocked_targets: AdminVfsCacheRepairAutomationBlockedTarget[];
  boundary: AdminVfsCacheRepairAutomationBoundary;
}

export interface AdminVfsCacheRepairAutomationPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  policy: AdminVfsCacheRepairAutomationPolicyReport;
}

export interface AdminVfsCacheRepairAutomationEnqueueRequest {
  enabled: boolean;
  priority?: AdminJobPriority | null;
}

export interface AdminVfsCacheRepairAutomationJob {
  outcome: AdminVfsCacheRepairEnqueueOutcome;
  job_id: string;
  status: AdminJobStatus;
  priority: AdminJobPriority;
  resource_class: string;
  library_id: string | null;
  source_id: string | null;
}

export interface AdminVfsCacheRepairAutomationEnqueueResponse {
  admin_api_version: string;
  public_api_version: string;
  policy: AdminVfsCacheRepairAutomationPolicyReport;
  jobs: AdminVfsCacheRepairAutomationJob[];
  enqueued_count: number;
  already_queued_count: number;
}

export interface AdminStorageBackendHealthDiagnostic {
  backend_key: string;
  library_id: string | null;
  scheme: string;
  status: StorageBackendHealthStatus;
  circuit_breaker_state: StorageCircuitBreakerState;
  consecutive_failures: number;
  last_success_at_ms: number | null;
  last_failure_at_ms: number | null;
  last_failure_class: StorageFailureClass | null;
  last_failure_safe_message: string | null;
  circuit_opened_at_ms: number | null;
  backoff_until_ms: number | null;
  updated_at_ms: number;
}

export interface AdminStorageBackendHealthDiagnosticsResponse {
  admin_api_version: string;
  public_api_version: string;
  backends: AdminStorageBackendHealthDiagnostic[];
  page: PageInfo;
}

export interface AdminStorageBackendHealthResetResponse {
  admin_api_version: string;
  public_api_version: string;
  backend: AdminStorageBackendHealthDiagnostic;
  reset_at_ms: number;
}

export interface AdminGeneratedArtifactProposalsQuery extends AdminPageQuery {}

export interface AdminGeneratedArtifactApplyOutcomesQuery extends AdminPageQuery {}

export type AdminGeneratedArtifactApplyRecoveryAttention =
  | "needs_repair"
  | "needs_review"
  | "replay_only"
  | "resolved";

export interface AdminGeneratedArtifactApplyRecoveryQuery extends AdminPageQuery {
  attention?: AdminGeneratedArtifactApplyRecoveryAttention;
}

export interface AdminGeneratedArtifactReviewRequest {
  decision: "accept" | "reject";
}

export interface AdminGeneratedArtifactMetadataApplyRequest {
  idempotency_key: string;
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanRequest {
  artifact_ids: string[];
}

export interface AdminGeneratedArtifactMetadataBulkApplyRequest {
  artifact_ids: string[];
  idempotency_key: string;
}

export type AdminArtworkKind =
  | "poster"
  | "backdrop"
  | "logo"
  | "thumbnail"
  | "banner";

export interface AdminItemArtworkGalleryQuery extends AdminPageQuery {}

export interface AdminSelectItemArtworkRequest {
  artifact_id: string;
}

export interface ManagedArtworkIngestSummary {
  id: string;
  candidate_id: string;
  job_id: string;
  library_id: string;
  item_id: string;
  kind: AdminArtworkKind | string;
  status: string;
  has_artifact: boolean;
  has_failure: boolean;
  failure_code: string | null;
  created_at: string;
  updated_at: string;
}

export interface SelectedArtworkSummary {
  id: string;
  library_id: string;
  item_id: string;
  kind: AdminArtworkKind | string;
  artifact_id: string;
  created_at: string;
  updated_at: string;
}

export interface AdminPublicImageRef {
  id: string;
  owner: Record<string, unknown>;
  kind: string;
  url: string;
  width: number | null;
  height: number | null;
  language: string | null;
  media_type: string | null;
  etag: string | null;
}

export interface AdminManagedArtworkGallerySummary {
  candidates: number;
  artifacts: number;
  selected: number;
}

export interface AdminManagedArtworkGalleryCandidate {
  id: string;
  addon_id: string;
  side_effect_id: string;
  library_id: string;
  item_id: string;
  kind: AdminArtworkKind | string;
  source_kind: string;
  status: string;
  width: number | null;
  height: number | null;
  language: string | null;
  ingest: ManagedArtworkIngestSummary | null;
  artifact_id: string | null;
  has_stored_artifact: boolean;
  selected_artwork_count: number;
  selected: boolean;
  created_at: string;
  updated_at: string;
}

export interface AdminManagedArtworkGalleryArtifact {
  id: string;
  ingest_id: string;
  candidate_id: string;
  library_id: string;
  item_id: string;
  kind: AdminArtworkKind | string;
  selected_artwork_count: number;
  selected: boolean;
  width: number | null;
  height: number | null;
  byte_len: number | null;
  media_type: string | null;
  has_content_hash: boolean;
  created_at: string;
  updated_at: string;
}

export interface AdminManagedArtworkGallerySelected {
  selected_artwork: SelectedArtworkSummary;
  artifact: AdminManagedArtworkGalleryArtifact;
  image: AdminPublicImageRef;
}

export interface AdminManagedArtworkGalleryResponse {
  item_id: string;
  summary: AdminManagedArtworkGallerySummary;
  candidates: AdminManagedArtworkGalleryCandidate[];
  artifacts: AdminManagedArtworkGalleryArtifact[];
  selected: AdminManagedArtworkGallerySelected[];
  page: PageInfo;
}

export interface PublishSelectedArtworkResponse {
  selected_artwork: SelectedArtworkSummary;
  image: AdminPublicImageRef;
  changed: boolean;
}

export interface UnpublishedSelectedArtworkSummary {
  selected_artwork: SelectedArtworkSummary;
  previous_image: AdminPublicImageRef;
}

export interface UnpublishSelectedArtworkResponse {
  item_id: string;
  kind: AdminArtworkKind | string;
  changed: boolean;
  unpublished: UnpublishedSelectedArtworkSummary | null;
}

export type AdminMetadataRefreshMode =
  | "none"
  | "validation_only"
  | "default"
  | "missing_only"
  | "full_refresh";

export type AdminLocalMetadataPolicy =
  | "disabled"
  | "read_only"
  | "local_first"
  | "remote_first"
  | "write_sidecar";

export interface AdminMetadataScanPolicy {
  enabled: boolean;
  addon_scrape: boolean;
  addon_writeback: boolean;
}

export interface AdminMetadataProfile {
  item_kinds: string[];
  local_readers: string[];
  metadata_providers: string[];
  image_providers: string[];
  language: string | null;
  country: string | null;
  refresh_mode: AdminMetadataRefreshMode;
  local_metadata_policy: AdminLocalMetadataPolicy;
  scan: AdminMetadataScanPolicy;
}

export interface AdminMetadataScanAcquisitionPlan {
  local_nfo_import: boolean;
  provider_refresh: boolean;
  addon_scrape: boolean;
  addon_writeback: boolean;
  embedded_read: boolean;
  sidecar_read: boolean;
  image_discovery: boolean;
}

export interface AdminUpdateLibraryMetadataProfileRequest {
  profile: AdminMetadataProfile;
}

export interface AdminLibraryMetadataProfileResponse {
  admin_api_version: string;
  public_api_version: string;
  library_id: string;
  profile: AdminMetadataProfile;
  scan_acquisition_plan: AdminMetadataScanAcquisitionPlan;
}
export type AddonStatus = "enabled" | "disabled" | "unregistered";

export type AddonScope =
  | "catalog_read"
  | "item_metadata_read"
  | "item_metadata_suggest"
  | "image_read"
  | "subtitle_read"
  | "stream_url_read"
  | "recommendation_write"
  | "automation_run"
  | "webhook_event_read"
  | "renderer_adapter_read"
  | "renderer_adapter_control"
  | "acquisition_search_read"
  | "acquisition_link_check_read";

export type AddonResource =
  | "catalog"
  | "metadata"
  | "image"
  | "stream"
  | "subtitle"
  | "recommendation"
  | "automation"
  | "webhook"
  | "renderer_adapter"
  | "resource_search"
  | "resource_link_check";

export type AddonEntryPointKind =
  | "item_action"
  | "library_action"
  | "admin_action"
  | "settings"
  | "diagnostics"
  | "task_launcher";

export type AddonPermission =
  | "metadata_write"
  | "artwork_write"
  | "subtitle_write"
  | "library_file_write";

export type AddonTokenStatus = "active" | "revoked" | "rotated";

export type AddonAuth = "none" | "bearer" | "shared_secret";

export type AdminAddonHealthCheckStatus =
  | "reachable"
  | "degraded"
  | "unhealthy"
  | "unreachable"
  | "protocol_mismatch"
  | "invalid_manifest";

export type AdminAddonResourceCallDiagnosticStatus =
  | "succeeded"
  | "missing_resource"
  | "missing_grant"
  | "authorization_gap"
  | "unreachable"
  | "protocol_mismatch"
  | "retryable_http_failure"
  | "http_failure"
  | "unsafe_response";

export type AddonResourceLinkType =
  | "aliyun"
  | "baidu"
  | "quark"
  | "tianyi"
  | "uc"
  | "mobile"
  | "115"
  | "pikpak"
  | "xunlei"
  | "123"
  | "magnet"
  | "ed2k"
  | "web"
  | "other";

export type AddonResourceLinkCheckStatus =
  | "reachable"
  | "unavailable"
  | "password_needed"
  | "unsupported"
  | "rate_limited"
  | "error"
  | "unknown";

export type AddonResourceSearchIntent =
  | { kind: "free_text"; text: string }
  | {
      kind: "media_title";
      title: string;
      year?: number;
      media_kind?: string;
    }
  | { kind: "external_id"; id_kind: string; value: string }
  | { kind: "exact_link"; url: string };

export type AddonResourceSearchProviderStatus = "ok" | "error" | "skipped";

export type AddonResourceSearchProviderFinality =
  | "complete"
  | "partial"
  | "unknown";

export type AddonSubtitleFormat = "vtt" | "srt";

export type AddonSubtitleProviderStatus = "ok" | "error" | "skipped";

export type AdminAddonSubtitleDeliveryKind =
  | "inline"
  | "download_url"
  | "artifact_ref";

export type AdminSubtitleSidecarRole =
  | "default"
  | "forced"
  | "sdh"
  | "commentary";

export type AdminSubtitleImportConflictPolicy =
  | "create_missing"
  | "replace_existing";

export type AdminSubtitleImportBackupPolicy =
  | "none"
  | "existing_file_keep_latest";

export type AdminSubtitleImportPlanStatus = "ready" | "blocked";

export type AdminSubtitleImportPlanReason =
  | "ready"
  | "media_source_matches_item"
  | "candidate_language_mismatch"
  | "candidate_format_mismatch";

export type AdminSubtitleImportApplyStatus = "applied" | "already_applied";

export interface AdminAddonsQuery {
  status?: AddonStatus;
}

export interface RegisterAddonRequest {
  id?: string;
  manifest: AdminAddonManifest;
  outbound_task_dispatch_secret_env?: string;
  granted_scopes?: AddonScope[];
  status?: AddonStatus;
}

export interface AdminAddonResourceDeclaration {
  kind: AddonResource;
  path: string;
  input_schema: string | null;
  output_schema: string | null;
  required_scopes: AddonScope[];
  timeout_ms: number | null;
  max_attempts: number | null;
}

export interface AdminAddonManifest {
  id: string;
  name: string;
  version: string;
  protocol_version: string;
  base_url: string;
  description: string | null;
  resources: AdminAddonResourceDeclaration[];
  entry_points?: AdminAddonEntryPointDeclaration[];
  hosted_pages?: AdminAddonHostedPageDeclaration[];
  configuration_schema?: AdminAddonConfigurationSchemaDeclaration;
  secret_reference_fields?: AdminAddonSecretReferenceFieldDeclaration[];
  event_subscriptions?: AdminAddonEventSubscriptionDeclaration[];
  tasks?: AdminAddonTaskDeclaration[];
  auth: AddonAuth;
  default_timeout_ms: number | null;
  default_max_attempts: number | null;
  scopes: AddonScope[];
}

export interface AdminAddonEntryPointDeclaration {
  id: string;
  kind: AddonEntryPointKind;
  label: string;
  path: string;
  hosted_page_id?: string;
  required_scopes?: AddonScope[];
}

export interface AdminAddonHostedPageDeclaration {
  id: string;
  title: string;
  path: string;
  required_scopes?: AddonScope[];
}

export interface AdminAddonConfigurationSchemaDeclaration {
  schema_id: string;
  schema: Record<string, unknown>;
}

export interface AdminAddonSecretReferenceFieldDeclaration {
  id: string;
  label: string;
  description?: string;
  required?: boolean;
}

export interface AdminAddonEventSubscriptionDeclaration {
  id: string;
  event_kind: string;
  path: string;
  required_scopes?: AddonScope[];
  filters?: Record<string, unknown>;
}

export interface AdminAddonTaskDeclaration {
  id: string;
  name: string;
  path: string;
  input_schema?: string;
  output_schema?: string;
  description?: string;
  required_scopes?: AddonScope[];
  timeout_ms?: number;
  max_attempts?: number;
}

export interface AdminAddonRegistrationSummary {
  id: string;
  manifest_id: string;
  name: string;
  version: string;
  protocol_version: string;
  base_url: string;
  outbound_task_dispatch_secret_env?: string;
  granted_scopes: string[];
  status: AddonStatus;
  created_at: string;
  updated_at: string;
}

export interface AdminAddonRegistrationDetail {
  summary: AdminAddonRegistrationSummary;
  manifest: AdminAddonManifest;
}

export interface AdminAddonRegistrationResponse {
  addon: AdminAddonRegistrationDetail;
}

export type AdminAddonLifecycleIntent = "install" | "update" | "remove";

export interface AdminAddonManagerPlanRequest {
  intent: AdminAddonLifecycleIntent;
  operator_confirmed: boolean;
}

export interface AdminAddonRegistrationsResponse {
  addons: AdminAddonRegistrationSummary[];
}

export interface UpdateAddonStatusRequest {
  status: AddonStatus;
}

export type AddonRuntimeKind = "http_sidecar";

export interface AdminAddonRuntimeRequirement {
  kind: AddonRuntimeKind;
  image?: string;
  binary?: string;
  command?: string;
}

export interface AdminAddonSecretReferenceBinding {
  field_id: string;
  secret_ref: string;
}

export interface AdminAddonInstallDescriptor {
  manifest: AdminAddonManifest;
  runtime: AdminAddonRuntimeRequirement;
  secret_reference_bindings?: AdminAddonSecretReferenceBinding[];
  install_notes?: string[];
}

export type AdminAddonRuntimeReferenceKind = "image" | "binary" | "command";

export interface AdminAddonRuntimeReference {
  kind: AdminAddonRuntimeReferenceKind;
  value: string;
}

export type AdminAddonInstallStepKind =
  | "run_sidecar"
  | "configure_secret_reference"
  | "register_manifest"
  | "grant_scopes";

export interface AdminAddonProtocolInstallSecretField {
  id: string;
  label: string;
  required: boolean;
  provided: boolean;
}

export interface AdminAddonProtocolInstallStep {
  kind: AdminAddonInstallStepKind;
  summary: string;
}

export interface AdminAddonProtocolInstallGuide {
  manifest_id: string;
  addon_name: string;
  protocol_version: string;
  runtime_kind: AddonRuntimeKind;
  runtime_reference: AdminAddonRuntimeReference;
  base_url_scheme: string;
  base_url_configured: boolean;
  declared_resources: AddonResource[];
  declared_scopes: AddonScope[];
  required_secret_fields: AdminAddonProtocolInstallSecretField[];
  provided_secret_refs: string[];
  missing_required_secret_fields: string[];
  has_configuration_schema: boolean;
  entry_point_count: number;
  hosted_page_count: number;
  task_count: number;
  event_subscription_count: number;
  install_steps: AdminAddonProtocolInstallStep[];
}

export type AdminAddonSourceCatalogSourceKind = "builtin_official";

export interface AdminAddonSourceCatalogSource {
  id: string;
  name: string;
  description?: string;
  kind: AdminAddonSourceCatalogSourceKind;
  entry_count: number;
  provides_package_signing: boolean;
  provides_process_supervision: boolean;
  provides_provider_breadth: boolean;
}

export interface AdminAddonSourceCatalogSourcesResponse {
  sources: AdminAddonSourceCatalogSource[];
}

export interface AdminAddonSourceCatalogEntry {
  source_id: string;
  entry_id: string;
  manifest_id: string;
  addon_name: string;
  addon_version: string;
  protocol_version: string;
  description?: string;
  runtime_kind: AddonRuntimeKind;
  resources: AddonResource[];
  scopes: AddonScope[];
  tasks: string[];
  package_signing_verified: boolean;
  lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary;
}

export interface AdminAddonSourceCatalogEntriesResponse {
  source_id: string;
  entries: AdminAddonSourceCatalogEntry[];
}

export interface AdminAddonSourceCatalogResolveResponse {
  source_id: string;
  entry: AdminAddonSourceCatalogEntry;
  descriptor: AdminAddonInstallDescriptor;
  install_guide: AdminAddonProtocolInstallGuide;
}

export interface AdminAddonHealthCheckResponse {
  addon_id: string;
  manifest_id: string;
  status: AdminAddonHealthCheckStatus;
  latency_ms: number;
  protocol_version?: string;
  addon_version?: string;
  resource_count?: number;
  protocol_checked_at?: string;
  safe_error_code?: string;
}

export interface AdminAddonEntryPointSurface {
  id: string;
  kind: AddonEntryPointKind;
  label: string;
  path: string;
  hosted_page_id?: string;
  required_scopes: AddonScope[];
}

export interface AdminAddonHostedPageSurface {
  id: string;
  title: string;
  path: string;
  url: string;
  required_scopes: AddonScope[];
}

export interface AdminAddonConfigurationSchemaSurface {
  schema_id: string;
  schema: Record<string, unknown>;
}

export interface AdminAddonSecretReferenceFieldSurface {
  id: string;
  label: string;
  description?: string;
  required: boolean;
}

export interface AdminAddonTaskSurface {
  id: string;
  name: string;
  path: string;
  input_schema?: string;
  output_schema?: string;
  description?: string;
  required_scopes: AddonScope[];
  timeout_ms?: number;
  max_attempts?: number;
}

export interface AdminAddonEventSubscriptionSurface {
  id: string;
  event_kind: string;
  path: string;
  required_scopes: AddonScope[];
  filters: Record<string, unknown>;
}

export interface AdminAddonSurfacesResponse {
  addon_id: string;
  manifest_id: string;
  entry_points: AdminAddonEntryPointSurface[];
  hosted_pages: AdminAddonHostedPageSurface[];
  configuration_schema?: AdminAddonConfigurationSchemaSurface;
  secret_reference_fields: AdminAddonSecretReferenceFieldSurface[];
  tasks: AdminAddonTaskSurface[];
  event_subscriptions: AdminAddonEventSubscriptionSurface[];
}

export interface AdminAddonResourceCallDiagnosticRequest {
  resource: AddonResource;
  payload?: Record<string, unknown>;
}

export interface AdminAddonResourceCallDiagnosticResponse {
  addon_id: string;
  manifest_id: string;
  resource: AddonResource;
  status: AdminAddonResourceCallDiagnosticStatus;
  latency_ms: number;
  attempts: number;
  http_status?: number;
  safe_error_code?: string;
}

export interface AdminAddonResourceSearchDiagnosticRequest {
  query: string;
  intent: AddonResourceSearchIntent;
  limit?: number;
  sources?: string[];
  link_types?: AddonResourceLinkType[];
  refresh?: boolean;
  context?: Record<string, unknown>;
}

export interface AdminAddonResourceSearchProviderDiagnostic {
  provider_id: string;
  status: AddonResourceSearchProviderStatus;
  result_count: number;
  finality: AddonResourceSearchProviderFinality;
  has_safe_message: boolean;
}

export interface AdminAddonResourceSearchDiagnosticResponse {
  addon_id: string;
  manifest_id: string;
  status: AdminAddonResourceCallDiagnosticStatus;
  latency_ms: number;
  attempts: number;
  limit: number;
  total: number;
  result_count: number;
  link_count: number;
  merged_link_count: number;
  provider_executions: AdminAddonResourceSearchProviderDiagnostic[];
  http_status?: number;
  safe_error_code?: string;
}

export interface AdminAddonResourceSearchRequest {
  query: string;
  intent: AddonResourceSearchIntent;
  limit?: number;
  sources?: string[];
  link_types?: AddonResourceLinkType[];
  refresh?: boolean;
  context?: Record<string, unknown>;
}

export interface AdminAddonResourceSearchResultSummary {
  result_ref_fingerprint: string;
  title: string;
  content?: string;
  source: string;
  tags?: string[];
  score: number;
  links: AdminAddonResourceSearchLinkSummary[];
}

export interface AdminAddonResourceSearchLinkSummary {
  selection_id: string;
  link_type: AddonResourceLinkType;
  source: string;
  source_ref_redacted: string;
  has_password: boolean;
  has_note: boolean;
}

export interface AdminAddonResourceSearchResponse {
  addon_id: string;
  manifest_id: string;
  search_id: string;
  status: AdminAddonResourceCallDiagnosticStatus;
  latency_ms: number;
  attempts: number;
  limit: number;
  total: number;
  result_count: number;
  results: AdminAddonResourceSearchResultSummary[];
  provider_executions: AdminAddonResourceSearchProviderDiagnostic[];
  http_status?: number;
  safe_error_code?: string;
}

export interface AdminAddonResourceSearchSelectionRequest {
  target_library_id: string;
}

export interface AdminAddonSubtitleSearchRequest {
  query: string;
  languages?: string[];
  limit?: number;
}

export interface AdminAddonSubtitleProviderDiagnostic {
  provider_id: string;
  status: AddonSubtitleProviderStatus;
  result_count: number;
  has_safe_message: boolean;
}

export interface AdminAddonSubtitleCandidateSummary {
  selection_id: string;
  candidate_ref_fingerprint: string;
  title: string;
  language: string;
  format: AddonSubtitleFormat;
  source: string;
  release?: string;
  score: number;
  delivery_kind: AdminAddonSubtitleDeliveryKind;
}

export interface AdminAddonSubtitleSearchResponse {
  addon_id: string;
  manifest_id: string;
  search_id: string;
  status: AdminAddonResourceCallDiagnosticStatus;
  latency_ms: number;
  attempts: number;
  limit: number;
  total: number;
  result_count: number;
  subtitles: AdminAddonSubtitleCandidateSummary[];
  provider_executions: AdminAddonSubtitleProviderDiagnostic[];
  http_status?: number;
  safe_error_code?: string;
}

export interface AdminAddonSubtitleSelectionRequest {}

export interface AdminAddonSubtitleSelectedReference {
  addon_id: string;
  manifest_id: string;
  search_id: string;
  selection_id: string;
  candidate_ref_fingerprint: string;
  delivery_kind: AdminAddonSubtitleDeliveryKind;
}

export interface AdminAddonSubtitleSelectionResponse {
  selected_ref: AdminAddonSubtitleSelectedReference;
  candidate: AdminAddonSubtitleCandidateSummary;
}

export interface AdminAddonSubtitleImportPlanRequest {
  media_item_id: string;
  media_source_id: string;
  language: string;
  format: AddonSubtitleFormat;
  sidecar_role: AdminSubtitleSidecarRole;
  conflict_policy: AdminSubtitleImportConflictPolicy;
  backup_policy: AdminSubtitleImportBackupPolicy;
}

export interface AdminSubtitleImportTargetSummary {
  library_id: string;
  media_item_id: string;
  media_source_id: string;
  item_title: string;
  media_file_name: string;
  source_ref_fingerprint: string;
}

export interface AdminSubtitleSidecarPlan {
  file_name: string;
  language: string;
  format: AddonSubtitleFormat;
  role: AdminSubtitleSidecarRole;
}

export interface AdminSubtitleImportPlan {
  idempotency_key: string;
  status: AdminSubtitleImportPlanStatus;
  reasons: AdminSubtitleImportPlanReason[];
  target: AdminSubtitleImportTargetSummary;
  sidecar: AdminSubtitleSidecarPlan;
  conflict_policy: AdminSubtitleImportConflictPolicy;
  backup_policy: AdminSubtitleImportBackupPolicy;
  preview_only: boolean;
  writes_library: boolean;
}

export interface AdminAddonSubtitleImportPlanResponse {
  selected_ref: AdminAddonSubtitleSelectedReference;
  candidate: AdminAddonSubtitleCandidateSummary;
  plan: AdminSubtitleImportPlan;
}

export interface AdminAddonSubtitleImportApplyRequest {
  plan_idempotency_key: string;
  media_item_id: string;
  media_source_id: string;
  language: string;
  format: AddonSubtitleFormat;
  sidecar_role: AdminSubtitleSidecarRole;
  conflict_policy: AdminSubtitleImportConflictPolicy;
  backup_policy: AdminSubtitleImportBackupPolicy;
}

export interface AdminSubtitleImportApplyReport {
  idempotency_key: string;
  status: AdminSubtitleImportApplyStatus;
  target: AdminSubtitleImportTargetSummary;
  sidecar: AdminSubtitleSidecarPlan;
  refreshed_fact: AdminSubtitleImportFactSummary;
  conflict_policy: AdminSubtitleImportConflictPolicy;
  backup_policy: AdminSubtitleImportBackupPolicy;
  write_mode: string;
  content_ref_fingerprint: string;
  byte_len: number;
  target_existed: boolean;
  backup_created: boolean;
  preview_only: boolean;
  writes_library: boolean;
}

export interface AdminSubtitleImportFactSummary {
  media_source_id: string;
  stream_index: number;
  origin: string;
  language: string;
  format: AddonSubtitleFormat;
  role: AdminSubtitleSidecarRole;
}

export interface AdminAddonSubtitleImportApplyResponse {
  selected_ref: AdminAddonSubtitleSelectedReference;
  candidate: AdminAddonSubtitleCandidateSummary;
  plan: AdminSubtitleImportPlan;
  apply: AdminSubtitleImportApplyReport;
}

export interface AdminAddonResourceLinkCheckRequest {
  refresh?: boolean;
}

export interface AddonAcquisitionCandidateSummary {
  id: string;
  target_library_id: string;
  state: string;
  source_kind: string;
  source_scheme: string | null;
  source_ref_redacted: string;
  source_key_fingerprint: string;
  has_display_name: boolean;
  has_intended_locator: boolean;
  size_bytes: number | null;
  has_fingerprint: boolean;
  has_diagnostics: boolean;
  managed_import_artifact_id: string | null;
  writes_library: boolean;
  creates_media_source: boolean;
  creates_managed_import: boolean;
  promotion_apply: boolean;
}

export interface AdminAddonResourceSearchSelectionResponse {
  addon_id: string;
  manifest_id: string;
  search_id: string;
  selection_id: string;
  candidate: AddonAcquisitionCandidateSummary;
  idempotent_replay: boolean;
}

export interface AdminAddonResourceLinkCheckResponse {
  addon_id: string;
  manifest_id: string;
  search_id: string;
  selection_id: string;
  status: AdminAddonResourceCallDiagnosticStatus;
  latency_ms: number;
  attempts: number;
  link_type: AddonResourceLinkType;
  check_status?: AddonResourceLinkCheckStatus;
  checked_at_ms?: number;
  requires_password?: boolean;
  retryable?: boolean;
  retry_after_ms?: number;
  has_safe_message: boolean;
  safe_facts?: Record<string, string>;
  http_status?: number;
  safe_error_code?: string;
}

export interface AdminAddonInstallGuideResponse {
  addon_id: string;
  manifest_id: string;
  addon_name: string;
  addon_version: string;
  protocol_version: string;
  base_url: string;
  status: AddonStatus;
  docker_compose: AdminAddonInstallGuideSnippet;
  systemd: AdminAddonInstallGuideSnippet;
  secret_references: AdminAddonInstallGuideSecretReference[];
  health_check_steps: AdminAddonInstallGuideStep[];
  registration_verification_steps: AdminAddonInstallGuideStep[];
  lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary;
}

export interface AdminAddonManagerPlanResponse {
  addon_id: string;
  intent?: AdminAddonLifecycleIntent;
  operator_confirmed: boolean;
  source: AdminAddonRegistrationDetail;
  health_check: AdminAddonHealthCheckResponse;
  tokens: AddonTokensResponse;
  grants: AddonGrantsResponse;
  install_guide: AdminAddonInstallGuideResponse;
}

export interface AdminAddonInstallGuideSnippet {
  title: string;
  filename: string;
  content: string;
  notes: string[];
}

export interface AdminAddonInstallGuideSecretReference {
  id: string;
  label: string;
  description?: string;
  required: boolean;
  env_var: string;
  placeholder: string;
}

export interface AdminAddonInstallGuideStep {
  title: string;
  command: string;
  expected_result: string;
}

export interface AdminAddonInstallGuideLifecycleBoundary {
  nako_manages_containers: boolean;
  nako_manages_processes: boolean;
  nako_manages_packages: boolean;
  message: string;
}

export interface IssueAddonTokenRequest {
  label?: string;
}

export interface AddonTokenSummary {
  id: string;
  addon_id: string;
  label: string;
  token_prefix: string;
  status: AddonTokenStatus;
  created_at: string;
  rotated_at: string | null;
  revoked_at: string | null;
  last_used_at: string | null;
}

export interface AddonTokensResponse {
  tokens: AddonTokenSummary[];
}

export interface AddonTokenResponse {
  token: AddonTokenSummary;
}

export interface AddonTokenIssuedResponse {
  token: AddonTokenSummary;
  raw_token: string;
}

export interface AddonTokenRotationResponse {
  rotated: AddonTokenSummary;
  token: AddonTokenSummary;
  raw_token: string;
}

export interface ReplaceAddonGrantsRequest {
  grants?: AddonGrantAssignment[];
}

export interface AddonGrantAssignment {
  permission: AddonPermission;
  library_id?: string | null;
}

export interface AddonGrantRecord {
  id: string;
  addon_id: string;
  permission: AddonPermission;
  library_id: string | null;
  created_at: string;
}

export interface AddonGrantsResponse {
  grants: AddonGrantRecord[];
}

export interface PageInfo {
  limit: number;
  offset: number;
  returned: number;
}

export type AdminOverviewStatus = "healthy" | "degraded";

export type AdminWatchFolderRuntimeCoverageStatus = "started" | "disabled" | "unsupported_root" | "missing_root";

export interface AdminOverviewSourceFingerprintHashSummary {
  total_sources: number;
  fingerprinted_sources: number;
  content_hash_sources: number;
  queued_jobs: number;
  running_jobs: number;
  succeeded_jobs: number;
  failed_jobs: number;
  cancelled_jobs: number;
  claimable_jobs: number;
  delayed_retry_jobs: number;
  oldest_queued_at: string | null;
  next_retry_at: string | null;
}

export interface AdminOverviewResponse {
  admin_api_version: string;
  public_api_version: string;
  status: AdminOverviewStatus;
  storage: {
    total_backends: number;
    ready_backends: number;
    degraded_backends: number;
    unavailable_backends: number;
    backends: Array<{
      library_id: string;
      library_name: string;
      backend_kind: string;
      status: string;
    }>;
  };
  catalog: {
    governed_items: number;
    unknown_kind_items: number;
    low_confidence_items: number;
    items_with_duplicate_relationships: number;
    items_missing_accepted_provider_mapping: number;
  };
  metadata: {
    total_providers: number;
    available_providers: number;
    disabled_providers: number;
    unavailable_providers: number;
    providers: Array<{
      provider: string;
      status: string;
    }>;
  };
  runtime: {
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    succeeded_jobs: number;
    cancelled_jobs: number;
    failed_jobs: number;
    shutdown_requested: boolean;
  };
  source_fingerprint_hash: AdminOverviewSourceFingerprintHashSummary;
  startup: {
    configured_libraries: number;
    recovered_transcode_sessions: number;
    recovered_jobs: number;
    staging_deleted_records: number;
    staging_deleted_files: number;
    metadata_raw_cache_deleted: number;
    metadata_lifecycle_tasks_started: number;
    artwork_ingest_worker_started: boolean;
    addon_event_scheduler_started: boolean;
    watch_folder_runtimes_started: number;
    watch_folder_runtime: {
      configured_libraries: number;
      realtime_enabled_libraries: number;
      started_libraries: number;
      skipped_libraries: number;
      diagnostics: Array<{
        library_id: string;
        library_name: string;
        root_scheme: string | null;
        root_ref_redacted: string;
        status: AdminWatchFolderRuntimeCoverageStatus;
        safe_reason: string;
      }>;
    };
  };
}

export interface AdminCatalogGovernanceItemListResponse {
  items: AdminCatalogGovernanceItem[];
  page: PageInfo;
}

export interface AdminCatalogGovernanceItem {
  item_id: string;
  library_id: string;
  kind: string;
  parent_id: string | null;
  title: string;
  release_date: string | null;
  source_count: number;
  representative_source_id: string | null;
  representative_file_name: string | null;
  local_inference: AdminLocalInferenceSummary | null;
  provider_mapping_count: number;
  accepted_provider_mapping_count: number;
  duplicate_relationship_count: number;
  issues: string[];
}

export interface AdminCatalogGovernanceItemDetailResponse {
  admin_api_version: string;
  public_api_version: string;
  item: AdminCatalogGovernanceItem;
  provider_mappings: AdminCatalogGovernanceProviderMappingSummary[];
  repair_actions: AdminCatalogGovernanceRepairAction[];
}

export interface AdminCatalogGovernanceProviderMappingSummary {
  mapping_id: string;
  item_id: string;
  status: AdminProviderMappingStatus;
  confidence_milli: number | null;
  source: AdminMetadataSource;
  subject: AdminCatalogGovernanceProviderSubjectSummary;
}

export interface AdminCatalogGovernanceProviderSubjectSummary {
  subject_id: string;
  provider: AdminExternalProvider;
  subject_kind: AdminProviderSubjectKind;
  subject_key: string;
  title: string | null;
  release_year: number | null;
  locale: string | null;
}

export type AdminCatalogGovernanceRepairAction = "provider_mapping_review";

export type AdminCatalogGovernanceProviderMappingReviewDecision = "accept" | "reject";

export type AdminProviderMappingStatus = "candidate" | "accepted" | "rejected";

export type AdminExternalProvider =
  | "tmdb"
  | "douban"
  | "bangumi"
  | "imdb"
  | "local"
  | { other: string };

export type AdminProviderSubjectKind =
  | "movie"
  | "series"
  | "season"
  | "episode"
  | "collection"
  | "subject"
  | "person"
  | { other: string };

export type AdminMetadataSource =
  | "local"
  | "nfo"
  | { provider: AdminExternalProvider }
  | "user"
  | { addon: string };

export type AdminMetadataCandidateReviewStatus =
  | "pending"
  | "accepted"
  | "rejected"
  | "superseded"
  | "expired";

export interface AdminMetadataCandidateReviewQueueQuery extends AdminPageQuery {
  status?: AdminMetadataCandidateReviewStatus;
  provider?: AdminExternalProvider;
}

export type AdminMetadataCandidateReviewApplicationAction =
  | "apply"
  | "skip"
  | "noop";

export type AdminMetadataCandidateReviewApplicationReason =
  | "review_not_accepted"
  | "missing_root_subject"
  | "unsupported_source"
  | "existing_accepted_mapping"
  | "existing_candidate_mapping"
  | "existing_rejected_mapping"
  | "ready";

export type AdminMetadataCandidateReviewMediaKind =
  | "movie"
  | "series"
  | "season"
  | "episode"
  | "collection"
  | "extra"
  | "unknown";

export type AdminMetadataCandidateReviewSource =
  | "local"
  | "nfo"
  | { provider: AdminExternalProvider }
  | { addon: string }
  | { automation: string }
  | "user"
  | { other: string };

export type AdminMetadataCandidateReviewRelationshipKind =
  | "contains"
  | "belongs_to"
  | "related";

export interface AdminMetadataCandidateSubject {
  provider: AdminExternalProvider;
  subject_kind: AdminProviderSubjectKind;
  subject_key: string;
  title: string | null;
  release_year: number | null;
  locale: string | null;
}

export interface AdminMetadataCandidateReviewMetadataSummary {
  title: string | null;
  original_title: string | null;
  sort_title: string | null;
  release_date: string | null;
  runtime_minutes: number | null;
  description_present: boolean;
  tagline_present: boolean;
  genre_count: number;
  tag_count: number;
  rating_count: number;
  image_count: number;
  credit_count: number;
  collection_count: number;
  studio_count: number;
  external_id_count: number;
}

export interface AdminMetadataCandidateReviewNode {
  source: AdminMetadataCandidateReviewSource;
  kind: AdminMetadataCandidateReviewMediaKind;
  subject: AdminMetadataCandidateSubject | null;
  metadata: AdminMetadataCandidateReviewMetadataSummary;
}

export interface AdminMetadataCandidateReviewRelationship {
  parent_subject: AdminMetadataCandidateSubject;
  child_subject: AdminMetadataCandidateSubject;
  kind: AdminMetadataCandidateReviewRelationshipKind;
}

export interface AdminMetadataCandidateReviewDetail {
  review_id: string;
  item_id: string;
  source: AdminMetadataCandidateReviewSource;
  source_key: string;
  status: AdminMetadataCandidateReviewStatus;
  root: AdminMetadataCandidateReviewNode;
  related: AdminMetadataCandidateReviewNode[];
  relationships: AdminMetadataCandidateReviewRelationship[];
  related_count: number;
  relationship_count: number;
  expires_at_ms: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminMetadataCandidateReviewApplicationPlan {
  review_id: string;
  item_id: string;
  action: AdminMetadataCandidateReviewApplicationAction;
  reasons: AdminMetadataCandidateReviewApplicationReason[];
  source: AdminMetadataSource | null;
  root_subject: AdminMetadataCandidateSubject | null;
  existing_mapping_id: string | null;
  existing_mapping_status: AdminProviderMappingStatus | null;
}

export interface AdminMetadataCandidateReviewApplicationBoundary {
  read_only: boolean;
  applies_on_read: boolean;
  apply_mutation_required: boolean;
  apply_updates_root_provider_subject: boolean;
  apply_updates_root_provider_mapping: boolean;
  apply_updates_related_provider_subjects: boolean;
  apply_updates_related_provider_mappings: boolean;
  updates_canonical_metadata: boolean;
  updates_hierarchy: boolean;
  writes_nfo: boolean;
  writes_library_files: boolean;
}

export interface AdminMetadataCandidateReviewApplyRequest {
  item_id: string;
  expected_updated_at_ms: number | null;
  idempotency_key: string;
}

export interface AdminMetadataCandidateReviewProviderSubject {
  subject_id: string;
  provider: AdminExternalProvider;
  subject_kind: AdminProviderSubjectKind;
  subject_key: string;
  title: string | null;
  release_year: number | null;
  locale: string | null;
}

export interface AdminMetadataCandidateReviewProviderMapping {
  mapping_id: string;
  item_id: string;
  subject_id: string;
  status: AdminProviderMappingStatus;
  confidence_milli: number | null;
  source: AdminMetadataSource;
}

export interface AdminMetadataCandidateReviewGovernance {
  audit_timeline: AdminMetadataCandidateReviewAuditTimeline;
  undo_plan: AdminMetadataCandidateReviewUndoPlan;
}

export interface AdminMetadataCandidateReviewAuditTimeline {
  read_only: boolean;
  replay_safe: boolean;
  events: AdminMetadataCandidateReviewAuditEvent[];
}

export type AdminMetadataCandidateReviewAuditEventKind =
  | "review_created"
  | "review_status_current"
  | "application_plan_read"
  | "application_result"
  | "batch_item_status";

export interface AdminMetadataCandidateReviewAuditEvent {
  kind: AdminMetadataCandidateReviewAuditEventKind;
  at_ms: number | null;
  status: AdminMetadataCandidateReviewStatus | null;
  batch_item_status: string | null;
  action: AdminMetadataCandidateReviewApplicationAction | null;
  changed: boolean | null;
  idempotent_replay: boolean | null;
  provider_mapping_id: string | null;
}

export type AdminMetadataCandidateReviewUndoMode =
  | "no_mutation_observed"
  | "deferred_until_apply_outcome_audit"
  | "manual_root_provider_mapping_review";

export type AdminMetadataCandidateReviewUndoReason =
  | "read_only_trust_slice"
  | "no_provider_mapping_mutation_observed"
  | "apply_outcome_audit_required"
  | "missing_persisted_pre_apply_snapshot"
  | "provider_mapping_may_preexist_review"
  | "root_only_provider_mapping_boundary"
  | "related_hierarchy_undo_deferred"
  | "public_client_contract_unchanged"
  | "stale_state_guard_required";

export interface AdminMetadataCandidateReviewUndoPlan {
  read_only: boolean;
  undo_mutation_available: boolean;
  replay_safe: boolean;
  stale_state_guard_updated_at_ms: number | null;
  target_mapping_id: string | null;
  target_mapping_status: AdminProviderMappingStatus | null;
  mode: AdminMetadataCandidateReviewUndoMode;
  reasons: AdminMetadataCandidateReviewUndoReason[];
}

export interface AdminMetadataCandidateReviewListEntry {
  review_id: string;
  item_id: string;
  source: AdminMetadataCandidateReviewSource;
  source_key: string;
  status: AdminMetadataCandidateReviewStatus;
  root: AdminMetadataCandidateReviewNode;
  related_count: number;
  relationship_count: number;
  application_plan: AdminMetadataCandidateReviewApplicationPlan;
  boundary: AdminMetadataCandidateReviewApplicationBoundary;
  governance: AdminMetadataCandidateReviewGovernance;
  expires_at_ms: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminMetadataCandidateReviewListResponse {
  admin_api_version: string;
  public_api_version: string;
  item_id: string;
  reviews: AdminMetadataCandidateReviewListEntry[];
  page: PageInfo;
}

export interface AdminMetadataCandidateReviewQueueResponse {
  admin_api_version: string;
  public_api_version: string;
  reviews: AdminMetadataCandidateReviewListEntry[];
  page: PageInfo;
}

export interface AdminMetadataCandidateReviewBatchPlanRequest {
  review_ids: string[];
}

export interface AdminMetadataCandidateReviewBatchPlanSummary {
  requested_count: number;
  returned_count: number;
  max_review_count: number;
  apply_count: number;
  noop_count: number;
  skip_count: number;
}

export interface AdminMetadataCandidateReviewBatchPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  summary: AdminMetadataCandidateReviewBatchPlanSummary;
  reviews: AdminMetadataCandidateReviewListEntry[];
}

export interface AdminMetadataCandidateReviewBatchApplyItemRequest {
  review_id: string;
  item_id: string;
  expected_updated_at_ms: number | null;
}

export interface AdminMetadataCandidateReviewBatchApplyRequest {
  idempotency_key: string;
  reviews: AdminMetadataCandidateReviewBatchApplyItemRequest[];
}

export interface AdminMetadataCandidateReviewBatchCreateRequest {
  idempotency_key: string;
  reviews: AdminMetadataCandidateReviewBatchApplyItemRequest[];
}

export interface AdminMetadataCandidateReviewBatchResponse {
  admin_api_version: string;
  public_api_version: string;
  batch: AdminMetadataCandidateReviewBatch;
}

export interface AdminMetadataCandidateReviewBatch {
  id: string;
  job_id: string;
  status: string;
  idempotency_key_fingerprint: string;
  selection: AdminMetadataCandidateReviewBatchPlanSelection;
  summary: AdminMetadataCandidateReviewBatchPlanSummary;
  execution_summary: AdminMetadataCandidateReviewBatchExecutionSummary;
  items: AdminMetadataCandidateReviewBatchItem[];
  created_at: string;
  updated_at: string;
}

export interface AdminMetadataCandidateReviewBatchPlanSelection {
  requested_review_count: number;
  selected_review_count: number;
  duplicate_review_count: number;
  max_review_count: number;
}

export interface AdminMetadataCandidateReviewBatchExecutionSummary {
  total_item_count: number;
  pending_item_count: number;
  skipped_item_count: number;
  blocked_item_count: number;
  applied_item_count: number;
  noop_item_count: number;
  stale_item_count: number;
  conflict_item_count: number;
  failed_item_count: number;
}

export interface AdminMetadataCandidateReviewBatchItem {
  review_id: string;
  item_id: string;
  position: number;
  status: string;
  idempotency_key_fingerprint: string;
  expected_updated_at_ms: number | null;
  provider_subject_id: string | null;
  provider_mapping_id: string | null;
  error: AdminMetadataCandidateReviewBatchApplyError | null;
  plan: AdminMetadataCandidateReviewApplicationPlan;
  boundary: AdminMetadataCandidateReviewApplicationBoundary;
  governance: AdminMetadataCandidateReviewGovernance;
  created_at: string;
  updated_at: string;
}

export type AdminMetadataCandidateReviewBatchApplyResultStatus =
  | "applied"
  | "noop"
  | "replayed"
  | "skipped"
  | "blocked"
  | "stale"
  | "conflict"
  | "failed";

export interface AdminMetadataCandidateReviewBatchApplySummary {
  requested_count: number;
  returned_count: number;
  max_review_count: number;
  applied_count: number;
  changed_count: number;
  noop_count: number;
  replay_count: number;
  skipped_count: number;
  blocked_count: number;
  stale_count: number;
  conflict_count: number;
  failed_count: number;
}

export interface AdminMetadataCandidateReviewBatchApplyError {
  code: string;
  message: string;
}

export interface AdminMetadataCandidateReviewBatchApplyResult {
  review_id: string;
  item_id: string;
  status: AdminMetadataCandidateReviewBatchApplyResultStatus;
  applied: boolean;
  changed: boolean;
  idempotent_replay: boolean;
  idempotency_key_fingerprint: string;
  plan: AdminMetadataCandidateReviewApplicationPlan | null;
  provider_subject: AdminMetadataCandidateReviewProviderSubject | null;
  provider_mapping: AdminMetadataCandidateReviewProviderMapping | null;
  boundary: AdminMetadataCandidateReviewApplicationBoundary | null;
  governance: AdminMetadataCandidateReviewGovernance | null;
  error: AdminMetadataCandidateReviewBatchApplyError | null;
}

export interface AdminMetadataCandidateReviewBatchApplyResponse {
  admin_api_version: string;
  public_api_version: string;
  idempotency_key_fingerprint: string;
  summary: AdminMetadataCandidateReviewBatchApplySummary;
  results: AdminMetadataCandidateReviewBatchApplyResult[];
}

export interface AdminMetadataCandidateReviewResponse {
  admin_api_version: string;
  public_api_version: string;
  review: AdminMetadataCandidateReviewDetail;
  application_plan: AdminMetadataCandidateReviewApplicationPlan;
  boundary: AdminMetadataCandidateReviewApplicationBoundary;
  governance: AdminMetadataCandidateReviewGovernance;
}

export interface AdminMetadataCandidateReviewApplyResponse {
  admin_api_version: string;
  public_api_version: string;
  review_id: string;
  item_id: string;
  applied: boolean;
  changed: boolean;
  idempotent_replay: boolean;
  idempotency_key_fingerprint: string;
  plan: AdminMetadataCandidateReviewApplicationPlan;
  provider_subject: AdminMetadataCandidateReviewProviderSubject | null;
  provider_mapping: AdminMetadataCandidateReviewProviderMapping | null;
  boundary: AdminMetadataCandidateReviewApplicationBoundary;
  governance: AdminMetadataCandidateReviewGovernance;
}

export interface AdminMetadataCandidateReviewRelatedHierarchyPlanRequest {
  item_id: string;
  expected_updated_at_ms: number | null;
}

export interface AdminMetadataCandidateReviewRelatedHierarchyPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  review_id: string;
  item_id: string;
  plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan;
  boundary: AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary;
}

export interface AdminMetadataCandidateReviewRelatedHierarchyApplyRequest {
  item_id: string;
  expected_updated_at_ms: number | null;
  idempotency_key: string;
}

export interface AdminMetadataCandidateReviewRelatedHierarchyApplyResponse {
  admin_api_version: string;
  public_api_version: string;
  review_id: string;
  item_id: string;
  applied: boolean;
  changed: boolean;
  idempotent_replay: boolean;
  idempotency_key_fingerprint: string;
  plan: AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan;
  provider_subjects: AdminMetadataCandidateReviewProviderSubject[];
  provider_mappings: AdminMetadataCandidateReviewProviderMapping[];
  confirmed_item_ids: string[];
  boundary: AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary;
}

export type AdminMetadataCandidateReviewRelatedHierarchyApplicationAction =
  | "apply"
  | "skip"
  | "noop";

export type AdminMetadataCandidateReviewRelatedHierarchyApplicationReason =
  | "review_not_accepted"
  | "missing_root_subject"
  | "unsupported_source"
  | "missing_accepted_root_mapping"
  | "no_safe_related_hierarchy_relationships"
  | "ready"
  | "already_applied";

export interface AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan {
  review_id: string;
  item_id: string;
  action: AdminMetadataCandidateReviewRelatedHierarchyApplicationAction;
  reasons: AdminMetadataCandidateReviewRelatedHierarchyApplicationReason[];
  source: AdminMetadataSource | null;
  root_subject: AdminMetadataCandidateSubject | null;
  root_mapping_id: string | null;
  root_mapping_status: AdminProviderMappingStatus | null;
  target_count: number;
  mapping_change_count: number;
  provisional_state_change_count: number;
  targets: AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget[];
}

export interface AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget {
  item_id: string;
  library_ids: string[];
  subject: AdminMetadataCandidateSubject;
  source: AdminMetadataSource;
  existing_subject_id: string | null;
  existing_mapping_id: string | null;
  existing_mapping_status: AdminProviderMappingStatus | null;
  mapping_change_required: boolean;
  provisional_library_state_count: number;
}

export interface AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary {
  read_only: boolean;
  applies_on_read: boolean;
  apply_mutation_required: boolean;
  apply_updates_root_provider_subject: boolean;
  apply_updates_root_provider_mapping: boolean;
  apply_updates_related_provider_subjects: boolean;
  apply_updates_related_provider_mappings: boolean;
  apply_confirms_related_library_item_state: boolean;
  updates_parent_hierarchy: boolean;
  updates_canonical_metadata: boolean;
  writes_nfo: boolean;
  writes_library_files: boolean;
}

export interface AdminCatalogGovernanceProviderMappingReviewRequest {
  decision: AdminCatalogGovernanceProviderMappingReviewDecision;
}

export interface AdminCatalogGovernanceProviderMappingReviewPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  plan: AdminCatalogGovernanceProviderMappingReviewPlan;
}

export interface AdminCatalogGovernanceProviderMappingReviewResponse {
  admin_api_version: string;
  public_api_version: string;
  item_id: string;
  mapping_id: string;
  decision: AdminCatalogGovernanceProviderMappingReviewDecision;
  previous_status: AdminProviderMappingStatus;
  current_status: AdminProviderMappingStatus;
  changed: boolean;
  idempotent_replay: boolean;
  plan: AdminCatalogGovernanceProviderMappingReviewPlan;
}

export interface AdminCatalogGovernanceProviderMappingReviewPlan {
  item: AdminCatalogGovernanceItem;
  mapping: AdminCatalogGovernanceProviderMappingSummary;
  decision: AdminCatalogGovernanceProviderMappingReviewDecision;
  current_status: AdminProviderMappingStatus;
  target_status: AdminProviderMappingStatus;
  status: AdminCatalogGovernanceRepairPlanStatus;
  readiness: AdminCatalogGovernanceRepairReadiness;
  boundary: AdminCatalogGovernanceRepairBoundary;
}

export type AdminCatalogGovernanceRepairPlanStatus = "ready" | "blocked";

export type AdminCatalogGovernanceRepairReason =
  | "provider_mapping_status_change"
  | "already_in_target_status";

export interface AdminCatalogGovernanceRepairReadiness {
  status: AdminCatalogGovernanceRepairPlanStatus;
  actionable: boolean;
  reasons: AdminCatalogGovernanceRepairReason[];
}

export interface AdminCatalogGovernanceRepairBoundary {
  updates_provider_mapping_status: boolean;
  updates_canonical_metadata: boolean;
  updates_provider_subject: boolean;
  updates_local_inference: boolean;
  updates_source_duplicates: boolean;
  updates_hierarchy: boolean;
  writes_nfo: boolean;
  writes_library_files: boolean;
  updates_artwork: boolean;
  updates_playback_state: boolean;
}

export interface AdminLocalInferenceSummary {
  source_id: string;
  inferred_kind: string;
  inferred_title: string | null;
  inferred_year: number | null;
  inferred_season: number | null;
  inferred_episode: number | null;
  confidence_milli: number | null;
  evidence_source: string;
  has_evidence: boolean;
  inference_version: string;
}

export interface AdminOutboxEventListResponse {
  events: AdminOutboxEventListItem[];
  page: PageInfo;
}

export interface AdminOutboxEventListItem {
  id: string;
  kind: string;
  subject: string | Record<string, unknown>;
  library_id: string | null;
  source_id: string | null;
  status: string;
  attempts: number;
  has_payload: boolean;
  has_error: boolean;
  occurred_at: string;
  updated_at: string;
  next_attempt_at: string | null;
}

export interface AdminJobDiagnostics {
  vfs_cache_repair: AdminVfsCacheRepairJobDiagnostics | null;
}

export type AdminVfsCacheRepairJobDiagnosticStatus =
  | "pending"
  | "summary_available"
  | "failed";

export interface AdminVfsCacheRepairJobDiagnostics {
  status: AdminVfsCacheRepairJobDiagnosticStatus;
  summary: AdminVfsCacheRepairJobSummary | null;
  failure: AdminVfsCacheRepairJobFailureDiagnostic | null;
}

export interface AdminVfsCacheRepairJobFailureDiagnostic {
  status: string;
  failure_class: StorageFailureClass;
  safe_message: string;
  retryable: boolean;
}

export interface AdminJobListItem {
  id: string;
  kind: string;
  status: string;
  resource_class: string;
  library_id: string | null;
  source_id: string | null;
  has_input: boolean;
  has_summary: boolean;
  has_error: boolean;
  queued_at: string;
  started_at: string | null;
  completed_at: string | null;
  diagnostics?: AdminJobDiagnostics | null;
}

export type AdminJobCommandResponse = AdminJobListItem;

export interface AdminJobListResponse {
  jobs: AdminJobListItem[];
  page: PageInfo;
}

export interface AdminPlaybackSessionListItem {
  id: string;
  principal_id: string;
  source_id: string;
  item_id: string;
  mode: string;
  state: string;
  transcode_session_id: string | null;
  has_client_capabilities: boolean;
  active: boolean;
  terminal: boolean;
  created_at: string;
  updated_at: string;
  started_at_ms: number;
  ended_at_ms: number | null;
  last_heartbeat_at_ms: number | null;
}

export interface AdminPlaybackSessionListResponse {
  sessions: AdminPlaybackSessionListItem[];
  page: PageInfo;
}

export interface AdminAcquisitionIntakeCandidateListResponse {
  admin_api_version: string;
  public_api_version: string;
  candidates: AdminAcquisitionIntakeCandidateDiagnostic[];
  page: PageInfo;
}

export interface AdminAcquisitionIntakeCandidateDiagnostic {
  id: string;
  target_library_id: string;
  source_kind: string;
  custom_source_kind: boolean;
  source_scheme: string | null;
  source_ref_redacted: string;
  source_key_fingerprint: string;
  has_display_name: boolean;
  has_intended_locator: boolean;
  size_bytes: number | null;
  has_fingerprint: boolean;
  managed_import_artifact_id: string | null;
  state: string;
  has_diagnostics: boolean;
  first_seen_at_ms: number;
  last_seen_at_ms: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminWatchFolderDiscoveryResponse {
  admin_api_version: string;
  public_api_version: string;
  target_library_id: string;
  root_scheme: string | null;
  root_ref_redacted: string;
  ready_candidates: number;
  inspecting_candidates: number;
  blocked_candidates: number;
  incomplete_candidates: number;
  unsupported_candidates: number;
  suppressed_candidates: number;
  recorded_candidates: number;
  newly_ready_candidates: number;
  active_suppressions: AdminWatchFolderSuppression[];
  failures: Array<{
    ref_redacted: string;
    safe_message: string;
  }>;
  writes_library: boolean;
  managed_import_artifacts_created: boolean;
  promotion_apply: boolean;
}

export interface AdminGeneratedArtifactProposalListResponse {
  admin_api_version: string;
  public_api_version: string;
  proposals: AdminGeneratedArtifactProposal[];
  page: PageInfo;
}

export interface AdminGeneratedArtifactMetadataApplyOutcomeListResponse {
  admin_api_version: string;
  public_api_version: string;
  outcomes: AdminGeneratedArtifactMetadataApplyOutcome[];
  page: PageInfo;
}

export interface AdminGeneratedArtifactMetadataApplyOutcomeResponse {
  admin_api_version: string;
  public_api_version: string;
  outcome: AdminGeneratedArtifactMetadataApplyOutcome;
}

export interface AdminGeneratedArtifactMetadataApplyRecoveryResponse {
  admin_api_version: string;
  public_api_version: string;
  summary: AdminGeneratedArtifactMetadataApplyRecoverySummary;
  entries: AdminGeneratedArtifactMetadataApplyRecoveryEntry[];
  page: PageInfo;
}

export interface AdminGeneratedArtifactProposal {
  id: string;
  kind: string;
  capability: string;
  status: string;
  target: AdminGeneratedArtifactTarget;
  provenance: AdminGeneratedArtifactProvenance;
  payload: AdminGeneratedArtifactPayloadSummary;
  readiness: AdminGeneratedArtifactReadiness;
  created_at: string;
  updated_at: string;
  accepted_at: string | null;
}

export interface AdminGeneratedArtifactTarget {
  kind: string;
  library_id: string | null;
  item_id: string | null;
  source_id: string | null;
}

export interface AdminGeneratedArtifactProvenance {
  provider_id: string;
  provider_name: string | null;
  job_id: string;
  capability: string;
  idempotency_key_fingerprint: string | null;
  prompt_fingerprint: string | null;
  attempt_count: number | null;
  artifact_created_at: string;
}

export interface AdminGeneratedArtifactPayloadSummary {
  valid_json: boolean;
  shape: string;
  payload_fingerprint: string;
  payload_bytes: number;
  object_field_count: number | null;
  array_item_count: number | null;
  has_textual_values: boolean;
  has_explanation: boolean;
  confidence_milli: number | null;
}

export interface AdminGeneratedArtifactReadiness {
  status: string;
  actionable: boolean;
  reasons: string[];
}

export interface AdminGeneratedArtifactReviewPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  plan: AdminGeneratedArtifactAcceptancePlan;
}

export interface AdminGeneratedArtifactReviewResponse {
  admin_api_version: string;
  public_api_version: string;
  artifact_id: string;
  decision: "accept" | "reject";
  artifact_status: string;
  accepted_at: string | null;
  idempotent_replay: boolean;
  plan: AdminGeneratedArtifactAcceptancePlan;
}

export interface AdminGeneratedArtifactAcceptancePlan {
  artifact_id: string;
  decision: "accept" | "reject";
  status: string;
  action: string;
  reasons: string[];
  capability: string;
  kind: string;
  target: AdminGeneratedArtifactTarget;
  payload: AdminGeneratedArtifactPayloadSummary;
  readiness: AdminGeneratedArtifactReadiness;
  boundary: {
    accepted_into_canonical_metadata: boolean;
    writes_sidecar: boolean;
    writes_library_files: boolean;
    applies_immediately: boolean;
    requires_metadata_authority_apply: boolean;
  };
}

export interface AdminGeneratedArtifactMetadataApplyPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  plan: AdminGeneratedArtifactMetadataApplyPlan;
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  plan: AdminGeneratedArtifactMetadataBulkApplyPlan;
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchResponse {
  admin_api_version: string;
  public_api_version: string;
  batch: AdminGeneratedArtifactMetadataBulkApplyBatch;
}

export interface AdminGeneratedArtifactMetadataApplyResponse {
  admin_api_version: string;
  public_api_version: string;
  outcome_id: string | null;
  artifact_id: string;
  status: string;
  applied: boolean;
  changed: boolean;
  idempotent_replay: boolean;
  applied_source: string | null;
  plan: AdminGeneratedArtifactMetadataApplyPlan;
}

export interface AdminGeneratedArtifactMetadataApplyOutcome {
  id: string;
  artifact_id: string;
  idempotency_key_fingerprint: string;
  status: string;
  applied: boolean;
  changed: boolean;
  applied_source: string | null;
  item_id: string | null;
  plan: AdminGeneratedArtifactMetadataApplyPlan;
  error_code: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminGeneratedArtifactMetadataApplyRecoverySummary {
  returned_entry_count: number;
  needs_repair_count: number;
  needs_review_count: number;
  replay_only_count: number;
  resolved_count: number;
}

export interface AdminGeneratedArtifactMetadataApplyRecoveryEntry {
  source: "apply_outcome" | "bulk_batch_item";
  attention: AdminGeneratedArtifactApplyRecoveryAttention;
  reason:
    | "apply_outcome_applied"
    | "apply_outcome_noop"
    | "apply_outcome_failed"
    | "bulk_batch_item_applied"
    | "bulk_batch_item_noop"
    | "bulk_batch_item_stale"
    | "bulk_batch_item_failed"
    | "bulk_batch_item_skipped";
  artifact_id: string;
  outcome_id: string | null;
  batch_id: string | null;
  batch_item_status: string | null;
  outcome_status: string | null;
  item_id: string | null;
  plan: AdminGeneratedArtifactMetadataApplyPlan | null;
  error_code: string | null;
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminGeneratedArtifactMetadataApplyPlan {
  artifact_id: string;
  status: string;
  executable: boolean;
  reasons: string[];
  target: AdminGeneratedArtifactTarget;
  payload: AdminGeneratedArtifactPayloadSummary;
  fields: AdminGeneratedArtifactMetadataApplyFieldPlan[];
  provider_mappings: AdminGeneratedArtifactProviderMappingPlan[];
  apply_field_count: number;
  skipped_field_count: number;
  noop_field_count: number;
  apply_provider_mapping_count: number;
  skipped_provider_mapping_count: number;
  noop_provider_mapping_count: number;
}

export interface AdminGeneratedArtifactMetadataApplyFieldPlan {
  field: string;
  action: string;
  reasons: string[];
  current: AdminGeneratedArtifactMetadataValueSummary;
  incoming: AdminGeneratedArtifactMetadataValueSummary;
}

export interface AdminGeneratedArtifactProviderMappingPlan {
  subject: AdminGeneratedArtifactProviderSubjectPlan;
  action: string;
  reasons: string[];
  confidence_milli: number | null;
  existing_mapping_status: string | null;
}

export interface AdminGeneratedArtifactProviderSubjectPlan {
  provider: string | null;
  provider_name: string | null;
  subject_kind: string | null;
  subject_kind_name: string | null;
  subject_key: string | null;
  title: string | null;
  release_year: number | null;
  locale: string | null;
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlan {
  selection: AdminGeneratedArtifactMetadataBulkApplyPlanSelection;
  summary: AdminGeneratedArtifactMetadataBulkApplyPlanSummary;
  items: AdminGeneratedArtifactMetadataBulkApplyPlanItem[];
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanSelection {
  requested_artifact_count: number;
  selected_artifact_count: number;
  duplicate_artifact_count: number;
  max_artifact_count: number;
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanSummary {
  planned_artifact_count: number;
  missing_artifact_count: number;
  ready_artifact_count: number;
  blocked_artifact_count: number;
  stale_artifact_count: number;
  executable_artifact_count: number;
  apply_field_count: number;
  skipped_field_count: number;
  noop_field_count: number;
  apply_provider_mapping_count: number;
  skipped_provider_mapping_count: number;
  noop_provider_mapping_count: number;
}

export interface AdminGeneratedArtifactMetadataBulkApplyPlanItem {
  artifact_id: string;
  status: string;
  executable: boolean;
  reasons: string[];
  plan: AdminGeneratedArtifactMetadataApplyPlan | null;
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatch {
  id: string;
  job_id: string;
  status: string;
  selection: AdminGeneratedArtifactMetadataBulkApplyPlanSelection;
  summary: AdminGeneratedArtifactMetadataBulkApplyPlanSummary;
  execution_summary: AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummary;
  items: AdminGeneratedArtifactMetadataBulkApplyBatchItem[];
  created_at: string;
  updated_at: string;
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummary {
  total_item_count: number;
  pending_item_count: number;
  skipped_item_count: number;
  applied_item_count: number;
  noop_item_count: number;
  stale_item_count: number;
  failed_item_count: number;
}

export interface AdminGeneratedArtifactMetadataBulkApplyBatchItem {
  artifact_id: string;
  position: number;
  status: string;
  outcome_id: string | null;
  error_code: string | null;
  error_message: string | null;
  plan_item: AdminGeneratedArtifactMetadataBulkApplyPlanItem;
  created_at: string;
  updated_at: string;
}

export interface AdminGeneratedArtifactMetadataValueSummary {
  present: boolean;
  empty: boolean;
  value_fingerprint: string | null;
  value_bytes: number | null;
  item_count: number | null;
}

export type AdminPlaybackPolicyRoleMergeStrategy = "restrictive";

export type AdminPlaybackPolicyPermission =
  | "media_playback"
  | "direct_play"
  | "remux"
  | "audio_transcode"
  | "video_transcode"
  | "remote_playback"
  | "remote_control"
  | "cast";

export interface AdminPlaybackPolicyDiagnostics {
  user_policy_rows_supported: boolean;
  role_policy_rows_supported: boolean;
  effective_resolution_supported: boolean;
  library_access_required: boolean;
  user_policy_overrides_role_policy: boolean;
  role_policy_merge: AdminPlaybackPolicyRoleMergeStrategy;
  permissions: AdminPlaybackPolicyPermission[];
}

export type AdminPlaybackResourceClass =
  | "remote_stream"
  | "remote_stage"
  | "remux_process"
  | "cpu_transcode"
  | "gpu_transcode"
  | "hls_artifact_io";

export type AdminPlaybackResourceEnforcement =
  | "host_owned"
  | "admission_permit"
  | "not_yet_enforced";

export interface AdminPlaybackRuntimeDiagnosticsResponse {
  admin_api_version: string;
  public_api_version: string;
  readiness: {
    status: string;
    reason: string;
    checks: Array<{
      name: string;
      status: string;
      reason: string;
    }>;
  };
  policy: AdminPlaybackPolicyDiagnostics;
  ffmpeg: {
    probe_status: string;
    has_probe_error: boolean;
    hardware_capability_count: number;
    available_gpu_capabilities: number;
  };
  hardware: {
    policy: Record<string, unknown>;
    selection: {
      acceleration: string;
      fallback_used: boolean;
      reason: string;
    };
    capabilities: Array<{
      accelerator: string;
      available: boolean;
      reason_code: string;
      encoder_discovery: {
        status: string;
        encoder: string | null;
        has_detail: boolean;
      };
      device_initialization: {
        status: string;
        operator_check: string;
        has_detail: boolean;
      };
      smoke_probe: {
        status: string;
        operator_check: string;
        has_detail: boolean;
      };
    }>;
  };
  transcode: {
    configured_cpu_slots: number;
    configured_gpu_slots: number;
    effective_cpu_slots: number;
    effective_gpu_slots: number;
    selected_hls_slots: number;
  };
  remux: {
    max_concurrent_sessions: number;
    timeout_ms: number;
  };
  resource_pressure: {
    classes: Array<{
      class: AdminPlaybackResourceClass;
      enforcement: AdminPlaybackResourceEnforcement;
      configured_capacity: number | null;
      available_permits: number | null;
      in_use_permits: number | null;
    }>;
  };
  remote_playback: {
    backend_count: number;
    stream_permits_available: number;
    stream_permits_max: number;
    stage_permits_available: number;
    stage_permits_max: number;
    state_scope: string;
  };
  staging: {
    max_bytes: number;
    retention_ms: number;
    cleanup_on_startup: boolean;
    startup_deleted_records: number;
    startup_deleted_files: number;
  };
  artifact_lifecycle: {
    transcode_artifact_retention_ms: number;
    transcode_artifact_cleanup_on_startup: boolean;
    hls_segment_cleanup_enabled: boolean;
    hls_segment_keep_ms: number;
    startup_examined_artifacts: number;
    startup_deleted_artifacts: number;
    startup_deleted_files: number;
    startup_deleted_directories: number;
    startup_deleted_bytes: number;
    startup_skipped_security: number;
  };
  throttle: {
    enabled: boolean;
    delay_ms: number;
  };
}

export type AdminRendererReadinessStatus = "ready" | "degraded" | "unavailable";

export type AdminRendererReadinessReason =
  | "ready"
  | "renderer_repository_ready"
  | "nako_remote_client_adapter_ready"
  | "nako_remote_client_cast_safe_transport_ready"
  | "renderer_repository_unavailable";

export type AdminRendererReadinessCheckName =
  | "renderer_repository"
  | "nako_remote_client_adapter"
  | "nako_remote_client_cast_safe_transport";

export type AdminRendererTargetKind =
  | "browser"
  | "native_desktop"
  | "native_mobile"
  | "nako_remote_client"
  | "chromecast"
  | "dlna_renderer"
  | "airplay";

export type AdminRendererNetworkScope = "local" | "remote" | "unknown";

export type AdminRendererTransportAuth =
  | "bearer"
  | "browser_ticket"
  | "cast_ticket"
  | "none";

export type AdminRendererSessionState = "online" | "offline" | "revoked";

export type AdminRendererControlCommand =
  | "show_item"
  | "play"
  | "pause"
  | "resume"
  | "seek"
  | "stop"
  | "set_volume";

export type AdminRendererAdapterKind =
  | "nako_remote_client"
  | "nako_remote_client_cast_safe_transport"
  | "chromecast"
  | "dlna_renderer"
  | "airplay";

export type AdminRendererAdapterStatus = "ready" | "planned";

export type AdminRendererAdapterReason =
  | "nako_remote_client_ready"
  | "cast_safe_transport_ready"
  | "cast_safe_transport_pending"
  | "chromecast_adapter_planned"
  | "dlna_adapter_planned"
  | "airplay_adapter_planned";

export type AdminRendererControlPlane =
  | "public_client_polling"
  | "adapter_process";

export type AdminRendererDiscoveryMode =
  | "client_registration"
  | "local_network_discovery"
  | "platform_discovery";

export type AdminRendererMediaTransport =
  | "authenticated_nako_client_stream"
  | "cast_safe_url"
  | "native_protocol_stream";

export interface AdminRendererRuntimeDiagnosticsResponse {
  admin_api_version: string;
  public_api_version: string;
  readiness: {
    status: AdminRendererReadinessStatus;
    reason: AdminRendererReadinessReason;
    checks: Array<{
      name: AdminRendererReadinessCheckName;
      status: AdminRendererReadinessStatus;
      reason: AdminRendererReadinessReason;
    }>;
  };
  summary: {
    returned_sessions: number;
    online_sessions: number;
    offline_sessions: number;
    revoked_sessions: number;
    expired_sessions: number;
    active_playback_sessions: number;
  };
  adapters: Array<{
    adapter: AdminRendererAdapterKind;
    target_kind: AdminRendererTargetKind;
    status: AdminRendererAdapterStatus;
    reason: AdminRendererAdapterReason;
    control_plane: AdminRendererControlPlane;
    discovery: AdminRendererDiscoveryMode;
    media_transport: AdminRendererMediaTransport;
    transport_auth: AdminRendererTransportAuth;
  }>;
  sessions: Array<{
    id: string;
    target_kind: AdminRendererTargetKind;
    display_name: string;
    network_scope: AdminRendererNetworkScope;
    transport_auth: AdminRendererTransportAuth;
    state: AdminRendererSessionState;
    active_playback_session_id: string | null;
    supported_commands: AdminRendererControlCommand[];
    has_media_capabilities: boolean;
    direct_play_supported: boolean;
    expired: boolean;
    last_seen_at_ms: number;
    expires_at_ms: number | null;
    created_at: string;
    updated_at: string;
  }>;
  page: PageInfo;
}

export interface AdminPlaybackRuntimeSettingsPayload {
  hardware_acceleration: string;
  hardware_fallback: string;
  cpu_concurrency: number;
  gpu_concurrency: number;
  remux_concurrency: number;
  remux_timeout_ms: number;
  remote_stream_concurrency: number;
  remote_stage_concurrency: number;
  staging_max_bytes: number;
  staging_retention_ms: number;
  staging_cleanup_on_startup: boolean;
  transcode_artifact_retention_ms: number;
  transcode_artifact_cleanup_on_startup: boolean;
  hls_segment_cleanup_enabled: boolean;
  hls_segment_keep_ms: number;
  transcode_throttle_enabled: boolean;
  transcode_throttle_delay_ms: number;
}

export interface AdminUpdatePlaybackRuntimeSettingsRequest {
  settings: AdminPlaybackRuntimeSettingsPayload;
}

export interface AdminPlaybackRuntimeSettingsResponse {
  admin_api_version: string;
  settings: AdminPlaybackRuntimeSettingsPayload;
  source: AdminSettingsSource;
  effect: AdminSettingsEffect;
  updated_at_ms: number | null;
}

export interface AdminPlaybackSupportEvidenceResponse {
  admin_api_version: string;
  public_api_version: string;
  subject: {
    session_id: string | null;
    source_id: string | null;
  };
  session: {
    id: string;
    source_id: string;
    kind: string;
    state: string;
    failure_category: string | null;
    has_failure_message: boolean;
    active: boolean;
    terminal: boolean;
    request_key_fingerprint: string;
    output_artifact_kind: string;
    runtime_metrics: {
      frame_count: number | null;
      fps_millis: number | null;
      bitrate_kbps: number | null;
      total_size_bytes: number | null;
      output_time_ms: number | null;
      dup_frames: number | null;
      drop_frames: number | null;
      speed_millis: number | null;
      progress: "continue" | "end" | null;
    };
    created_at: string;
    updated_at: string;
    started_at: string | null;
    completed_at: string | null;
  } | null;
  source: {
    source_id: string;
    library_id: string;
    item_id: string;
    source_scheme: string;
    file_name: string;
    size_bytes: number | null;
    has_fingerprint: boolean;
  } | null;
  runtime: {
    readiness: AdminPlaybackRuntimeDiagnosticsResponse["readiness"];
    policy: AdminPlaybackPolicyDiagnostics;
    ffmpeg: AdminPlaybackRuntimeDiagnosticsResponse["ffmpeg"];
    hardware: {
      policy: Record<string, unknown>;
      selected_acceleration: string;
      fallback_used: boolean;
      capability_count: number;
      unavailable_capabilities: Array<{
        accelerator: string;
        reason_code: string;
        encoder_discovery_status: string;
        device_initialization_status: string;
        smoke_probe_status: string;
      }>;
    };
    transcode: AdminPlaybackRuntimeDiagnosticsResponse["transcode"];
    remux: AdminPlaybackRuntimeDiagnosticsResponse["remux"];
    remote_playback: AdminPlaybackRuntimeDiagnosticsResponse["remote_playback"];
    staging: AdminPlaybackRuntimeDiagnosticsResponse["staging"];
    artifact_lifecycle: AdminPlaybackRuntimeDiagnosticsResponse["artifact_lifecycle"];
    throttle: AdminPlaybackRuntimeDiagnosticsResponse["throttle"];
  };
  redaction: {
    paths_redacted: boolean;
    source_references_redacted: boolean;
    ffmpeg_commands_redacted: boolean;
    stderr_redacted: boolean;
    credentials_redacted: boolean;
  };
}

export type AdminAddonRuntimeReadinessStatus = "ready" | "degraded" | "unavailable";

export type AdminAddonRuntimeReadinessReason =
  | "ready"
  | "unavailable"
  | "manifest_mismatch"
  | "protocol_mismatch"
  | "missing_grant"
  | "missing_secret_reference"
  | "network_policy_blocked"
  | "sidecar_degraded"
  | "sidecar_unhealthy"
  | "unsafe_response";

export type AdminAddonRuntimeReadinessCheckName =
  | "reachability"
  | "protocol"
  | "manifest"
  | "grants"
  | "secret_references"
  | "network"
  | "safety";

export interface AdminAddonRuntimeReadinessResponse {
  addon_id: string;
  manifest_id: string;
  readiness: {
    status: AdminAddonRuntimeReadinessStatus;
    reason: AdminAddonRuntimeReadinessReason;
    checks: Array<{
      name: AdminAddonRuntimeReadinessCheckName;
      status: AdminAddonRuntimeReadinessStatus;
      reason: AdminAddonRuntimeReadinessReason;
      safe_error_code?: string;
    }>;
  };
}

export type AdminAddonRoutingDeclarationKind = "task" | "event_subscription";

export type AdminAddonRoutingPlanStatus = "executable" | "deferred";

export type AdminAddonRoutingPlanTarget = "addon_task_job" | "event_outbox" | "none";

export interface AdminAddonRoutingPlansResponse {
  addon_id: string;
  manifest_id: string;
  manifest_version: string;
  manifest_fingerprint: string;
  executable: number;
  deferred: number;
  plans: Array<{
    declaration_kind: AdminAddonRoutingDeclarationKind;
    declaration_id: string;
    status: AdminAddonRoutingPlanStatus;
    target: AdminAddonRoutingPlanTarget;
    safe_reason_code?: string;
    job_kind?: string;
    event_kind?: string;
    required_scope_count: number;
    filter_configured: boolean;
    timeout_ms?: number;
    max_attempts?: number;
  }>;
}

export interface AdminStorageStagingDiagnosticsResponse {
  admin_api_version: string;
  public_api_version: string;
  summary: {
    configured_max_bytes: number;
    used_manifest_bytes: number;
    pressure: AdminStorageStagingPressureSummary;
    policy_slices: AdminStorageStagingPolicySlice[];
    purpose_state_summaries: AdminStorageStagingPurposeStateSummary[];
    cleanup_purpose_state_summaries: AdminStorageStagingCleanupPurposeStateSummary[];
    cleanup_on_startup: boolean;
    retention_ms: number;
    startup_deleted_records: number;
    startup_deleted_files: number;
    cleanup_candidate_records: number;
    cleanup_candidate_bytes: number;
    process_cached_backends: number;
    vfs_cache: {
      object_count: number;
      listing_count: number;
      failure_count: number;
      stale_object_count: number;
      stale_listing_count: number;
      last_failure_at_ms: number | null;
      repair: {
        classification: AdminVfsCacheRepairClassification;
        recommended_action: AdminVfsCacheRepairAction;
        operation: VfsCacheOperation | null;
        failure_class: StorageFailureClass | null;
        retryable: boolean;
        failed_at_ms: number | null;
        failure_count: number | null;
        safe_message: string | null;
        operator_action: string;
      } | null;
    };
  };
  records: Array<{
    id: string;
    attribution_kind: AdminStorageStagingAttributionKind;
    attribution_library_id: string | null;
    source_scheme: string;
    purpose: string;
    state: string;
    size_bytes: number | null;
    has_etag: boolean;
    has_fingerprint: boolean;
    active_leases: number;
    has_validation_error: boolean;
    created_at_ms: number;
    updated_at_ms: number;
    last_accessed_at_ms: number;
    expires_at_ms: number | null;
  }>;
  page: PageInfo;
}

export interface AdminVfsCacheRefreshResponse {
  admin_api_version: string;
  public_api_version: string;
  action: AdminVfsCacheRepairAction;
  operation: VfsCacheOperation;
  refreshed: boolean;
  repair: {
    classification: AdminVfsCacheRepairClassification;
    recommended_action: AdminVfsCacheRepairAction;
    operation: VfsCacheOperation | null;
    failure_class: StorageFailureClass | null;
    retryable: boolean;
    failed_at_ms: number | null;
    failure_count: number | null;
    safe_message: string | null;
    operator_action: string;
  };
}

export interface AdminVfsCacheRepairActionPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  plan: {
    status: AdminVfsCacheRepairActionPlanStatus;
    action: AdminVfsCacheRepairAction;
    readiness: AdminVfsCacheRepairActionReadiness;
    boundary: AdminVfsCacheRepairActionBoundary;
    executable_action: AdminVfsCacheRepairExecutableAction | null;
    repair: {
      classification: AdminVfsCacheRepairClassification;
      recommended_action: AdminVfsCacheRepairAction;
      operation: VfsCacheOperation | null;
      failure_class: StorageFailureClass | null;
      retryable: boolean;
      failed_at_ms: number | null;
      failure_count: number | null;
      safe_message: string | null;
      operator_action: string;
    } | null;
  };
}

export interface AdminVfsCacheRepairTargetListResponse {
  admin_api_version: string;
  public_api_version: string;
  targets: AdminVfsCacheRepairTarget[];
  page: PageInfo;
}

export interface AdminVfsCacheRepairTargetPreviewResponse {
  admin_api_version: string;
  public_api_version: string;
  target: AdminVfsCacheRepairTarget;
  plan: AdminVfsCacheRepairActionPlanResponse["plan"];
}

export interface AdminVfsCacheRepairEnqueueRequest {
  priority?: AdminJobPriority | null;
}

export type AdminVfsCacheRepairEnqueueOutcome =
  | "enqueued"
  | "already_queued";

export interface AdminVfsCacheRepairEnqueueResponse {
  admin_api_version: string;
  public_api_version: string;
  outcome: AdminVfsCacheRepairEnqueueOutcome;
  job: AdminJobListItem;
}

export type AdminVfsCacheRepairCacheState =
  | "fresh"
  | "stale_fallback";

export interface AdminVfsCacheRepairJobSummary {
  action: AdminVfsCacheRepairAction;
  source_scheme: string;
  operation: VfsCacheOperation;
  classification: AdminVfsCacheRepairClassification;
  failure_class: StorageFailureClass | null;
  failed_at_ms: number;
  failure_count: number;
  refreshed_cache_state: AdminVfsCacheRepairCacheState | null;
}

export interface AdminVfsCacheRepairExecuteResponse {
  admin_api_version: string;
  public_api_version: string;
  job: AdminJobListItem;
  summary: AdminVfsCacheRepairJobSummary;
}

export interface AdminVfsCacheRepairRetryRequest {
  max_attempts?: number | null;
  next_attempt_at?: string | null;
}

export interface AdminVfsCacheRepairRemediationPlanResponse {
  admin_api_version: string;
  public_api_version: string;
  total_unresolved_targets: number;
  action_groups: AdminVfsCacheRepairRemediationActionGroup[];
  classification_counts: AdminVfsCacheRepairClassificationCount[];
  boundary: AdminVfsCacheRepairRemediationPlanBoundary;
}

export type AdminNetworkExposureMode =
  | "local_only"
  | "private_network"
  | "reverse_proxy"
  | "tunnel_provider";

export type AdminNetworkReadinessStatus = "ready" | "degraded" | "unavailable";

export type AdminNetworkReadinessReason =
  | "ready"
  | "local_only"
  | "auth_disabled"
  | "missing_external_base_url"
  | "missing_trusted_proxy_sources"
  | "missing_tunnel_provider"
  | "missing_tunnel_token"
  | "browser_origins_not_configured";

export type AdminNetworkReadinessCheckName =
  | "exposure_mode"
  | "auth"
  | "external_endpoint"
  | "trusted_proxy"
  | "origin_policy"
  | "tunnel_provider";

export type AdminTunnelProviderKind =
  | "external"
  | "cloudflare_tunnel"
  | "tailscale_funnel"
  | "ngrok";

export interface AdminNetworkAccessDiagnostics {
  exposure_mode: AdminNetworkExposureMode;
  readiness: {
    status: AdminNetworkReadinessStatus;
    reason: AdminNetworkReadinessReason;
    checks: Array<{
      name: AdminNetworkReadinessCheckName;
      status: AdminNetworkReadinessStatus;
      reason: AdminNetworkReadinessReason;
    }>;
  };
  external_endpoint: {
    configured: boolean;
    scheme: string | null;
    host_fingerprint: string | null;
  };
  trusted_proxy: {
    headers_enabled: boolean;
    source_count: number;
  };
  origins: {
    allowed_origin_count: number;
    configured: boolean;
  };
  tunnel_providers: Array<{
    id: string;
    kind: AdminTunnelProviderKind;
    endpoint_configured: boolean;
    endpoint_scheme: string | null;
    endpoint_host_fingerprint: string | null;
    token_env: string | null;
    token_present: boolean;
  }>;
}

export interface AdminAccessSummaryResponse {
  admin_api_version: string;
  public_api_version: string;
  mode: AdminAccessMode;
  principal: {
    principal_id: string;
    display_name: string;
    principal_kind: AdminAccessPrincipalKind;
  };
  auth: {
    enabled: boolean;
    token_reference_configured: boolean;
  };
  readiness: {
    single_admin_mode: AdminAccessCapabilityState;
    user_accounts: AdminAccessCapabilityState;
    roles: AdminAccessCapabilityState;
    library_access_policy: AdminAccessCapabilityState;
  };
  library_access: {
    configured_libraries: number;
    libraries: Array<{
      library_id: string;
      library_name: string;
      preset: string;
      backend_kind: string;
      access: AdminLibraryAccessLevel;
      reason: AdminLibraryAccessReason;
    }>;
  };
}

export interface AdminAccessUserRecord {
  user_id: string;
  principal_id: string;
  username: string;
  display_name: string;
  status: AdminUserStatus;
  roles: AdminUserRole[];
  bootstrap: boolean;
  local_password_configured: boolean;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminAccessUserListResponse {
  admin_api_version: string;
  public_api_version: string;
  users: AdminAccessUserRecord[];
  page: PageInfo;
}

export interface AdminAccessUserResponse {
  admin_api_version: string;
  public_api_version: string;
  user: AdminAccessUserRecord;
}

export interface AdminCreateUserRequest {
  username: string;
  display_name: string;
  roles?: AdminUserRole[];
}

export interface AdminReplaceUserRolesRequest {
  roles: AdminUserRole[];
}

export interface AdminUpdateUserStatusRequest {
  status: AdminUserStatus;
}

export interface AdminSetLocalPasswordRequest {
  password: string;
}

export interface AdminLocalPasswordResponse {
  admin_api_version: string;
  public_api_version: string;
  user_id: string;
  local_password_configured: boolean;
}

export interface AdminInvitationRecord {
  invitation_id: string;
  created_by_user_id: string;
  email_or_username: string | null;
  status: "pending" | "redeemed" | "revoked" | "expired";
  roles: AdminUserRole[];
  expires_at_ms: number;
  redeemed_at_ms: number | null;
  redeemed_by_user_id: string | null;
  revoked_at_ms: number | null;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminInvitationListResponse {
  admin_api_version: string;
  public_api_version: string;
  invitations: AdminInvitationRecord[];
  page: PageInfo;
}

export interface AdminInvitationResponse {
  admin_api_version: string;
  public_api_version: string;
  invitation: AdminInvitationRecord;
}

export interface AdminCreateInvitationResponse {
  admin_api_version: string;
  public_api_version: string;
  invitation: AdminInvitationRecord;
  token: string;
}

export interface AdminCreateInvitationRequest {
  email_or_username?: string | null;
  roles?: AdminUserRole[];
  expires_in_ms?: number | null;
}

export interface AdminLibraryAccessPolicyRecord {
  scope: AdminLibraryAccessPolicyScope;
  library_id: string;
  access: AdminLibraryAccessPolicyLevel;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface AdminLibraryAccessPolicyListResponse {
  admin_api_version: string;
  public_api_version: string;
  policies: AdminLibraryAccessPolicyRecord[];
  page: PageInfo;
}

export interface AdminLibraryAccessPolicyResponse {
  admin_api_version: string;
  public_api_version: string;
  policy: AdminLibraryAccessPolicyRecord;
}

export interface AdminLibraryAccessPolicyDeleteResponse {
  admin_api_version: string;
  public_api_version: string;
  deleted: boolean;
}

export interface AdminUpsertLibraryAccessPolicyRequest {
  scope: AdminLibraryAccessPolicyScope;
  library_id: string;
  access: AdminLibraryAccessPolicyLevel;
}

export interface AdminServerConfigDiagnosticsResponse {
  admin_api_version: string;
  public_api_version: string;
  auth: {
    enabled: boolean;
    token_env: string | null;
  };
  network: AdminNetworkAccessDiagnostics;
  database: {
    configured_backend_kind: string;
    active_backend_kind: string;
    url_scheme: string;
    runtime_supported: boolean;
    migrated_on_startup: boolean;
    capabilities: {
      lifecycle: boolean;
      libraries: boolean;
      jobs: boolean;
      job_leases: boolean;
      media: boolean;
      scan_commits: boolean;
      metadata: boolean;
      catalog: boolean;
      playback_state: boolean;
      playback_sessions: boolean;
      transcode_sessions: boolean;
      event_outbox: boolean;
      addons: boolean;
      automation: boolean;
      managed_artwork: boolean;
      vfs_cache: boolean;
      webhooks: boolean;
      search_index: boolean;
    };
  };
  runtime: {
    listen_addr: string;
    scan_concurrency: number;
    probe_concurrency: number;
    metadata_concurrency: number;
    remux_concurrency: number;
    webhook_concurrency: number;
    remux_timeout_ms: number;
  };
  libraries: Array<{
    id: string;
    name: string;
    preset: string;
    backend_kind: string;
    root_scheme: string;
    has_webdav_password_env: boolean;
    webdav_timeout_ms: number | null;
    webdav_max_attempts: number | null;
  }>;
  metadata: {
    raw_cache_retention_ms: number;
    raw_cache_cleanup_on_startup: boolean;
    raw_cache_cleanup_interval_ms: number;
    maintenance_policies: number;
    providers: Array<{
      provider: string;
      enabled: boolean;
      token_env: string | null;
      api_key_env: string | null;
      has_api_base_url: boolean;
      has_image_base_url: boolean;
      language: string | null;
      include_adult: boolean;
      header_count: number;
      secret_header_count: number;
      has_provider_runtime_override: boolean;
    }>;
    runtime: {
      timeout_ms: number;
      max_attempts: number;
      min_interval_ms: number;
      concurrency: number;
      user_agent: string;
      has_proxy: boolean;
      circuit_breaker_failures: number;
      circuit_breaker_backoff_ms: number;
    };
  };
  transcode: {
    hardware_policy: Record<string, unknown>;
    cpu_concurrency: number;
    gpu_concurrency: number;
  };
  staging: {
    max_bytes: number;
    retention_ms: number;
    cleanup_on_startup: boolean;
  };
  playback: {
    remote_stream_concurrency: number;
    remote_stage_concurrency: number;
    transcode_artifact_retention_ms: number;
    transcode_artifact_cleanup_on_startup: boolean;
    hls_segment_cleanup_enabled: boolean;
    hls_segment_keep_ms: number;
    transcode_throttle_enabled: boolean;
    transcode_throttle_delay_ms: number;
  };
  artwork: {
    artifact_root_configured: boolean;
    fetch_timeout_ms: number;
    fetch_max_attempts: number;
    fetch_max_bytes: number;
    fetch_concurrency: number;
    ingest_worker_enabled: boolean;
    ingest_worker_idle_ms: number;
    fetch_user_agent: string;
    has_fetch_proxy: boolean;
    max_width: number;
    max_height: number;
  };
}

export interface AdminUpdateMetadataRawCacheSettingsRequest {
  retention_ms: number;
  cleanup_on_startup: boolean;
}

export interface AdminMetadataRawCacheSettingsResponse {
  admin_api_version: string;
  retention_ms: number;
  cleanup_on_startup: boolean;
  source: AdminSettingsSource;
  effect: AdminSettingsEffect;
  updated_at_ms: number | null;
}
"#;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use nako_client_protocol::public_client_paths;

    use super::*;

    #[test]
    fn admin_contract_includes_route_constants() {
        let contract = admin_typescript_contract();

        for (key, suffix) in ADMIN_ROUTE_SUFFIXES {
            let path = admin_route_path(suffix);
            assert!(
                contract.contains(&format!("{key}: \"{path}\"")),
                "Admin contract missing route constant {key} -> {path}"
            );
        }

        for expected in [
            "AdminCatalogGovernanceItemsQuery",
            "AdminCatalogGovernanceItemDetailResponse",
            "AdminCatalogGovernanceProviderMappingReviewRequest",
            "AdminCatalogGovernanceProviderMappingReviewPlanResponse",
            "AdminCatalogGovernanceProviderMappingReviewResponse",
            "AdminCatalogGovernanceRepairBoundary",
            "AdminMetadataCandidateReviewQueueQuery",
            "AdminMetadataCandidateReviewQueueResponse",
            "AdminMetadataCandidateReviewBatchPlanRequest",
            "AdminMetadataCandidateReviewBatchPlanSummary",
            "AdminMetadataCandidateReviewBatchPlanResponse",
            "AdminMetadataCandidateReviewBatchApplyItemRequest",
            "AdminMetadataCandidateReviewBatchApplyRequest",
            "AdminMetadataCandidateReviewBatchCreateRequest",
            "AdminMetadataCandidateReviewBatchResponse",
            "AdminMetadataCandidateReviewBatch",
            "AdminMetadataCandidateReviewBatchPlanSelection",
            "AdminMetadataCandidateReviewBatchExecutionSummary",
            "AdminMetadataCandidateReviewBatchItem",
            "AdminMetadataCandidateReviewBatchApplyResultStatus",
            "AdminMetadataCandidateReviewBatchApplySummary",
            "AdminMetadataCandidateReviewBatchApplyError",
            "AdminMetadataCandidateReviewBatchApplyResult",
            "AdminMetadataCandidateReviewBatchApplyResponse",
            "AdminMetadataCandidateReviewListResponse",
            "AdminMetadataCandidateReviewListEntry",
            "AdminMetadataCandidateReviewResponse",
            "AdminMetadataCandidateReviewDetail",
            "AdminMetadataCandidateReviewNode",
            "AdminMetadataCandidateReviewMetadataSummary",
            "AdminMetadataCandidateReviewApplicationPlan",
            "AdminMetadataCandidateReviewApplicationBoundary",
            "AdminMetadataCandidateReviewGovernance",
            "AdminMetadataCandidateReviewAuditTimeline",
            "AdminMetadataCandidateReviewAuditEvent",
            "AdminMetadataCandidateReviewAuditEventKind",
            "AdminMetadataCandidateReviewUndoPlan",
            "AdminMetadataCandidateReviewUndoMode",
            "AdminMetadataCandidateReviewUndoReason",
            "AdminMetadataCandidateReviewApplyRequest",
            "AdminMetadataCandidateReviewApplyResponse",
            "AdminMetadataCandidateReviewRelatedHierarchyPlanRequest",
            "AdminMetadataCandidateReviewRelatedHierarchyPlanResponse",
            "AdminMetadataCandidateReviewRelatedHierarchyApplyRequest",
            "AdminMetadataCandidateReviewRelatedHierarchyApplyResponse",
            "AdminMetadataCandidateReviewRelatedHierarchyApplicationPlan",
            "AdminMetadataCandidateReviewRelatedHierarchyApplicationTarget",
            "AdminMetadataCandidateReviewRelatedHierarchyApplicationBoundary",
            "AdminMetadataCandidateReviewRelatedHierarchyApplicationAction",
            "AdminMetadataCandidateReviewRelatedHierarchyApplicationReason",
            "AdminMetadataCandidateReviewProviderSubject",
            "AdminMetadataCandidateReviewProviderMapping",
            "AdminMetadataCandidateReviewApplicationAction",
            "AdminMetadataCandidateReviewApplicationReason",
            "AdminOutboxEventsQuery",
            "AdminJobsQuery",
            "AdminSourceFingerprintHashMode",
            "AdminJobPriority",
            "AdminJobStatus",
            "AdminSourceFingerprintHashEnqueueRequest",
            "AdminSourceFingerprintHashRetryRequest",
            "AdminSourceDuplicateReconciliationPlanQuery",
            "AdminSourceDuplicateReconciliationApplyExpectedAction",
            "AdminSourceDuplicateReconciliationApplyRequest",
            "AdminSourceFingerprintEvidenceKind",
            "AdminSourceDuplicateEvidenceKind",
            "AdminSourceDuplicateRelationshipStatus",
            "AdminSourceDuplicateReconciliationAction",
            "AdminSourceDuplicateReconciliationCandidate",
            "AdminSourceDuplicateReconciliationPlanResponse",
            "AdminSourceDuplicateReconciliationApplyResponse",
            "AdminPlaybackSessionsQuery",
            "AdminPlaybackSupportQuery",
            "AdminAddonsQuery",
            "RegisterAddonRequest",
            "AdminAddonManifest",
            "AdminAddonEntryPointDeclaration",
            "AdminAddonRegistrationSummary",
            "AdminAddonRegistrationsResponse",
            "AdminAddonRegistrationResponse",
            "AdminAddonLifecycleIntent",
            "AdminAddonManagerPlanRequest",
            "AdminAddonManagerPlanResponse",
            "AdminAddonSourceCatalogSourcesResponse",
            "AdminAddonSourceCatalogEntriesResponse",
            "AdminAddonSourceCatalogResolveResponse",
            "UpdateAddonStatusRequest",
            "AdminAddonHealthCheckResponse",
            "AdminAddonSurfacesResponse",
            "AdminAddonInstallGuideResponse",
            "AdminAddonResourceCallDiagnosticRequest",
            "AdminAddonResourceCallDiagnosticResponse",
            "AdminAddonResourceSearchDiagnosticRequest",
            "AdminAddonResourceSearchDiagnosticResponse",
            "AdminAddonResourceSearchProviderDiagnostic",
            "AdminAddonResourceSearchRequest",
            "AdminAddonResourceSearchResponse",
            "AdminAddonResourceSearchResultSummary",
            "AdminAddonResourceSearchLinkSummary",
            "AdminAddonResourceSearchSelectionRequest",
            "AdminAddonResourceSearchSelectionResponse",
            "AddonSubtitleFormat",
            "AddonSubtitleProviderStatus",
            "AdminAddonSubtitleDeliveryKind",
            "AdminAddonSubtitleSearchRequest",
            "AdminAddonSubtitleProviderDiagnostic",
            "AdminAddonSubtitleCandidateSummary",
            "AdminAddonSubtitleSearchResponse",
            "AdminAddonSubtitleSelectionRequest",
            "AdminAddonSubtitleSelectedReference",
            "AdminAddonSubtitleSelectionResponse",
            "AdminSubtitleSidecarRole",
            "AdminSubtitleImportConflictPolicy",
            "AdminSubtitleImportBackupPolicy",
            "AdminSubtitleImportPlanStatus",
            "AdminSubtitleImportPlanReason",
            "AdminSubtitleImportApplyStatus",
            "AdminAddonSubtitleImportPlanRequest",
            "AdminSubtitleImportTargetSummary",
            "AdminSubtitleSidecarPlan",
            "AdminSubtitleImportPlan",
            "AdminAddonSubtitleImportPlanResponse",
            "AdminAddonSubtitleImportApplyRequest",
            "AdminSubtitleImportApplyReport",
            "AdminSubtitleImportFactSummary",
            "AdminAddonSubtitleImportApplyResponse",
            "AdminAddonResourceLinkCheckRequest",
            "AdminAddonResourceLinkCheckResponse",
            "AddonResourceLinkCheckStatus",
            "AddonAcquisitionCandidateSummary",
            "IssueAddonTokenRequest",
            "AddonTokensResponse",
            "AddonTokenIssuedResponse",
            "AddonTokenRotationResponse",
            "AddonTokenResponse",
            "ReplaceAddonGrantsRequest",
            "AddonGrantsResponse",
            "AdminAcquisitionIntakeCandidatesQuery",
            "AdminWatchFolderDiscoveryRequest",
            "AdminGeneratedArtifactProposalsQuery",
            "AdminGeneratedArtifactReviewRequest",
            "AdminGeneratedArtifactMetadataApplyRequest",
            "AdminGeneratedArtifactMetadataBulkApplyPlanRequest",
            "AdminGeneratedArtifactMetadataBulkApplyRequest",
            "AdminArtworkKind",
            "AdminItemArtworkGalleryQuery",
            "AdminSelectItemArtworkRequest",
            "AdminAcquisitionIntakeCandidateListResponse",
            "AdminWatchFolderSuppression",
            "AdminWatchFolderDiscoveryResponse",
            "AdminGeneratedArtifactProposalListResponse",
            "AdminGeneratedArtifactReviewPlanResponse",
            "AdminGeneratedArtifactReviewResponse",
            "AdminGeneratedArtifactMetadataApplyPlanResponse",
            "AdminGeneratedArtifactMetadataBulkApplyPlanResponse",
            "AdminGeneratedArtifactMetadataBulkApplyBatchResponse",
            "AdminGeneratedArtifactMetadataApplyResponse",
            "AdminGeneratedArtifactMetadataApplyPlan",
            "AdminGeneratedArtifactMetadataApplyFieldPlan",
            "AdminGeneratedArtifactProviderMappingPlan",
            "AdminGeneratedArtifactProviderSubjectPlan",
            "AdminGeneratedArtifactMetadataBulkApplyPlan",
            "AdminGeneratedArtifactMetadataBulkApplyPlanSelection",
            "AdminGeneratedArtifactMetadataBulkApplyPlanSummary",
            "AdminGeneratedArtifactMetadataBulkApplyPlanItem",
            "AdminGeneratedArtifactMetadataBulkApplyBatch",
            "AdminGeneratedArtifactMetadataBulkApplyBatchExecutionSummary",
            "AdminGeneratedArtifactMetadataBulkApplyBatchItem",
            "AdminGeneratedArtifactMetadataValueSummary",
            "AdminManagedArtworkGalleryResponse",
            "AdminManagedArtworkGallerySummary",
            "AdminManagedArtworkGalleryCandidate",
            "AdminManagedArtworkGalleryArtifact",
            "AdminManagedArtworkGallerySelected",
            "ManagedArtworkIngestSummary",
            "SelectedArtworkSummary",
            "PublishSelectedArtworkResponse",
            "UnpublishSelectedArtworkResponse",
            "AdminMetadataProfile",
            "AdminMetadataScanPolicy",
            "AdminUpdateLibraryMetadataProfileRequest",
            "AdminLibraryMetadataProfileResponse",
            "AdminJobDiagnostics",
            "AdminVfsCacheRepairJobDiagnostics",
            "AdminVfsCacheRepairJobDiagnosticStatus",
            "AdminVfsCacheRepairJobFailureDiagnostic",
            "AdminJobCommandResponse",
            "AdminStorageBackendsQuery",
            "AdminStorageStagingQuery",
            "AdminStorageStagingPressureStatus",
            "AdminStorageStagingAttributionKind",
            "AdminStorageStagingPressureSummary",
            "AdminStorageStagingPolicySlice",
            "AdminStorageStagingPurposeStateSummary",
            "AdminStorageStagingCleanupPurposeStateSummary",
            "StorageBackendHealthStatus",
            "StorageCircuitBreakerState",
            "StorageFailureClass",
            "VfsCacheOperation",
            "AdminVfsCacheRepairClassification",
            "AdminVfsCacheRepairActionPlanStatus",
            "AdminVfsCacheRepairActionPlanReason",
            "AdminVfsCacheRepairActionReadiness",
            "AdminVfsCacheRepairActionBoundary",
            "AdminVfsCacheRepairExecutableAction",
            "AdminVfsCacheRepairTarget",
            "AdminVfsCacheRepairTargetListResponse",
            "AdminVfsCacheRepairTargetPreviewResponse",
            "AdminVfsCacheRepairEnqueueRequest",
            "AdminVfsCacheRepairEnqueueOutcome",
            "AdminVfsCacheRepairEnqueueResponse",
            "AdminVfsCacheRepairCacheState",
            "AdminVfsCacheRepairJobSummary",
            "AdminVfsCacheRepairExecuteResponse",
            "AdminVfsCacheRepairRetryRequest",
            "AdminVfsCacheRepairActionPlanResponse",
            "AdminVfsCacheRepairRemediationPlanBoundary",
            "AdminVfsCacheRepairClassificationCount",
            "AdminVfsCacheRepairRemediationActionGroup",
            "AdminVfsCacheRepairRemediationPlanResponse",
            "AdminVfsCacheRepairAutomationPolicyRequest",
            "AdminVfsCacheRepairAutomationBlockReason",
            "AdminVfsCacheRepairAutomationBoundary",
            "AdminVfsCacheRepairAutomationEligibleTarget",
            "AdminVfsCacheRepairAutomationBlockedTarget",
            "AdminVfsCacheRepairAutomationPolicyReport",
            "AdminVfsCacheRepairAutomationPlanResponse",
            "AdminVfsCacheRepairAutomationEnqueueRequest",
            "AdminVfsCacheRepairAutomationJob",
            "AdminVfsCacheRepairAutomationEnqueueResponse",
            "AdminStorageBackendHealthDiagnostic",
            "AdminStorageBackendHealthDiagnosticsResponse",
            "AdminStorageBackendHealthResetResponse",
            "AdminOverviewSourceFingerprintHashSummary",
            "AdminOverviewResponse",
            "AdminPlaybackSupportEvidenceResponse",
            "runtime_metrics",
            "total_size_bytes",
            "output_time_ms",
            "AdminPlaybackPolicyDiagnostics",
            "AdminPlaybackPolicyPermission",
            "AdminRendererRuntimeDiagnosticsResponse",
            "AdminRendererAdapterKind",
            "AdminRendererMediaTransport",
            "AdminAddonRuntimeReadinessResponse",
            "AdminAddonRoutingPlansResponse",
            "AdminNetworkAccessDiagnostics",
            "AdminAccessSummaryResponse",
            "AdminSetLocalPasswordRequest",
            "AdminLocalPasswordResponse",
            "AdminAccessMode",
            "AdminAccessCapabilityState",
            "AdminLibraryAccessLevel",
            "AdminServerConfigDiagnosticsResponse",
            "AdminSettingsSource",
            "AdminSettingsEffect",
            "AdminUpdateMetadataRawCacheSettingsRequest",
            "AdminMetadataRawCacheSettingsResponse",
        ] {
            assert!(
                contract.contains(expected),
                "Admin contract missing {expected}"
            );
        }
    }

    #[test]
    fn admin_route_normalization_treats_axum_and_generated_params_as_same() {
        assert_eq!(
            normalize_admin_route_path("/admin/v1/addons/:addon_id/status"),
            "/admin/v1/addons/{addon_id}/status"
        );
        assert_eq!(
            normalize_admin_route_path("/admin/v1/addons/{addon_id}/status"),
            "/admin/v1/addons/{addon_id}/status"
        );
        assert_eq!(
            normalize_admin_route_path("/admin/v1/addons/:addon_id/tokens/:token_id/revoke"),
            "/admin/v1/addons/{addon_id}/tokens/{token_id}/revoke"
        );
    }

    #[test]
    fn admin_route_inventory_exclusions_are_explicit_and_disjoint() {
        let admin_prefix = format!("/admin/{ADMIN_API_VERSION}/");
        let mut route_keys = BTreeSet::new();
        let mut generated_paths = BTreeSet::new();

        for route in admin_contract_routes() {
            assert!(
                route.path.starts_with(&admin_prefix),
                "Admin route constant {} must stay under {admin_prefix}: {}",
                route.key,
                route.path
            );
            assert!(
                route_keys.insert(route.key),
                "Duplicate Admin route key: {}",
                route.key
            );
            assert!(
                generated_paths.insert(normalize_admin_route_path(&route.path)),
                "Duplicate normalized generated Admin route path: {}",
                route.path
            );
        }

        let mut excluded_paths = BTreeSet::new();
        for exclusion in admin_contract_route_exclusions() {
            assert!(
                exclusion.path.starts_with(&admin_prefix),
                "Excluded Admin route must stay under {admin_prefix}: {}",
                exclusion.path
            );
            assert!(
                !exclusion.reason.trim().is_empty(),
                "Excluded Admin route must have an explicit reason: {}",
                exclusion.path
            );

            let normalized = normalize_admin_route_path(&exclusion.path);
            assert!(
                !generated_paths.contains(&normalized),
                "Excluded Admin route is also generated: {}",
                exclusion.path
            );
            assert!(
                excluded_paths.insert(normalized),
                "Duplicate excluded Admin route path: {}",
                exclusion.path
            );
        }
    }

    #[test]
    fn admin_contract_excludes_generated_fetch_runtime_and_raw_sensitive_fields() {
        let contract = admin_typescript_contract().to_ascii_lowercase();

        for forbidden in [
            "class adminapiclient",
            "fetchlike",
            "requestjson",
            "source_uri",
            "source_locator",
            "cache_uri",
            "storage_uri",
            "prompt_json",
            "artifact_json",
            "database_url",
            "output_path",
            "local_path",
            "raw_source_url",
            "raw_generated",
            "token_value",
            "access_token",
            "bearer_token",
            "resolved_secret",
            "providerrawresponse",
        ] {
            assert!(
                !contract.contains(forbidden),
                "Admin contract leaked forbidden term: {forbidden}"
            );
        }

        assert!(
            contract.contains("raw_token: string"),
            "Admin contract should expose raw_token only for one-time addon token issue/rotation responses"
        );
        assert_eq!(
            contract.matches("raw_token").count(),
            2,
            "raw_token must stay limited to explicit one-time token response DTOs"
        );
    }

    #[test]
    fn public_typescript_sdk_still_excludes_admin_routes() {
        let public_sdk = crate::sdk::typescript_sdk().to_ascii_lowercase();

        for forbidden in ["/admin", "/admin/v1", "nako_admin_routes"] {
            assert!(
                !public_sdk.contains(forbidden),
                "Public TypeScript SDK leaked admin term: {forbidden}"
            );
        }
    }

    #[test]
    fn admin_contract_routes_stay_out_of_public_client_inventory() {
        let public_paths = public_client_paths().collect::<Vec<_>>();

        for route in admin_contract_routes() {
            assert!(
                !public_paths.contains(&route.path.as_str()),
                "Public Client route inventory leaked Admin API path: {}",
                route.path
            );
        }

        for exclusion in admin_contract_route_exclusions() {
            assert!(
                !public_paths.contains(&exclusion.path.as_str()),
                "Public Client route inventory leaked excluded Admin API path: {}",
                exclusion.path
            );
        }
    }

    #[test]
    fn provider_governance_route_shapes_stay_out_of_public_client_inventory() {
        let public_paths = public_client_paths().collect::<Vec<_>>();

        for suffix in [
            "catalog/governance/items",
            "catalog/governance/items/{item_id}",
            "catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan",
            "catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review",
            "metadata/candidate-reviews",
            "metadata/candidate-reviews/batch-application-plan",
            "metadata/candidate-reviews/batch-apply",
            "metadata/candidate-reviews/batches",
            "metadata/candidate-reviews/batches/{batch_id}",
            "metadata/items/{item_id}/candidate-reviews",
            "metadata/candidate-reviews/{review_id}",
            "metadata/candidate-reviews/{review_id}/apply",
            "metadata/candidate-reviews/{review_id}/related-hierarchy/application-plan",
            "metadata/candidate-reviews/{review_id}/related-hierarchy/apply",
        ] {
            let public_shape_path = format!("/{suffix}");
            assert!(
                !public_paths.contains(&public_shape_path.as_str()),
                "Public Client route inventory leaked provider governance path: {public_shape_path}"
            );
        }
    }

    #[test]
    fn admin_web_generated_contract_matches_generator_output() {
        let generated = admin_typescript_contract().replace("\r\n", "\n");
        let app_local = include_str!("../../../apps/admin-web/src/adminApi/generated/contract.ts")
            .replace("\r\n", "\n");
        let web_local =
            include_str!("../../../web/src/api/admin/generated/contract.ts").replace("\r\n", "\n");

        assert_eq!(
            app_local, generated,
            "run `cargo run -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts` to refresh the Admin API TypeScript contract"
        );
        assert_eq!(
            web_local, generated,
            "run `cargo run -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts` to refresh the web Admin API TypeScript contract"
        );
    }
}
