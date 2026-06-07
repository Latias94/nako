use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write as _,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU8, AtomicU64, Ordering},
    },
};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore, TryAcquireError};

use crate::config::{
    LocalLibraryConfig, NakoServerConfig, PlaybackConfig, WebDavLibraryConfig,
    configured_library_config_for,
};
use nako_api::admin::{
    StorageBackendDiagnostic, StorageBackendDiagnosticsResponse, StorageBackendHealthDiagnostic,
    StorageBackendKind, StorageBackendRegistryDiagnostic, StorageBackendRuntimeStateScope,
    StorageBackendStatus,
};
use nako_core::{
    EnqueueJobRetry, Job, JobId, JobKind, JobLeaseClaimFilter, JobListFilter, JobPriority,
    JobRepository, JobStatus, LeasedJob, Library, LibraryId, LibraryRepository, MediaSource,
    NakoError, NewJob, PageRequest, Result, StagingAttribution, StagingManifestRecord,
    StagingManifestRepository, StagingPurpose, StagingState, StorageBackendHealthListFilter,
    StorageBackendHealthRecord, StorageBackendHealthRepository, StorageBackendHealthStatus,
    StorageCircuitBreakerState, StorageFailureClass, VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS,
    VfsCacheFailure, VfsCacheFailureAuthority, VfsCacheOperation, VfsCacheRepairJobInput,
    VfsCacheRepository, VfsCacheSummary,
};
use nako_db::NakoDatabase;
use nako_vfs::{
    LocalFsBackend, ObjectCacheState, StorageBackend, StorageUri, VfsCacheRefreshReport,
    VfsCacheRepairAction, VfsCacheRepairClassification, VfsCacheRepairDiagnostic,
};
use serde::{Deserialize, Serialize};

use super::current_time_ms;
use super::job_retry::canonical_retry_next_attempt;
use super::job_runtime::{DurableJobOperationError, DurableJobRunOutcome, DurableJobRuntime};

type VfsCacheRepairTargetRefMac = Hmac<Sha256>;
const VFS_CACHE_REPAIR_REMEDIATION_SAMPLE_LIMIT: usize = 3;

#[derive(Clone, Debug)]
pub(crate) struct StorageDiagnosticsAppService {
    registry: StorageBackendRegistry,
    repair_target_ref_secret: Arc<[u8; 32]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StagingCleanupPressureSummary {
    pub(crate) cleanup_candidate_records: usize,
    pub(crate) cleanup_candidate_bytes: u64,
    pub(crate) cleanup_purpose_state_summaries: Vec<StagingCleanupPurposeStateSummary>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StagingManifestPressureSummary {
    pub(crate) total_records: usize,
    pub(crate) in_flight_records: usize,
    pub(crate) failed_records: usize,
    pub(crate) unknown_size_records: usize,
    pub(crate) active_leases: u64,
    pub(crate) ffmpeg_input_records: usize,
    pub(crate) probe_input_records: usize,
    pub(crate) purpose_state_summaries: Vec<StagingPurposeStateSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StagingPurposeStateSummary {
    pub(crate) purpose: StagingPurpose,
    pub(crate) state: StagingState,
    pub(crate) record_count: u32,
    pub(crate) used_manifest_bytes: u64,
    pub(crate) active_leases: u32,
    pub(crate) unknown_size_records: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StagingCleanupPurposeStateSummary {
    pub(crate) purpose: StagingPurpose,
    pub(crate) state: StagingState,
    pub(crate) record_count: u32,
    pub(crate) cleanup_candidate_bytes: u64,
    pub(crate) active_leases: u32,
    pub(crate) unknown_size_records: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairRefreshActionReport {
    pub(crate) action: VfsCacheRepairAction,
    pub(crate) operation: VfsCacheOperation,
    pub(crate) repair: VfsCacheRepairDiagnostic,
    pub(crate) refresh: VfsCacheRefreshReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VfsCacheRepairActionPlanStatus {
    NoAction,
    Executable,
    PlanOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VfsCacheRepairActionPlanReason {
    NoRepairDiagnostic,
    NoActionRequired,
    RefreshCacheExecutable,
    BackendConfigurationRequired,
    ManualFailureInspectionRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VfsCacheRepairExecutableRoute {
    LatestRefreshCache,
    TargetRefreshCache,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairActionBoundary {
    pub(crate) refreshes_vfs_cache: bool,
    pub(crate) changes_backend_configuration: bool,
    pub(crate) requires_manual_failure_inspection: bool,
    pub(crate) deletes_cache_entries: bool,
    pub(crate) writes_library_files: bool,
    pub(crate) starts_durable_job: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairActionPlanReport {
    pub(crate) status: VfsCacheRepairActionPlanStatus,
    pub(crate) action: VfsCacheRepairAction,
    pub(crate) api_executable: bool,
    pub(crate) reasons: Vec<VfsCacheRepairActionPlanReason>,
    pub(crate) boundary: VfsCacheRepairActionBoundary,
    pub(crate) executable_route: Option<VfsCacheRepairExecutableRoute>,
    pub(crate) repair: Option<VfsCacheRepairDiagnostic>,
}

impl VfsCacheRepairActionPlanReport {
    fn from_repair(repair: Option<VfsCacheRepairDiagnostic>) -> Self {
        let Some(repair) = repair else {
            return Self {
                status: VfsCacheRepairActionPlanStatus::NoAction,
                action: VfsCacheRepairAction::None,
                api_executable: false,
                reasons: vec![VfsCacheRepairActionPlanReason::NoRepairDiagnostic],
                boundary: VfsCacheRepairActionBoundary::default(),
                executable_route: None,
                repair: None,
            };
        };

        let (status, api_executable, reasons, boundary, executable_route) =
            match repair.recommended_action {
                VfsCacheRepairAction::None => (
                    VfsCacheRepairActionPlanStatus::NoAction,
                    false,
                    vec![VfsCacheRepairActionPlanReason::NoActionRequired],
                    VfsCacheRepairActionBoundary::default(),
                    None,
                ),
                VfsCacheRepairAction::RefreshCache => (
                    VfsCacheRepairActionPlanStatus::Executable,
                    true,
                    vec![VfsCacheRepairActionPlanReason::RefreshCacheExecutable],
                    VfsCacheRepairActionBoundary {
                        refreshes_vfs_cache: true,
                        ..VfsCacheRepairActionBoundary::default()
                    },
                    Some(VfsCacheRepairExecutableRoute::LatestRefreshCache),
                ),
                VfsCacheRepairAction::FixBackendConfiguration => (
                    VfsCacheRepairActionPlanStatus::PlanOnly,
                    false,
                    vec![VfsCacheRepairActionPlanReason::BackendConfigurationRequired],
                    VfsCacheRepairActionBoundary {
                        changes_backend_configuration: true,
                        ..VfsCacheRepairActionBoundary::default()
                    },
                    None,
                ),
                VfsCacheRepairAction::InspectFailure => (
                    VfsCacheRepairActionPlanStatus::PlanOnly,
                    false,
                    vec![VfsCacheRepairActionPlanReason::ManualFailureInspectionRequired],
                    VfsCacheRepairActionBoundary {
                        requires_manual_failure_inspection: true,
                        ..VfsCacheRepairActionBoundary::default()
                    },
                    None,
                ),
            };

        Self {
            status,
            action: repair.recommended_action,
            api_executable,
            reasons,
            boundary,
            executable_route,
            repair: Some(repair),
        }
    }

    fn from_target_preview_repair(repair: VfsCacheRepairDiagnostic) -> Self {
        let mut plan = Self::from_repair(Some(repair));
        if plan.executable_route == Some(VfsCacheRepairExecutableRoute::LatestRefreshCache) {
            plan.executable_route = Some(VfsCacheRepairExecutableRoute::TargetRefreshCache);
        }

        plan
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairTargetReport {
    pub(crate) target_ref: String,
    pub(crate) scheme: String,
    pub(crate) operation: VfsCacheOperation,
    pub(crate) failed_at_ms: i64,
    pub(crate) failure_count: u32,
    pub(crate) repair: VfsCacheRepairDiagnostic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairTargetPreviewReport {
    pub(crate) target: VfsCacheRepairTargetReport,
    pub(crate) plan: VfsCacheRepairActionPlanReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EnqueueVfsCacheRepairTargetOutcome {
    Enqueued(Job),
    AlreadyQueued(Job),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetryVfsCacheRepairJobRequest {
    pub(crate) job_id: JobId,
    pub(crate) max_attempts: Option<u32>,
    pub(crate) next_attempt_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct VfsCacheRepairJobSummary {
    pub(crate) action: VfsCacheRepairAction,
    pub(crate) source_scheme: String,
    pub(crate) operation: VfsCacheOperation,
    pub(crate) classification: VfsCacheRepairClassification,
    pub(crate) failure_class: Option<StorageFailureClass>,
    pub(crate) failed_at_ms: i64,
    pub(crate) failure_count: u32,
    pub(crate) refreshed_cache_state: Option<ObjectCacheState>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairCommandOutput {
    pub(crate) job: Job,
    pub(crate) summary: VfsCacheRepairJobSummary,
}

impl VfsCacheRepairJobSummary {
    fn from_refresh_report(
        input: &VfsCacheRepairJobInput,
        report: &VfsCacheRepairRefreshActionReport,
    ) -> Self {
        Self {
            action: report.action,
            source_scheme: input.source_scheme.clone(),
            operation: report.operation,
            classification: report.repair.classification,
            failure_class: report.repair.failure_class,
            failed_at_ms: input.failed_at_ms,
            failure_count: input.failure_count,
            refreshed_cache_state: report.refresh.cache.as_ref().map(|cache| cache.state),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairRemediationPlanBoundary {
    pub(crate) read_only: bool,
    pub(crate) refreshes_vfs_cache: bool,
    pub(crate) changes_backend_configuration: bool,
    pub(crate) deletes_cache_entries: bool,
    pub(crate) writes_library_files: bool,
    pub(crate) starts_durable_job: bool,
}

impl Default for VfsCacheRepairRemediationPlanBoundary {
    fn default() -> Self {
        Self {
            read_only: true,
            refreshes_vfs_cache: false,
            changes_backend_configuration: false,
            deletes_cache_entries: false,
            writes_library_files: false,
            starts_durable_job: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairRemediationActionGroupReport {
    pub(crate) action: VfsCacheRepairAction,
    pub(crate) count: u32,
    pub(crate) status: VfsCacheRepairActionPlanStatus,
    pub(crate) api_executable: bool,
    pub(crate) reasons: Vec<VfsCacheRepairActionPlanReason>,
    pub(crate) boundary: VfsCacheRepairActionBoundary,
    pub(crate) executable_route: Option<VfsCacheRepairExecutableRoute>,
    pub(crate) sample_targets: Vec<VfsCacheRepairTargetReport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairRemediationClassificationCountReport {
    pub(crate) classification: VfsCacheRepairClassification,
    pub(crate) count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairRemediationPlanReport {
    pub(crate) total_unresolved_targets: u32,
    pub(crate) action_groups: Vec<VfsCacheRepairRemediationActionGroupReport>,
    pub(crate) classification_counts: Vec<VfsCacheRepairRemediationClassificationCountReport>,
    pub(crate) boundary: VfsCacheRepairRemediationPlanBoundary,
}

#[derive(Default)]
struct VfsCacheRepairRemediationPlanAccumulator {
    total_unresolved_targets: u32,
    none: VfsCacheRepairRemediationActionAccumulator,
    refresh_cache: VfsCacheRepairRemediationActionAccumulator,
    fix_backend_configuration: VfsCacheRepairRemediationActionAccumulator,
    inspect_failure: VfsCacheRepairRemediationActionAccumulator,
    healthy: u32,
    repairable_stale_fallback: u32,
    retryable_refresh_failure: u32,
    operator_action_required: u32,
    unknown_failure: u32,
}

#[derive(Default)]
struct VfsCacheRepairRemediationActionAccumulator {
    count: u32,
    sample_targets: Vec<VfsCacheRepairTargetReport>,
}

impl VfsCacheRepairRemediationPlanAccumulator {
    fn record(&mut self, target: VfsCacheRepairTargetReport) {
        self.total_unresolved_targets = self.total_unresolved_targets.saturating_add(1);
        match target.repair.classification {
            VfsCacheRepairClassification::Healthy => {
                self.healthy = self.healthy.saturating_add(1);
            }
            VfsCacheRepairClassification::RepairableStaleFallback => {
                self.repairable_stale_fallback = self.repairable_stale_fallback.saturating_add(1);
            }
            VfsCacheRepairClassification::RetryableRefreshFailure => {
                self.retryable_refresh_failure = self.retryable_refresh_failure.saturating_add(1);
            }
            VfsCacheRepairClassification::OperatorActionRequired => {
                self.operator_action_required = self.operator_action_required.saturating_add(1);
            }
            VfsCacheRepairClassification::UnknownFailure => {
                self.unknown_failure = self.unknown_failure.saturating_add(1);
            }
        }

        let action = target.repair.recommended_action;
        let accumulator = match action {
            VfsCacheRepairAction::None => &mut self.none,
            VfsCacheRepairAction::RefreshCache => &mut self.refresh_cache,
            VfsCacheRepairAction::FixBackendConfiguration => &mut self.fix_backend_configuration,
            VfsCacheRepairAction::InspectFailure => &mut self.inspect_failure,
        };
        accumulator.record(target);
    }

    fn into_report(self) -> VfsCacheRepairRemediationPlanReport {
        let mut action_groups = Vec::new();
        push_remediation_action_group(
            &mut action_groups,
            VfsCacheRepairAction::RefreshCache,
            self.refresh_cache,
        );
        push_remediation_action_group(
            &mut action_groups,
            VfsCacheRepairAction::FixBackendConfiguration,
            self.fix_backend_configuration,
        );
        push_remediation_action_group(
            &mut action_groups,
            VfsCacheRepairAction::InspectFailure,
            self.inspect_failure,
        );
        push_remediation_action_group(&mut action_groups, VfsCacheRepairAction::None, self.none);

        VfsCacheRepairRemediationPlanReport {
            total_unresolved_targets: self.total_unresolved_targets,
            action_groups,
            classification_counts: vec![
                VfsCacheRepairRemediationClassificationCountReport {
                    classification: VfsCacheRepairClassification::Healthy,
                    count: self.healthy,
                },
                VfsCacheRepairRemediationClassificationCountReport {
                    classification: VfsCacheRepairClassification::RepairableStaleFallback,
                    count: self.repairable_stale_fallback,
                },
                VfsCacheRepairRemediationClassificationCountReport {
                    classification: VfsCacheRepairClassification::RetryableRefreshFailure,
                    count: self.retryable_refresh_failure,
                },
                VfsCacheRepairRemediationClassificationCountReport {
                    classification: VfsCacheRepairClassification::OperatorActionRequired,
                    count: self.operator_action_required,
                },
                VfsCacheRepairRemediationClassificationCountReport {
                    classification: VfsCacheRepairClassification::UnknownFailure,
                    count: self.unknown_failure,
                },
            ],
            boundary: VfsCacheRepairRemediationPlanBoundary::default(),
        }
    }
}

impl VfsCacheRepairRemediationActionAccumulator {
    fn record(&mut self, target: VfsCacheRepairTargetReport) {
        self.count = self.count.saturating_add(1);
        if self.sample_targets.len() < VFS_CACHE_REPAIR_REMEDIATION_SAMPLE_LIMIT {
            self.sample_targets.push(target);
        }
    }
}

fn push_remediation_action_group(
    action_groups: &mut Vec<VfsCacheRepairRemediationActionGroupReport>,
    action: VfsCacheRepairAction,
    accumulator: VfsCacheRepairRemediationActionAccumulator,
) {
    if accumulator.count == 0 {
        return;
    }

    let (status, api_executable, reasons, boundary, executable_route) = match action {
        VfsCacheRepairAction::None => (
            VfsCacheRepairActionPlanStatus::NoAction,
            false,
            vec![VfsCacheRepairActionPlanReason::NoActionRequired],
            VfsCacheRepairActionBoundary::default(),
            None,
        ),
        VfsCacheRepairAction::RefreshCache => (
            VfsCacheRepairActionPlanStatus::Executable,
            true,
            vec![VfsCacheRepairActionPlanReason::RefreshCacheExecutable],
            VfsCacheRepairActionBoundary {
                refreshes_vfs_cache: true,
                ..VfsCacheRepairActionBoundary::default()
            },
            Some(VfsCacheRepairExecutableRoute::TargetRefreshCache),
        ),
        VfsCacheRepairAction::FixBackendConfiguration => (
            VfsCacheRepairActionPlanStatus::PlanOnly,
            false,
            vec![VfsCacheRepairActionPlanReason::BackendConfigurationRequired],
            VfsCacheRepairActionBoundary {
                changes_backend_configuration: true,
                ..VfsCacheRepairActionBoundary::default()
            },
            None,
        ),
        VfsCacheRepairAction::InspectFailure => (
            VfsCacheRepairActionPlanStatus::PlanOnly,
            false,
            vec![VfsCacheRepairActionPlanReason::ManualFailureInspectionRequired],
            VfsCacheRepairActionBoundary {
                requires_manual_failure_inspection: true,
                ..VfsCacheRepairActionBoundary::default()
            },
            None,
        ),
    };

    action_groups.push(VfsCacheRepairRemediationActionGroupReport {
        action,
        count: accumulator.count,
        status,
        api_executable,
        reasons,
        boundary,
        executable_route,
        sample_targets: accumulator.sample_targets,
    });
}

impl StagingManifestPressureSummary {
    fn record(&mut self, record: &StagingManifestRecord) {
        self.total_records = self.total_records.saturating_add(1);
        self.active_leases = self
            .active_leases
            .saturating_add(u64::from(record.active_leases));
        if record.size_bytes.is_none() {
            self.unknown_size_records = self.unknown_size_records.saturating_add(1);
        }
        match record.state {
            StagingState::Reserved | StagingState::Staging | StagingState::Leased => {
                self.in_flight_records = self.in_flight_records.saturating_add(1);
            }
            StagingState::Failed => {
                self.failed_records = self.failed_records.saturating_add(1);
            }
            StagingState::Ready | StagingState::Expired | StagingState::Deleted => {}
        }
        match record.purpose {
            StagingPurpose::FfmpegInput => {
                self.ffmpeg_input_records = self.ffmpeg_input_records.saturating_add(1);
            }
            StagingPurpose::ProbeInput => {
                self.probe_input_records = self.probe_input_records.saturating_add(1);
            }
        }
        if let Some(summary) = self
            .purpose_state_summaries
            .iter_mut()
            .find(|summary| summary.purpose == record.purpose && summary.state == record.state)
        {
            summary.record(record);
        } else {
            self.purpose_state_summaries
                .push(StagingPurposeStateSummary::new(record));
        }
    }

    fn finish(&mut self) {
        self.purpose_state_summaries.sort_by(|left, right| {
            left.purpose
                .as_str()
                .cmp(right.purpose.as_str())
                .then_with(|| left.state.as_str().cmp(right.state.as_str()))
        });
    }
}

impl StagingCleanupPressureSummary {
    fn record(&mut self, record: &StagingManifestRecord) {
        self.cleanup_candidate_records = self.cleanup_candidate_records.saturating_add(1);
        self.cleanup_candidate_bytes = self
            .cleanup_candidate_bytes
            .saturating_add(record.size_bytes.unwrap_or(0));
        if let Some(summary) = self
            .cleanup_purpose_state_summaries
            .iter_mut()
            .find(|summary| summary.purpose == record.purpose && summary.state == record.state)
        {
            summary.record(record);
        } else {
            self.cleanup_purpose_state_summaries
                .push(StagingCleanupPurposeStateSummary::new(record));
        }
    }

    fn finish(&mut self) {
        self.cleanup_purpose_state_summaries.sort_by(|left, right| {
            left.purpose
                .as_str()
                .cmp(right.purpose.as_str())
                .then_with(|| left.state.as_str().cmp(right.state.as_str()))
        });
    }
}

impl StagingPurposeStateSummary {
    fn new(record: &StagingManifestRecord) -> Self {
        let mut summary = Self {
            purpose: record.purpose,
            state: record.state,
            record_count: 0,
            used_manifest_bytes: 0,
            active_leases: 0,
            unknown_size_records: 0,
        };
        summary.record(record);
        summary
    }

    fn record(&mut self, record: &StagingManifestRecord) {
        self.record_count = self.record_count.saturating_add(1);
        self.used_manifest_bytes = self
            .used_manifest_bytes
            .saturating_add(record.size_bytes.unwrap_or(0));
        self.active_leases = self.active_leases.saturating_add(record.active_leases);
        if record.size_bytes.is_none() {
            self.unknown_size_records = self.unknown_size_records.saturating_add(1);
        }
    }
}

impl StagingCleanupPurposeStateSummary {
    fn new(record: &StagingManifestRecord) -> Self {
        let mut summary = Self {
            purpose: record.purpose,
            state: record.state,
            record_count: 0,
            cleanup_candidate_bytes: 0,
            active_leases: 0,
            unknown_size_records: 0,
        };
        summary.record(record);
        summary
    }

    fn record(&mut self, record: &StagingManifestRecord) {
        self.record_count = self.record_count.saturating_add(1);
        self.cleanup_candidate_bytes = self
            .cleanup_candidate_bytes
            .saturating_add(record.size_bytes.unwrap_or(0));
        self.active_leases = self.active_leases.saturating_add(record.active_leases);
        if record.size_bytes.is_none() {
            self.unknown_size_records = self.unknown_size_records.saturating_add(1);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StagingBudgetPolicySlice {
    pub(crate) backend_key: String,
    pub(crate) library_id: Option<LibraryId>,
    pub(crate) library_name: Option<String>,
    pub(crate) backend_kind: Option<StorageBackendKind>,
    pub(crate) source_scheme: String,
    pub(crate) configured_max_bytes: u64,
    pub(crate) used_manifest_bytes: u64,
    pub(crate) manifest_pressure: StagingManifestPressureSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StagingBudgetPolicyScope {
    source_scheme: String,
}

impl StagingBudgetPolicyScope {
    fn from_config(config: &LocalLibraryConfig) -> Self {
        let source_scheme = config
            .webdav
            .as_ref()
            .map_or("local", |_| "webdav")
            .to_owned();

        Self { source_scheme }
    }

    fn backend(source_scheme: &str) -> Self {
        Self {
            source_scheme: source_scheme.to_owned(),
        }
    }

    fn matches_source_scheme(&self, source_scheme: &str) -> bool {
        self.source_scheme == source_scheme
    }
}

struct StagingBudgetPolicyAccumulator {
    slice: StagingBudgetPolicySlice,
    scope: StagingBudgetPolicyScope,
}

impl StagingBudgetPolicyAccumulator {
    fn configured(config: &LocalLibraryConfig, configured_max_bytes: u64) -> Self {
        let scope = StagingBudgetPolicyScope::from_config(config);

        Self {
            slice: StagingBudgetPolicySlice {
                backend_key: storage_backend_key(config.id, &scope.source_scheme),
                library_id: Some(config.id),
                library_name: Some(config.name.clone()),
                backend_kind: Some(backend_kind(config)),
                source_scheme: scope.source_scheme.clone(),
                configured_max_bytes,
                used_manifest_bytes: 0,
                manifest_pressure: StagingManifestPressureSummary::default(),
            },
            scope,
        }
    }

    fn backend(source_scheme: &str, configured_max_bytes: u64) -> Self {
        Self {
            slice: StagingBudgetPolicySlice {
                backend_key: format!("backend:{source_scheme}"),
                library_id: None,
                library_name: None,
                backend_kind: backend_kind_from_scheme(source_scheme),
                source_scheme: source_scheme.to_owned(),
                configured_max_bytes,
                used_manifest_bytes: 0,
                manifest_pressure: StagingManifestPressureSummary::default(),
            },
            scope: StagingBudgetPolicyScope::backend(source_scheme),
        }
    }

    fn matches_source_scheme(&self, source_scheme: &str) -> bool {
        self.scope.matches_source_scheme(source_scheme)
    }

    fn record(&mut self, record: &StagingManifestRecord) {
        self.slice.used_manifest_bytes = self
            .slice
            .used_manifest_bytes
            .saturating_add(record.size_bytes.unwrap_or(0));
        self.slice.manifest_pressure.record(record);
    }

    fn into_slice(self) -> StagingBudgetPolicySlice {
        self.slice
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageStagingPressureStatus {
    Disabled,
    Healthy,
    Elevated,
    Critical,
    Exhausted,
}

impl StorageStagingPressureStatus {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Healthy => "healthy",
            Self::Elevated => "elevated",
            Self::Critical => "critical",
            Self::Exhausted => "exhausted",
        }
    }

    #[must_use]
    pub(crate) const fn blocks_library_scan(self) -> bool {
        matches!(self, Self::Critical | Self::Exhausted)
    }
}

#[must_use]
pub(crate) const fn storage_staging_pressure_status(
    configured_max_bytes: u64,
    used_manifest_bytes: u64,
) -> StorageStagingPressureStatus {
    if configured_max_bytes == 0 {
        return StorageStagingPressureStatus::Disabled;
    }

    let configured = configured_max_bytes as u128;
    let used = used_manifest_bytes as u128;

    if used >= configured {
        StorageStagingPressureStatus::Exhausted
    } else if used.saturating_mul(100) >= configured.saturating_mul(90) {
        StorageStagingPressureStatus::Critical
    } else if used.saturating_mul(100) >= configured.saturating_mul(75) {
        StorageStagingPressureStatus::Elevated
    } else {
        StorageStagingPressureStatus::Healthy
    }
}

impl StorageDiagnosticsAppService {
    pub(super) fn new(registry: StorageBackendRegistry) -> Self {
        Self {
            registry,
            repair_target_ref_secret: Arc::new(vfs_cache_repair_target_ref_secret()),
        }
    }

    pub(crate) async fn list_storage_backend_diagnostics(
        &self,
    ) -> StorageBackendDiagnosticsResponse {
        self.registry.diagnostics().await
    }

    pub(crate) async fn list_storage_backend_health(
        &self,
        page: PageRequest,
    ) -> Result<Vec<StorageBackendHealthRecord>> {
        self.registry
            .store
            .list_storage_backend_health(StorageBackendHealthListFilter::default(), page)
            .await
    }

    pub(crate) async fn reset_storage_backend_health(
        &self,
        backend_key: &str,
        reset_at_ms: i64,
    ) -> Result<Option<StorageBackendHealthRecord>> {
        self.registry
            .store
            .clear_storage_backend_health(backend_key, reset_at_ms)
            .await
    }

    pub(crate) async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        self.registry
            .store
            .list_staging_manifest_records(purpose, state, page)
            .await
    }

    pub(crate) async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        self.registry.store.sum_staging_manifest_bytes().await
    }

    pub(crate) async fn process_cached_backend_count(&self) -> usize {
        self.registry.backends.lock().await.len()
    }

    pub(crate) async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        self.registry.store.summarize_vfs_cache(now_ms).await
    }

    pub(crate) async fn latest_vfs_cache_repair_diagnostic(
        &self,
    ) -> Result<Option<VfsCacheRepairDiagnostic>> {
        let failure = self.registry.store.get_latest_vfs_cache_failure().await?;
        let Some(failure) = failure else {
            return Ok(None);
        };
        if self
            .registry
            .vfs_cache_failure_resolved_by_cache(&failure)
            .await?
        {
            return Ok(None);
        }

        Ok(Some(VfsCacheRepairDiagnostic::from_failure(&failure)))
    }

    pub(crate) async fn plan_latest_vfs_cache_repair_action(
        &self,
    ) -> Result<VfsCacheRepairActionPlanReport> {
        let repair = self.latest_vfs_cache_repair_diagnostic().await?;

        Ok(VfsCacheRepairActionPlanReport::from_repair(repair))
    }

    pub(crate) async fn list_vfs_cache_repair_targets(
        &self,
        page: PageRequest,
    ) -> Result<Vec<VfsCacheRepairTargetReport>> {
        let page = page.clamped();
        let mut targets = Vec::new();
        let mut repair_target_offset = 0_u64;
        let mut failure_offset = 0_u64;

        loop {
            let failures = self
                .registry
                .store
                .list_vfs_cache_failures(PageRequest::new(PageRequest::MAX_LIMIT, failure_offset))
                .await?;
            let failure_count = failures.len();
            if failure_count == 0 {
                break;
            }

            for failure in failures {
                let Some(target) = self.vfs_cache_repair_target_from_failure(failure).await? else {
                    continue;
                };
                if repair_target_offset < page.offset {
                    repair_target_offset += 1;
                    continue;
                }

                targets.push(target);
                if targets.len() >= page.limit as usize {
                    return Ok(targets);
                }
            }

            if failure_count < PageRequest::MAX_LIMIT as usize {
                break;
            }

            failure_offset = failure_offset.saturating_add(failure_count as u64);
        }

        Ok(targets)
    }

    pub(crate) async fn plan_vfs_cache_repair_remediation(
        &self,
    ) -> Result<VfsCacheRepairRemediationPlanReport> {
        let mut accumulator = VfsCacheRepairRemediationPlanAccumulator::default();
        let mut failure_offset = 0_u64;

        loop {
            let failures = self
                .registry
                .store
                .list_vfs_cache_failures(PageRequest::new(PageRequest::MAX_LIMIT, failure_offset))
                .await?;
            let failure_count = failures.len();
            if failure_count == 0 {
                break;
            }

            for failure in failures {
                let Some(target) = self.vfs_cache_repair_target_from_failure(failure).await? else {
                    continue;
                };
                accumulator.record(target);
            }

            if failure_count < PageRequest::MAX_LIMIT as usize {
                break;
            }

            failure_offset = failure_offset.saturating_add(failure_count as u64);
        }

        Ok(accumulator.into_report())
    }

    pub(crate) async fn preview_vfs_cache_repair_target(
        &self,
        target_ref: &str,
    ) -> Result<VfsCacheRepairTargetPreviewReport> {
        let (_failure, target) = self.vfs_cache_repair_target_failure(target_ref).await?;

        Ok(VfsCacheRepairTargetPreviewReport {
            plan: VfsCacheRepairActionPlanReport::from_target_preview_repair(target.repair.clone()),
            target,
        })
    }

    pub(crate) async fn refresh_latest_vfs_cache_repair(
        &self,
    ) -> Result<VfsCacheRepairRefreshActionReport> {
        let failure = self
            .registry
            .store
            .get_latest_vfs_cache_failure()
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "vfs_cache_failure",
                id: "latest".to_owned(),
            })?;
        if self
            .registry
            .vfs_cache_failure_resolved_by_cache(&failure)
            .await?
        {
            return Err(NakoError::NotFound {
                entity: "vfs_cache_failure",
                id: "latest".to_owned(),
            });
        }
        self.refresh_vfs_cache_repair_failure(
            failure,
            "vfs_cache_failure",
            "latest",
            "latest VFS cache repair diagnostic does not recommend refresh_cache",
        )
        .await
    }

    pub(crate) async fn refresh_vfs_cache_repair_target(
        &self,
        target_ref: &str,
    ) -> Result<VfsCacheRepairRefreshActionReport> {
        let (failure, _target) = self.vfs_cache_repair_target_failure(target_ref).await?;
        self.refresh_vfs_cache_repair_failure(
            failure,
            "vfs_cache_repair_target",
            "target_ref",
            "selected VFS cache repair target diagnostic does not recommend refresh_cache",
        )
        .await
    }

    pub(crate) async fn enqueue_vfs_cache_repair_target(
        &self,
        target_ref: &str,
        priority: Option<JobPriority>,
    ) -> Result<EnqueueVfsCacheRepairTargetOutcome> {
        let (failure, target) = self.vfs_cache_repair_target_failure(target_ref).await?;
        if target.repair.recommended_action != VfsCacheRepairAction::RefreshCache {
            return Err(NakoError::InvalidInput {
                message: "selected VFS cache repair target diagnostic does not recommend durable refresh_cache"
                    .to_owned(),
            });
        }

        let input = VfsCacheRepairJobInput::from_failure(&failure)?;
        if let Some(existing) = self
            .existing_incomplete_vfs_cache_repair_job(&input)
            .await?
        {
            return Ok(EnqueueVfsCacheRepairTargetOutcome::AlreadyQueued(existing));
        }

        let input_json = serde_json::to_string(&input).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize VFS cache repair job input: {err}"),
        })?;

        self.registry
            .store
            .enqueue_job(NewJob {
                id: nako_core::JobId::new(),
                kind: JobKind::VfsCacheRepair,
                resource_class: VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned(),
                priority: priority.unwrap_or_default(),
                library_id: failure.authority.library_id,
                source_id: None,
                input_json: Some(input_json),
            })
            .await
            .map(EnqueueVfsCacheRepairTargetOutcome::Enqueued)
    }

    pub(crate) async fn retry_vfs_cache_repair_job(
        &self,
        request: RetryVfsCacheRepairJobRequest,
    ) -> Result<Job> {
        let next_attempt_at =
            canonical_vfs_cache_repair_retry_next_attempt(&request.next_attempt_at)?;
        let source = self.vfs_cache_repair_job(request.job_id).await?;
        let input = vfs_cache_repair_job_input_from_job(&source)?;
        validate_vfs_cache_repair_job_bindings(&source, &input)?;
        if source.status != JobStatus::Failed {
            return Err(NakoError::Conflict {
                message: "only failed VFS cache repair jobs can be retried".to_owned(),
            });
        }

        self.vfs_cache_repair_failure_for_job_input(&input).await?;
        let max_attempts = request
            .max_attempts
            .unwrap_or_else(|| source.max_attempts.max(source.attempt.saturating_add(1)));

        self.registry
            .store
            .enqueue_job_retry(EnqueueJobRetry {
                source_job_id: source.id,
                retry_job_id: JobId::new(),
                max_attempts,
                next_attempt_at,
            })
            .await
    }

    pub(crate) async fn execute_vfs_cache_repair_job(
        &self,
        job_id: JobId,
    ) -> Result<VfsCacheRepairCommandOutput> {
        let runtime = DurableJobRuntime::new(self.registry.store.clone());
        let leased = runtime
            .claim_next_job_lease(JobLeaseClaimFilter {
                job_id: Some(job_id),
                kind: Some(JobKind::VfsCacheRepair),
                resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                ..JobLeaseClaimFilter::default()
            })
            .await?
            .ok_or_else(|| NakoError::Conflict {
                message: format!("job {job_id} is not queued and claimable"),
            })?;

        self.execute_claimed_vfs_cache_repair_job(leased).await
    }

    pub(crate) async fn execute_claimed_vfs_cache_repair_job(
        &self,
        leased: LeasedJob,
    ) -> Result<VfsCacheRepairCommandOutput> {
        let job = leased.job.clone();
        let runtime = DurableJobRuntime::new(self.registry.store.clone());
        let run = runtime
            .run_leased_job_with_trace_context(
                leased,
                "VFS cache repair job",
                None,
                |_context| async {
                    self.run_vfs_cache_repair_job_from_job(&job)
                        .await
                        .map_err(DurableJobOperationError::from)
                },
                vfs_cache_repair_job_summary_json,
            )
            .await?;

        match run {
            DurableJobRunOutcome::Completed(run) => Ok(VfsCacheRepairCommandOutput {
                job: run.job,
                summary: run.output,
            }),
            DurableJobRunOutcome::Cancelled(job) => Err(NakoError::Conflict {
                message: format!("job {} was cancelled", job.id),
            }),
        }
    }

    async fn existing_incomplete_vfs_cache_repair_job(
        &self,
        input: &VfsCacheRepairJobInput,
    ) -> Result<Option<Job>> {
        for status in [JobStatus::Queued, JobStatus::Running] {
            let mut offset = 0_u64;
            loop {
                let jobs = self
                    .registry
                    .store
                    .list_jobs(
                        JobListFilter {
                            status: Some(status),
                            kind: Some(JobKind::VfsCacheRepair),
                            resource_class: Some(VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned()),
                            library_id: input.authority.library_id,
                            source_id: None,
                        },
                        PageRequest::new(PageRequest::MAX_LIMIT, offset),
                    )
                    .await?;
                let job_count = jobs.len();
                if job_count == 0 {
                    break;
                }

                for job in jobs {
                    if vfs_cache_repair_job_input_from_job(&job)
                        .is_ok_and(|existing| existing == *input)
                    {
                        return Ok(Some(job));
                    }
                }

                if job_count < PageRequest::MAX_LIMIT as usize {
                    break;
                }

                offset = offset.saturating_add(job_count as u64);
            }
        }

        Ok(None)
    }

    async fn vfs_cache_repair_job(&self, job_id: JobId) -> Result<Job> {
        self.registry
            .store
            .get_job(job_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "job",
                id: job_id.to_string(),
            })
    }

    async fn run_vfs_cache_repair_job_from_job(
        &self,
        job: &Job,
    ) -> Result<VfsCacheRepairJobSummary> {
        let input = vfs_cache_repair_job_input_from_job(job)?;
        validate_vfs_cache_repair_job_bindings(job, &input)?;
        let failure = self.vfs_cache_repair_failure_for_job_input(&input).await?;
        let report = self
            .refresh_vfs_cache_repair_failure(
                failure,
                "vfs_cache_repair_target",
                "job_input",
                "VFS cache repair job target diagnostic does not recommend refresh_cache",
            )
            .await
            .map_err(|err| redact_vfs_cache_repair_execution_error(err, &input.source_scheme))?;

        Ok(VfsCacheRepairJobSummary::from_refresh_report(
            &input, &report,
        ))
    }

    async fn vfs_cache_repair_failure_for_job_input(
        &self,
        input: &VfsCacheRepairJobInput,
    ) -> Result<VfsCacheFailure> {
        let mut failure_offset = 0_u64;

        loop {
            let failures = self
                .registry
                .store
                .list_vfs_cache_failures(PageRequest::new(PageRequest::MAX_LIMIT, failure_offset))
                .await?;
            let failure_count = failures.len();
            if failure_count == 0 {
                break;
            }

            for failure in failures {
                if !input.matches_failure(&failure) {
                    continue;
                }

                let Some(target) = self
                    .vfs_cache_repair_target_from_failure(failure.clone())
                    .await?
                else {
                    return Err(vfs_cache_repair_job_target_not_found());
                };
                if target.repair.recommended_action != VfsCacheRepairAction::RefreshCache {
                    return Err(NakoError::InvalidInput {
                        message:
                            "VFS cache repair job target diagnostic does not recommend refresh_cache"
                                .to_owned(),
                    });
                }

                return Ok(failure);
            }

            if failure_count < PageRequest::MAX_LIMIT as usize {
                break;
            }

            failure_offset = failure_offset.saturating_add(failure_count as u64);
        }

        Err(vfs_cache_repair_job_target_not_found())
    }

    async fn vfs_cache_repair_target_failure(
        &self,
        target_ref: &str,
    ) -> Result<(VfsCacheFailure, VfsCacheRepairTargetReport)> {
        if !vfs_cache_repair_target_ref_is_valid(target_ref) {
            return Err(vfs_cache_repair_target_not_found());
        }

        let mut failure_offset = 0_u64;

        loop {
            let failures = self
                .registry
                .store
                .list_vfs_cache_failures(PageRequest::new(PageRequest::MAX_LIMIT, failure_offset))
                .await?;
            let failure_count = failures.len();
            if failure_count == 0 {
                break;
            }

            for failure in failures {
                let Some(target) = self
                    .vfs_cache_repair_target_from_failure(failure.clone())
                    .await?
                else {
                    continue;
                };
                if target.target_ref == target_ref {
                    return Ok((failure, target));
                }
            }

            if failure_count < PageRequest::MAX_LIMIT as usize {
                break;
            }

            failure_offset = failure_offset.saturating_add(failure_count as u64);
        }

        Err(vfs_cache_repair_target_not_found())
    }

    async fn refresh_vfs_cache_repair_failure(
        &self,
        failure: VfsCacheFailure,
        not_found_entity: &'static str,
        not_found_id: &'static str,
        invalid_action_message: &'static str,
    ) -> Result<VfsCacheRepairRefreshActionReport> {
        if self
            .registry
            .vfs_cache_failure_resolved_by_cache(&failure)
            .await?
        {
            return Err(NakoError::NotFound {
                entity: not_found_entity,
                id: not_found_id.to_owned(),
            });
        }
        let repair = VfsCacheRepairDiagnostic::from_failure(&failure);
        if repair.recommended_action != VfsCacheRepairAction::RefreshCache {
            return Err(NakoError::InvalidInput {
                message: invalid_action_message.to_owned(),
            });
        }

        let (uri, backend) = self
            .registry
            .backend_for_vfs_cache_failure(&failure)
            .await?;
        let refresh = backend.refresh_cache(&uri, failure.operation).await?;

        Ok(VfsCacheRepairRefreshActionReport {
            action: VfsCacheRepairAction::RefreshCache,
            operation: failure.operation,
            repair,
            refresh,
        })
    }

    async fn vfs_cache_repair_target_from_failure(
        &self,
        failure: VfsCacheFailure,
    ) -> Result<Option<VfsCacheRepairTargetReport>> {
        if self
            .registry
            .vfs_cache_failure_resolved_by_cache(&failure)
            .await?
        {
            return Ok(None);
        }

        let repair = VfsCacheRepairDiagnostic::from_failure(&failure);

        Ok(Some(VfsCacheRepairTargetReport {
            target_ref: self.vfs_cache_repair_target_ref(&failure),
            scheme: safe_storage_scheme(&failure.scheme),
            operation: failure.operation,
            failed_at_ms: failure.failed_at_ms,
            failure_count: failure.failure_count,
            repair,
        }))
    }

    fn vfs_cache_repair_target_ref(&self, failure: &VfsCacheFailure) -> String {
        let mut mac =
            VfsCacheRepairTargetRefMac::new_from_slice(self.repair_target_ref_secret.as_ref())
                .expect("HMAC accepts any key length");
        mac.update(b"nako:vfs-cache-repair-target:v1");
        mac_vfs_cache_repair_target_part(&mut mac, &failure.uri);
        mac_vfs_cache_repair_target_part(&mut mac, &failure.scheme);
        mac_vfs_cache_repair_target_part(&mut mac, failure.operation.as_str());
        mac_vfs_cache_repair_target_part(&mut mac, &failure.failed_at_ms.to_string());
        mac_vfs_cache_repair_target_part(&mut mac, &failure.failure_count.to_string());
        mac_vfs_cache_repair_target_part(
            &mut mac,
            failure
                .authority
                .library_id
                .map(|library_id| library_id.to_string())
                .as_deref()
                .unwrap_or_default(),
        );
        mac_vfs_cache_repair_target_part(
            &mut mac,
            failure.authority.backend_key.as_deref().unwrap_or_default(),
        );

        let digest = mac.finalize().into_bytes();
        let mut target_ref = String::with_capacity(38);
        target_ref.push_str("vfsrt_");
        for byte in &digest[..16] {
            let _ = write!(&mut target_ref, "{byte:02x}");
        }
        target_ref
    }

    pub(crate) async fn summarize_staging_cleanup_pressure(
        &self,
        now_ms: i64,
    ) -> Result<StagingCleanupPressureSummary> {
        let mut summary = StagingCleanupPressureSummary::default();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .registry
                .store
                .list_staging_cleanup_candidates(now_ms, page)
                .await?;
            let returned = records.len();

            for record in &records {
                summary.record(record);
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                summary.finish();
                return Ok(summary);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage staging cleanup diagnostics pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    pub(crate) async fn summarize_staging_manifest_pressure(
        &self,
    ) -> Result<StagingManifestPressureSummary> {
        let mut summary = StagingManifestPressureSummary::default();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .registry
                .store
                .list_staging_manifest_records(None, None, page)
                .await?;
            let returned = records.len();

            for record in &records {
                summary.record(record);
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                summary.finish();
                return Ok(summary);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message:
                            "storage staging pressure diagnostics pagination offset overflowed"
                                .to_owned(),
                    })?;
        }
    }

    pub(crate) async fn summarize_staging_budget_policy(
        &self,
    ) -> Result<Vec<StagingBudgetPolicySlice>> {
        self.registry.summarize_staging_budget_policy().await
    }

    #[cfg(test)]
    pub(super) async fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        self.registry.backend_for_library_root(library).await
    }

    #[cfg(test)]
    pub(crate) async fn replace_backend_for_test(
        &self,
        config: LocalLibraryConfig,
        backend: Arc<dyn StorageBackend>,
    ) {
        self.registry
            .replace_backend_for_test(config, backend)
            .await;
    }
}

#[derive(Clone, Debug)]
pub(super) struct StorageBackendRegistry {
    config: NakoServerConfig,
    store: NakoDatabase,
    playback: PlaybackConfig,
    backends: Arc<Mutex<HashMap<LibraryId, Arc<LibraryStorageBackend>>>>,
}

impl StorageBackendRegistry {
    pub(super) fn new(config: &NakoServerConfig, store: NakoDatabase) -> Self {
        Self {
            config: config.clone(),
            store,
            playback: config.playback,
            backends: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(super) async fn backend_for_library_root(
        &self,
        library: &Library,
    ) -> Result<Arc<LibraryStorageBackend>> {
        let config = configured_library_config_for(&self.config, library.id)?;
        self.backend_for_library_config(config).await
    }

    pub(super) async fn library_scan_admission_error(
        &self,
        library: &Library,
    ) -> Result<Option<NakoError>> {
        let backend = self.backend_for_library_root(library).await?;
        if let Some(err) = backend.library_scan_admission_error().await? {
            return Ok(Some(err));
        }

        self.library_scan_staging_pressure_admission_error(library)
            .await
    }

    pub(super) async fn backend_for_media_source(
        &self,
        source: &MediaSource,
    ) -> Result<(StorageUri, Arc<LibraryStorageBackend>)> {
        let uri = StorageUri::parse(&source.locator)?;
        let library_config = configured_library_config_for(&self.config, source.library_id)?;
        let backend = self.backend_for_library_config(library_config).await?;

        Ok((uri, backend))
    }

    async fn backend_for_vfs_cache_failure(
        &self,
        failure: &VfsCacheFailure,
    ) -> Result<(StorageUri, Arc<LibraryStorageBackend>)> {
        let uri = parse_vfs_cache_repair_target_uri(&failure.uri)?;
        if failure.authority.is_present() {
            return self
                .backend_for_vfs_cache_failure_authority(failure, uri)
                .await;
        }

        let mut matches = Vec::new();

        for config in &self.config.libraries {
            let root = cache_repair_root_uri(config)?;
            if storage_uri_matches_root(&uri, &root) {
                matches.push(config.clone());
            }
        }

        match matches.as_slice() {
            [] => Err(NakoError::NotFound {
                entity: "storage_backend",
                id: "vfs_cache_repair_target".to_owned(),
            }),
            [config] => Ok((uri, self.backend_for_library_config(config.clone()).await?)),
            _ => Err(NakoError::Conflict {
                message:
                    "latest VFS cache repair target matches multiple configured storage backends"
                        .to_owned(),
            }),
        }
    }

    async fn backend_for_vfs_cache_failure_authority(
        &self,
        failure: &VfsCacheFailure,
        uri: StorageUri,
    ) -> Result<(StorageUri, Arc<LibraryStorageBackend>)> {
        let (Some(library_id), Some(backend_key)) = (
            failure.authority.library_id,
            failure.authority.backend_key.as_deref(),
        ) else {
            return Err(NakoError::InvalidInput {
                message: "latest VFS cache repair target authority is incomplete".to_owned(),
            });
        };

        let Some(config) = self
            .config
            .libraries
            .iter()
            .find(|config| config.id == library_id)
            .cloned()
        else {
            return Err(NakoError::NotFound {
                entity: "storage_backend",
                id: "vfs_cache_repair_target".to_owned(),
            });
        };

        let expected_scheme = configured_library_scheme(&config);
        let expected_backend_key = storage_backend_key(config.id, &expected_scheme);
        if failure.scheme != expected_scheme
            || uri.scheme() != expected_scheme
            || backend_key != expected_backend_key
        {
            return Err(NakoError::Conflict {
                message: "latest VFS cache repair target authority does not match configured storage backend"
                    .to_owned(),
            });
        }

        Ok((uri, self.backend_for_library_config(config).await?))
    }

    async fn vfs_cache_failure_resolved_by_cache(&self, failure: &VfsCacheFailure) -> Result<bool> {
        for candidate in cache_repair_lookup_uris(&failure.uri) {
            match failure.operation {
                VfsCacheOperation::Stat => {
                    if let Some(object) = self.store.get_vfs_cache_object(&candidate).await? {
                        return Ok(object.fetched_at_ms >= failure.failed_at_ms);
                    }
                }
                VfsCacheOperation::List => {
                    if let Some(listing) = self.store.get_vfs_cache_listing(&candidate).await? {
                        return Ok(listing.fetched_at_ms >= failure.failed_at_ms);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn backend_for_library_config(
        &self,
        config: LocalLibraryConfig,
    ) -> Result<Arc<LibraryStorageBackend>> {
        let mut backends = self.backends.lock().await;
        if let Some(backend) = backends.get(&config.id) {
            return Ok(backend.clone());
        }

        let backend = Arc::new(LibraryStorageBackend::new(
            config.clone(),
            self.build_backend(&config)?,
            self.playback,
            self.store.clone(),
        ));
        backends.insert(config.id, backend.clone());

        Ok(backend)
    }

    pub(super) async fn diagnostics(&self) -> StorageBackendDiagnosticsResponse {
        let libraries = match self.list_all_libraries().await {
            Ok(libraries) => libraries,
            Err(err) => {
                return StorageBackendDiagnosticsResponse {
                    backends: vec![unavailable_registry_diagnostic(err)],
                };
            }
        };
        let mut backends = Vec::with_capacity(libraries.len());

        for library in libraries {
            let config = match configured_library_config_for(&self.config, library.id) {
                Ok(config) => config,
                Err(err) => {
                    let backend_kind = library_backend_kind(&library);
                    backends.push(unavailable_backend_diagnostic(
                        &library,
                        None,
                        Some(backend_kind),
                        safe_unavailable_reason(&err, backend_kind),
                    ));
                    continue;
                }
            };
            let root_uri = library
                .roots
                .first()
                .cloned()
                .unwrap_or_else(|| "local:///".to_owned());
            let backend_kind = backend_kind(&config);

            match self.backend_for_library_config(config).await {
                Ok(backend) => {
                    backends.push(backend.diagnostic(
                        library.id,
                        library.name,
                        root_uri,
                        backend_kind,
                    ));
                }
                Err(err) => {
                    let reason = safe_unavailable_reason(&err, backend_kind);
                    backends.push(unavailable_backend_diagnostic(
                        &library,
                        Some(root_uri),
                        Some(backend_kind),
                        reason,
                    ));
                }
            }
        }

        backends.sort_by(|left, right| left.library_name.cmp(&right.library_name));
        StorageBackendDiagnosticsResponse { backends }
    }

    async fn list_all_libraries(&self) -> Result<Vec<Library>> {
        let mut libraries = Vec::new();
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let mut batch = self.store.list_libraries(page).await?;
            let returned = batch.len();
            libraries.append(&mut batch);

            if returned < PageRequest::MAX_LIMIT as usize {
                return Ok(libraries);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage diagnostics library pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    async fn library_scan_staging_pressure_admission_error(
        &self,
        library: &Library,
    ) -> Result<Option<NakoError>> {
        if remote_probe_staging_root(library, &self.config).is_none() {
            return Ok(None);
        }

        let config = configured_library_config_for(&self.config, library.id)?;
        let Some(slice) = self.staging_budget_policy_slice_for_config(&config).await? else {
            return Ok(None);
        };
        let status =
            storage_staging_pressure_status(slice.configured_max_bytes, slice.used_manifest_bytes);
        if status.blocks_library_scan() {
            return Ok(Some(storage_staging_pressure_library_scan_error(status)));
        }

        Ok(None)
    }

    async fn summarize_staging_budget_policy(&self) -> Result<Vec<StagingBudgetPolicySlice>> {
        let mut policy = self
            .config
            .libraries
            .iter()
            .map(|config| {
                StagingBudgetPolicyAccumulator::configured(config, self.config.staging.max_bytes)
            })
            .collect::<Vec<_>>();
        let mut backend_policy = BTreeMap::<String, StagingBudgetPolicyAccumulator>::new();
        for slice in &policy {
            let source_scheme = slice.slice.source_scheme.clone();
            backend_policy
                .entry(source_scheme.clone())
                .or_insert_with(|| {
                    StagingBudgetPolicyAccumulator::backend(
                        &source_scheme,
                        self.config.staging.max_bytes,
                    )
                });
        }
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .store
                .list_staging_manifest_records(None, None, page)
                .await?;
            let returned = records.len();

            for record in &records {
                let source_scheme = staging_record_source_scheme(record);
                backend_policy
                    .entry(source_scheme.clone())
                    .or_insert_with(|| {
                        StagingBudgetPolicyAccumulator::backend(
                            &source_scheme,
                            self.config.staging.max_bytes,
                        )
                    })
                    .record(record);

                if let StagingAttribution::Attributed { library_id } = record.attribution {
                    if let Some(slice) = policy.iter_mut().find(|slice| {
                        slice.slice.library_id == Some(library_id)
                            && slice.matches_source_scheme(&source_scheme)
                    }) {
                        slice.record(record);
                    }
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                let mut slices = policy
                    .into_iter()
                    .map(StagingBudgetPolicyAccumulator::into_slice)
                    .chain(
                        backend_policy
                            .into_values()
                            .map(StagingBudgetPolicyAccumulator::into_slice),
                    )
                    .collect::<Vec<_>>();
                slices.sort_by(|left, right| left.backend_key.cmp(&right.backend_key));
                return Ok(slices);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage staging budget policy pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    async fn staging_budget_policy_slice_for_config(
        &self,
        config: &LocalLibraryConfig,
    ) -> Result<Option<StagingBudgetPolicySlice>> {
        let mut library_policy =
            StagingBudgetPolicyAccumulator::configured(config, self.config.staging.max_bytes);
        let mut backend_policy = StagingBudgetPolicyAccumulator::backend(
            &library_policy.slice.source_scheme,
            self.config.staging.max_bytes,
        );
        let mut saw_source_scheme_record = false;
        let mut offset = 0;

        loop {
            let page = PageRequest::new(PageRequest::MAX_LIMIT, offset);
            let records = self
                .store
                .list_staging_manifest_records(None, None, page)
                .await?;
            let returned = records.len();

            for record in &records {
                let source_scheme = staging_record_source_scheme(record);
                if backend_policy.matches_source_scheme(&source_scheme) {
                    backend_policy.record(record);
                    saw_source_scheme_record = true;
                }
                if record.attribution.library_id() == Some(config.id)
                    && library_policy.matches_source_scheme(&source_scheme)
                {
                    library_policy.record(record);
                }
            }

            if returned < PageRequest::MAX_LIMIT as usize {
                let library_slice = library_policy.into_slice();
                let backend_slice = backend_policy.into_slice();
                let backend_status = storage_staging_pressure_status(
                    backend_slice.configured_max_bytes,
                    backend_slice.used_manifest_bytes,
                );
                if saw_source_scheme_record && backend_status.blocks_library_scan() {
                    return Ok(Some(backend_slice));
                }
                if library_slice.manifest_pressure.total_records > 0 {
                    return Ok(Some(library_slice));
                }
                if saw_source_scheme_record {
                    return Ok(Some(backend_slice));
                }
                return Ok(None);
            }

            offset =
                offset
                    .checked_add(returned as u64)
                    .ok_or_else(|| NakoError::InvalidInput {
                        message: "storage staging admission policy pagination offset overflowed"
                            .to_owned(),
                    })?;
        }
    }

    fn build_backend(&self, config: &LocalLibraryConfig) -> Result<Arc<dyn StorageBackend>> {
        match config.webdav.as_ref() {
            Some(webdav) => self.webdav_storage_backend(config, webdav),
            None => Ok(Arc::new(LocalFsBackend::new(&config.root)?)),
        }
    }

    fn webdav_storage_backend(
        &self,
        library: &LocalLibraryConfig,
        config: &WebDavLibraryConfig,
    ) -> Result<Arc<dyn StorageBackend>> {
        let backend = nako_vfs::WebDavBackend::new(webdav_backend_config(config))?;
        let authority = VfsCacheFailureAuthority::attributed(
            library.id,
            storage_backend_key(library.id, backend.scheme()),
        );
        Ok(Arc::new(
            nako_vfs::CachedStorageBackend::new(backend, self.store.clone())
                .with_failure_authority(authority),
        ))
    }

    #[cfg(test)]
    async fn replace_backend_for_test(
        &self,
        config: LocalLibraryConfig,
        backend: Arc<dyn StorageBackend>,
    ) {
        self.backends.lock().await.insert(
            config.id,
            Arc::new(LibraryStorageBackend::new(
                config,
                backend,
                self.playback,
                self.store.clone(),
            )),
        );
    }
}

pub(super) struct LibraryStorageBackend {
    library_id: LibraryId,
    backend_key: String,
    scheme: String,
    store: NakoDatabase,
    inner: Arc<dyn StorageBackend>,
    stream_permits: Arc<Semaphore>,
    stream_permits_max: usize,
    stage_permits: Arc<Semaphore>,
    stage_permits_max: usize,
    health: Arc<StorageBackendHealth>,
    health_update_lock: Arc<Mutex<()>>,
}

impl LibraryStorageBackend {
    fn new(
        config: LocalLibraryConfig,
        inner: Arc<dyn StorageBackend>,
        playback: PlaybackConfig,
        store: NakoDatabase,
    ) -> Self {
        let stream_permits_max = playback.remote_stream_concurrency.max(1);
        let stage_permits_max = playback.remote_stage_concurrency.max(1);
        let scheme = inner.scheme().to_owned();

        Self {
            library_id: config.id,
            backend_key: storage_backend_key(config.id, &scheme),
            scheme,
            store,
            inner,
            stream_permits: Arc::new(Semaphore::new(stream_permits_max)),
            stream_permits_max,
            stage_permits: Arc::new(Semaphore::new(stage_permits_max)),
            stage_permits_max,
            health: Arc::new(StorageBackendHealth::new()),
            health_update_lock: Arc::new(Mutex::new(())),
        }
    }

    #[must_use]
    pub(super) fn library_id(&self) -> LibraryId {
        self.library_id
    }

    #[must_use]
    pub(super) fn stage_permits(&self) -> Arc<Semaphore> {
        self.stage_permits.clone()
    }

    #[must_use]
    pub(super) fn clone_backend(&self) -> Arc<dyn StorageBackend> {
        self.inner.clone()
    }

    #[must_use]
    pub(super) fn health(&self) -> Arc<StorageBackendHealth> {
        self.health.clone()
    }

    async fn library_scan_admission_error(&self) -> Result<Option<NakoError>> {
        self.durable_backoff_error().await
    }

    pub(super) fn try_acquire_stream_permit(&self) -> Result<OwnedSemaphorePermit> {
        self.stream_permits
            .clone()
            .try_acquire_owned()
            .map_err(|err| match err {
                TryAcquireError::NoPermits => NakoError::Conflict {
                    message: "playback resource remote_stream is busy".to_owned(),
                },
                TryAcquireError::Closed => NakoError::storage_resource_budget_closed(
                    format!("library:{}", self.library_id),
                    "remote stream resource budget was closed",
                ),
            })
    }

    #[cfg(test)]
    pub(super) fn available_stream_permits(&self) -> usize {
        self.stream_permits.available_permits()
    }

    fn diagnostic(
        &self,
        library_id: LibraryId,
        library_name: String,
        root_uri: String,
        backend_kind: StorageBackendKind,
    ) -> StorageBackendDiagnostic {
        let health = self.health.diagnostic();
        let status = if health.consecutive_errors == 0 {
            StorageBackendStatus::Ready
        } else {
            StorageBackendStatus::Degraded
        };

        StorageBackendDiagnostic {
            library_id,
            library_name,
            root_uri,
            backend_kind,
            scheme: self.scheme.clone(),
            status,
            reason: None,
            registry: StorageBackendRegistryDiagnostic {
                cached: true,
                stream_permits_available: self.stream_permits.available_permits(),
                stream_permits_max: self.stream_permits_max,
                stage_permits_available: self.stage_permits.available_permits(),
                stage_permits_max: self.stage_permits_max,
                state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
            },
            health,
        }
    }

    async fn record_success(&self) {
        let _guard = self.health_update_lock.lock().await;
        let now_ms = current_time_ms().unwrap_or_default();
        self.health.record_success(now_ms);
        let record = self.health_record(
            StorageBackendHealthStatus::Healthy,
            StorageCircuitBreakerState::Closed,
            0,
            Some(now_ms),
            None,
            None,
            None,
            None,
            None,
            now_ms,
        );
        let _ = self.store.upsert_storage_backend_health(record).await;
    }

    async fn record_error(&self, err: &NakoError) {
        let _guard = self.health_update_lock.lock().await;
        let now_ms = current_time_ms().unwrap_or_default();
        let class = err
            .storage_failure_class()
            .unwrap_or(StorageFailureClass::Unknown);
        self.health.record_error(err, now_ms);
        let diagnostic = self.health.diagnostic();
        let circuit_breaker_state = if class.is_retryable() && diagnostic.backoff_until_ms.is_some()
        {
            StorageCircuitBreakerState::Open
        } else {
            StorageCircuitBreakerState::Closed
        };
        let record = self.health_record(
            StorageBackendHealthStatus::Unavailable,
            circuit_breaker_state,
            u64_to_u32_saturating(diagnostic.consecutive_errors),
            diagnostic.last_success_at_ms,
            Some(now_ms),
            Some(class),
            Some(class.safe_message().to_owned()),
            (circuit_breaker_state == StorageCircuitBreakerState::Open).then_some(now_ms),
            diagnostic.backoff_until_ms,
            now_ms,
        );
        let _ = self.store.upsert_storage_backend_health(record).await;
    }

    fn health_record(
        &self,
        status: StorageBackendHealthStatus,
        circuit_breaker_state: StorageCircuitBreakerState,
        consecutive_failures: u32,
        last_success_at_ms: Option<i64>,
        last_failure_at_ms: Option<i64>,
        last_failure_class: Option<StorageFailureClass>,
        last_failure_safe_message: Option<String>,
        circuit_opened_at_ms: Option<i64>,
        backoff_until_ms: Option<i64>,
        updated_at_ms: i64,
    ) -> StorageBackendHealthRecord {
        StorageBackendHealthRecord {
            backend_key: self.backend_key.clone(),
            library_id: Some(self.library_id),
            scheme: self.scheme.clone(),
            status,
            circuit_breaker_state,
            consecutive_failures,
            last_success_at_ms,
            last_failure_at_ms,
            last_failure_class,
            last_failure_safe_message,
            circuit_opened_at_ms,
            backoff_until_ms,
            updated_at_ms,
        }
    }
}

#[derive(Debug)]
pub(super) struct StorageBackendHealth {
    last_success_at_ms: AtomicI64,
    last_error_at_ms: AtomicI64,
    last_error_class: AtomicU8,
    consecutive_errors: AtomicU64,
    backoff_until_ms: AtomicI64,
}

impl StorageBackendHealth {
    fn new() -> Self {
        Self {
            last_success_at_ms: AtomicI64::new(0),
            last_error_at_ms: AtomicI64::new(0),
            last_error_class: AtomicU8::new(0),
            consecutive_errors: AtomicU64::new(0),
            backoff_until_ms: AtomicI64::new(0),
        }
    }

    fn record_success(&self, now_ms: i64) {
        self.last_success_at_ms.store(now_ms, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
        self.backoff_until_ms.store(0, Ordering::Relaxed);
    }

    fn record_error(&self, err: &NakoError, now_ms: i64) {
        let class = err
            .storage_failure_class()
            .unwrap_or(StorageFailureClass::Unknown);
        self.last_error_at_ms.store(now_ms, Ordering::Relaxed);
        self.last_error_class
            .store(encode_storage_failure_class(class), Ordering::Relaxed);
        let consecutive_errors = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
        if class.is_retryable() {
            let backoff_until_ms = now_ms.saturating_add(storage_backoff_ms(consecutive_errors));
            self.backoff_until_ms
                .store(backoff_until_ms, Ordering::Relaxed);
        } else {
            self.backoff_until_ms.store(0, Ordering::Relaxed);
        }
    }

    #[cfg(test)]
    pub(super) fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(super) fn backoff_until_ms(&self) -> Option<i64> {
        timestamp_diagnostic(self.backoff_until_ms.load(Ordering::Relaxed))
    }

    fn backoff_error(&self, library_id: LibraryId) -> Option<NakoError> {
        let backoff_until_ms = self.backoff_until_ms.load(Ordering::Relaxed);
        if backoff_until_ms <= current_time_ms().unwrap_or_default() {
            return None;
        }

        Some(NakoError::storage_rate_limited(
            format!("library:{library_id}"),
            "storage backend is in process-local backoff",
        ))
    }

    fn diagnostic(&self) -> StorageBackendHealthDiagnostic {
        StorageBackendHealthDiagnostic {
            consecutive_errors: self.consecutive_errors.load(Ordering::Relaxed),
            last_success_at_ms: timestamp_diagnostic(
                self.last_success_at_ms.load(Ordering::Relaxed),
            ),
            last_error_at_ms: timestamp_diagnostic(self.last_error_at_ms.load(Ordering::Relaxed)),
            last_error_class: decode_storage_failure_class(
                self.last_error_class.load(Ordering::Relaxed),
            ),
            backoff_until_ms: timestamp_diagnostic(self.backoff_until_ms.load(Ordering::Relaxed)),
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for LibraryStorageBackend {
    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    async fn stat(&self, uri: &StorageUri) -> Result<nako_vfs::ObjectMetadata> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stat(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn list(&self, uri: &StorageUri) -> Result<Vec<nako_vfs::ObjectMetadata>> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.list(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn list_with_status(&self, uri: &StorageUri) -> Result<nako_vfs::ObjectListing> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.list_with_status(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn refresh_cache(
        &self,
        uri: &StorageUri,
        operation: VfsCacheOperation,
    ) -> Result<VfsCacheRefreshReport> {
        // Manual repair refresh is an explicit operator probe. It bypasses
        // admission backoff for one attempt but still records the outcome.
        let result = self.inner.refresh_cache(uri, operation).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn open_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::VirtualFile> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.open_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn read_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::ReadRange> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.read_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn stream_range(
        &self,
        uri: &StorageUri,
        range: Option<nako_vfs::ByteRange>,
    ) -> Result<nako_vfs::ReadStream> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stream_range(uri, range).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.read_to_string(uri).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn write_string(&self, uri: &StorageUri, content: &str) -> Result<()> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.write_string(uri, content).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn write(
        &self,
        request: nako_vfs::StorageWriteRequest,
    ) -> Result<nako_vfs::StorageWriteReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.write(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn stage(&self, request: nako_vfs::StageRequest) -> Result<nako_vfs::StagedFile> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.stage(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn plan_link(
        &self,
        request: nako_vfs::StorageLinkPlanRequest,
    ) -> Result<nako_vfs::StorageLinkPlan> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.plan_link(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn apply(
        &self,
        request: nako_vfs::StorageApplyRequest,
    ) -> Result<nako_vfs::StorageApplyReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.apply(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn cleanup(
        &self,
        request: nako_vfs::StorageCleanupRequest,
    ) -> Result<nako_vfs::StorageCleanupReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.cleanup(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }

    async fn restore(
        &self,
        request: nako_vfs::StorageRestoreRequest,
    ) -> Result<nako_vfs::StorageRestoreReport> {
        if let Some(result) = self.reject_if_backing_off().await {
            return result;
        }
        let result = self.inner.restore(request).await;
        self.record_result(result.as_ref().err().cloned()).await;
        result
    }
}

impl LibraryStorageBackend {
    async fn record_result(&self, err: Option<NakoError>) {
        match err {
            None => self.record_success().await,
            Some(err) => self.record_error(&err).await,
        }
    }

    async fn reject_if_backing_off<T>(&self) -> Option<Result<T>> {
        match self.durable_backoff_error().await {
            Ok(Some(err)) => return Some(Err(err)),
            Ok(None) => {}
            Err(err) => return Some(Err(err)),
        }

        let err = self.health.backoff_error(self.library_id)?;
        Some(Err(err))
    }

    async fn durable_health_record(&self) -> Result<Option<StorageBackendHealthRecord>> {
        self.store
            .get_storage_backend_health(&self.backend_key)
            .await
    }

    async fn durable_backoff_error(&self) -> Result<Option<NakoError>> {
        let Some(record) = self.durable_health_record().await? else {
            return Ok(None);
        };
        if record.circuit_breaker_state != StorageCircuitBreakerState::Open {
            return Ok(None);
        }
        let Some(backoff_until_ms) = record.backoff_until_ms else {
            return Ok(None);
        };
        if backoff_until_ms <= current_time_ms().unwrap_or_default() {
            return Ok(None);
        }

        Ok(Some(storage_circuit_breaker_open_error(self.library_id)))
    }
}

impl std::fmt::Debug for LibraryStorageBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LibraryStorageBackend")
            .field("library_id", &self.library_id)
            .field("scheme", &self.scheme)
            .field(
                "available_stream_permits",
                &self.stream_permits.available_permits(),
            )
            .field(
                "available_stage_permits",
                &self.stage_permits.available_permits(),
            )
            .field("stream_permits_max", &self.stream_permits_max)
            .field("stage_permits_max", &self.stage_permits_max)
            .field("health", &self.health)
            .finish()
    }
}

pub(super) fn webdav_backend_config(config: &WebDavLibraryConfig) -> nako_vfs::WebDavBackendConfig {
    nako_vfs::WebDavBackendConfig {
        base_url: config.base_url.clone(),
        username: config.username.clone(),
        password_env: config.password_env.clone(),
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
    }
}

fn backend_kind(config: &LocalLibraryConfig) -> StorageBackendKind {
    if config.webdav.is_some() {
        StorageBackendKind::WebDav
    } else {
        StorageBackendKind::Local
    }
}

fn configured_library_scheme(config: &LocalLibraryConfig) -> String {
    config
        .webdav
        .as_ref()
        .map_or_else(|| "local".to_owned(), |_| "webdav".to_owned())
}

fn backend_kind_from_scheme(scheme: &str) -> Option<StorageBackendKind> {
    match scheme {
        "local" => Some(StorageBackendKind::Local),
        "webdav" => Some(StorageBackendKind::WebDav),
        _ => None,
    }
}

fn safe_storage_scheme(scheme: &str) -> String {
    let normalized = scheme.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || !normalized
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        "unknown".to_owned()
    } else {
        normalized
    }
}

fn staging_record_source_scheme(record: &StagingManifestRecord) -> String {
    safe_storage_scheme(&record.source_scheme)
}

fn vfs_cache_repair_target_ref_secret() -> [u8; 32] {
    let first = uuid::Uuid::new_v4();
    let second = uuid::Uuid::new_v4();
    let mut secret = [0_u8; 32];
    secret[..16].copy_from_slice(first.as_bytes());
    secret[16..].copy_from_slice(second.as_bytes());
    secret
}

fn mac_vfs_cache_repair_target_part(mac: &mut VfsCacheRepairTargetRefMac, value: &str) {
    mac.update(value.as_bytes());
    mac.update(&[0]);
}

fn vfs_cache_repair_target_ref_is_valid(value: &str) -> bool {
    const PREFIX: &str = "vfsrt_";
    value.len() == PREFIX.len() + 32
        && value.starts_with(PREFIX)
        && value[PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
}

fn vfs_cache_repair_target_not_found() -> NakoError {
    NakoError::NotFound {
        entity: "vfs_cache_repair_target",
        id: "target_ref".to_owned(),
    }
}

fn vfs_cache_repair_job_target_not_found() -> NakoError {
    NakoError::NotFound {
        entity: "vfs_cache_repair_target",
        id: "job_input".to_owned(),
    }
}

fn vfs_cache_repair_job_summary_json(summary: &VfsCacheRepairJobSummary) -> Result<Option<String>> {
    DurableJobRuntime::serialize_summary(summary, "VFS cache repair job summary")
}

fn canonical_vfs_cache_repair_retry_next_attempt(
    next_attempt_at: &Option<String>,
) -> Result<Option<String>> {
    canonical_retry_next_attempt(
        next_attempt_at,
        "VFS cache repair retry next_attempt_at must be an RFC3339 timestamp",
        "VFS cache repair retry next_attempt_at could not be canonicalized",
    )
}

fn vfs_cache_repair_job_input_from_job(job: &Job) -> Result<VfsCacheRepairJobInput> {
    if job.kind != JobKind::VfsCacheRepair {
        return Err(NakoError::InvalidInput {
            message: "job is not a VFS cache repair job".to_owned(),
        });
    }
    if job.resource_class != VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job uses unsupported resource class".to_owned(),
        });
    }

    let input_json = job
        .input_json
        .as_deref()
        .ok_or_else(|| NakoError::InvalidInput {
            message: "VFS cache repair job input is missing".to_owned(),
        })?;
    let input: VfsCacheRepairJobInput =
        serde_json::from_str(input_json).map_err(|_err| NakoError::InvalidInput {
            message: "VFS cache repair job input is invalid".to_owned(),
        })?;

    VfsCacheRepairJobInput::new(
        input.action,
        input.source_scheme,
        input.operation,
        input.failed_at_ms,
        input.failure_count,
        input.uri_digest,
        input.authority,
    )
}

fn validate_vfs_cache_repair_job_bindings(job: &Job, input: &VfsCacheRepairJobInput) -> Result<()> {
    if job.library_id != input.authority.library_id {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job library binding does not match input".to_owned(),
        });
    }
    if job.source_id.is_some() {
        return Err(NakoError::InvalidInput {
            message: "VFS cache repair job must not include a source binding".to_owned(),
        });
    }

    Ok(())
}

fn redact_vfs_cache_repair_execution_error(err: NakoError, source_scheme: &str) -> NakoError {
    match err {
        NakoError::Storage { kind, .. } => NakoError::Storage {
            uri: format!("{source_scheme}://<redacted>"),
            kind,
            message: kind.failure_class().safe_message().to_owned(),
        },
        err => err,
    }
}

fn parse_vfs_cache_repair_target_uri(value: &str) -> Result<StorageUri> {
    StorageUri::parse(value).map_err(|_err| NakoError::InvalidInput {
        message: "latest VFS cache repair target URI is invalid".to_owned(),
    })
}

fn cache_repair_root_uri(config: &LocalLibraryConfig) -> Result<StorageUri> {
    match config.webdav.as_ref() {
        Some(webdav) => StorageUri::parse(&webdav.root).map_err(|_err| NakoError::InvalidInput {
            message: "configured VFS cache repair backend root URI is invalid".to_owned(),
        }),
        None => StorageUri::from_parts("local", ""),
    }
}

fn storage_uri_matches_root(uri: &StorageUri, root: &StorageUri) -> bool {
    if uri.scheme() != root.scheme() {
        return false;
    }

    let target_path = uri.path_part().trim_matches('/');
    let root_path = root.path_part().trim_matches('/');
    root_path.is_empty()
        || target_path == root_path
        || target_path
            .strip_prefix(root_path)
            .is_some_and(|remaining| remaining.starts_with('/'))
}

fn cache_repair_lookup_uris(uri: &str) -> Vec<String> {
    let mut candidates = vec![uri.to_owned()];
    let Ok(parsed) = StorageUri::parse(uri) else {
        return candidates;
    };
    if parsed.path_part().trim_matches('/').is_empty() {
        return candidates;
    }

    if uri.ends_with('/') {
        candidates.push(uri.trim_end_matches('/').to_owned());
    } else {
        candidates.push(format!("{uri}/"));
    }
    candidates
}

fn storage_staging_pressure_library_scan_error(status: StorageStagingPressureStatus) -> NakoError {
    NakoError::storage_staging_budget_exhausted(
        "staging://library-scan-admission",
        format!(
            "library scan admission blocked while staging pressure is {}",
            status.as_str()
        ),
    )
}

fn unavailable_backend_diagnostic(
    library: &Library,
    root_uri: Option<String>,
    backend_kind: Option<StorageBackendKind>,
    reason: String,
) -> StorageBackendDiagnostic {
    StorageBackendDiagnostic {
        library_id: library.id,
        library_name: library.name.clone(),
        root_uri: root_uri.unwrap_or_else(|| {
            library
                .roots
                .first()
                .cloned()
                .unwrap_or_else(|| "local:///".to_owned())
        }),
        backend_kind: backend_kind.unwrap_or(StorageBackendKind::Local),
        scheme: library
            .roots
            .first()
            .and_then(|root| StorageUri::parse(root).ok())
            .map(|uri| uri.scheme().to_owned())
            .unwrap_or_else(|| "unknown".to_owned()),
        status: StorageBackendStatus::Unavailable,
        reason: Some(reason),
        registry: StorageBackendRegistryDiagnostic {
            cached: false,
            stream_permits_available: 0,
            stream_permits_max: 0,
            stage_permits_available: 0,
            stage_permits_max: 0,
            state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
        },
        health: StorageBackendHealthDiagnostic {
            consecutive_errors: 0,
            last_success_at_ms: None,
            last_error_at_ms: None,
            last_error_class: None,
            backoff_until_ms: None,
        },
    }
}

fn library_backend_kind(library: &Library) -> StorageBackendKind {
    library
        .roots
        .first()
        .and_then(|root| StorageUri::parse(root).ok())
        .map(|uri| {
            if uri.scheme() == "webdav" {
                StorageBackendKind::WebDav
            } else {
                StorageBackendKind::Local
            }
        })
        .unwrap_or(StorageBackendKind::Local)
}

fn unavailable_registry_diagnostic(err: NakoError) -> StorageBackendDiagnostic {
    StorageBackendDiagnostic {
        library_id: LibraryId::new(),
        library_name: "Library registry".to_owned(),
        root_uri: "unknown:///".to_owned(),
        backend_kind: StorageBackendKind::Local,
        scheme: "unknown".to_owned(),
        status: StorageBackendStatus::Unavailable,
        reason: Some(safe_unavailable_reason(&err, StorageBackendKind::Local)),
        registry: StorageBackendRegistryDiagnostic {
            cached: false,
            stream_permits_available: 0,
            stream_permits_max: 0,
            stage_permits_available: 0,
            stage_permits_max: 0,
            state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
        },
        health: StorageBackendHealthDiagnostic {
            consecutive_errors: 0,
            last_success_at_ms: None,
            last_error_at_ms: None,
            last_error_class: None,
            backoff_until_ms: None,
        },
    }
}

fn safe_unavailable_reason(err: &NakoError, backend_kind: StorageBackendKind) -> String {
    match err {
        NakoError::InvalidInput { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend configuration is invalid",
            StorageBackendKind::WebDav => "WebDAV storage backend configuration is invalid",
        },
        NakoError::Storage { .. } => match backend_kind {
            StorageBackendKind::Local => "local storage backend could not be initialized",
            StorageBackendKind::WebDav => "WebDAV storage backend could not be initialized",
        },
        NakoError::NotFound { .. } => "configured library backend was not found",
        _ => "storage backend is unavailable",
    }
    .to_owned()
}

fn timestamp_diagnostic(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}

fn storage_backoff_ms(consecutive_errors: u64) -> i64 {
    const BASE_MS: i64 = 250;
    const MAX_MS: i64 = 30_000;

    let exponent = consecutive_errors.saturating_sub(1).min(7) as u32;
    BASE_MS.saturating_mul(2_i64.pow(exponent)).min(MAX_MS)
}

fn storage_backend_key(library_id: LibraryId, scheme: &str) -> String {
    format!("library:{library_id}:{scheme}")
}

fn storage_circuit_breaker_open_error(library_id: LibraryId) -> NakoError {
    NakoError::storage_rate_limited(
        format!("library:{library_id}"),
        "storage circuit breaker is open",
    )
}

fn u64_to_u32_saturating(value: u64) -> u32 {
    value.min(u64::from(u32::MAX)) as u32
}

fn encode_storage_failure_class(class: StorageFailureClass) -> u8 {
    match class {
        StorageFailureClass::Timeout => 1,
        StorageFailureClass::Unavailable => 2,
        StorageFailureClass::Permission => 3,
        StorageFailureClass::RateLimited => 4,
        StorageFailureClass::StaleCache => 5,
        StorageFailureClass::PartialRead => 6,
        StorageFailureClass::Budget => 7,
        StorageFailureClass::Security => 8,
        StorageFailureClass::Unknown => 9,
    }
}

fn decode_storage_failure_class(value: u8) -> Option<StorageFailureClass> {
    match value {
        1 => Some(StorageFailureClass::Timeout),
        2 => Some(StorageFailureClass::Unavailable),
        3 => Some(StorageFailureClass::Permission),
        4 => Some(StorageFailureClass::RateLimited),
        5 => Some(StorageFailureClass::StaleCache),
        6 => Some(StorageFailureClass::PartialRead),
        7 => Some(StorageFailureClass::Budget),
        8 => Some(StorageFailureClass::Security),
        9 => Some(StorageFailureClass::Unknown),
        _ => None,
    }
}

pub(super) fn remote_probe_staging_root(
    library: &Library,
    config: &NakoServerConfig,
) -> Option<PathBuf> {
    library
        .roots
        .iter()
        .any(|root| root.starts_with("webdav://"))
        .then(|| config.remux_staging_root.join("probe-inputs"))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use async_trait::async_trait;
    use nako_core::{
        DatabaseLifecycle, LibraryOptions, LibraryPreset, MediaItemId, MediaSourceId, NakoError,
        NewStagingManifestRecord, Result, StagingAttribution, StagingManifestId, StorageErrorKind,
        StorageFailureClass,
    };
    use nako_vfs::{
        ByteRange, ObjectMetadata, ReadRange, ReadStream, StageRequest, StagedFile, StorageBackend,
        StorageUri, VirtualFile,
    };
    use tempfile::tempdir;

    use super::*;
    use crate::config::{
        LocalLibraryConfig, NakoServerConfig, PlaybackConfig, StagingConfig,
        library_from_library_config,
    };

    #[test]
    fn vfs_cache_repair_action_plan_reports_no_action_without_repair() {
        let plan = VfsCacheRepairActionPlanReport::from_repair(None);

        assert_eq!(plan.status, VfsCacheRepairActionPlanStatus::NoAction);
        assert_eq!(plan.action, VfsCacheRepairAction::None);
        assert!(!plan.api_executable);
        assert_eq!(
            plan.reasons,
            vec![VfsCacheRepairActionPlanReason::NoRepairDiagnostic]
        );
        assert_eq!(plan.boundary, VfsCacheRepairActionBoundary::default());
        assert_eq!(plan.executable_route, None);
        assert!(plan.repair.is_none());
    }

    #[test]
    fn vfs_cache_repair_action_plan_reports_executable_refresh() {
        let repair = repair_diagnostic(VfsCacheRepairAction::RefreshCache);
        let plan = VfsCacheRepairActionPlanReport::from_repair(Some(repair.clone()));

        assert_eq!(plan.status, VfsCacheRepairActionPlanStatus::Executable);
        assert_eq!(plan.action, VfsCacheRepairAction::RefreshCache);
        assert!(plan.api_executable);
        assert_eq!(
            plan.reasons,
            vec![VfsCacheRepairActionPlanReason::RefreshCacheExecutable]
        );
        assert!(plan.boundary.refreshes_vfs_cache);
        assert!(!plan.boundary.changes_backend_configuration);
        assert!(!plan.boundary.requires_manual_failure_inspection);
        assert_eq!(
            plan.executable_route,
            Some(VfsCacheRepairExecutableRoute::LatestRefreshCache)
        );
        assert_eq!(plan.repair, Some(repair));
    }

    #[test]
    fn vfs_cache_repair_target_preview_reports_target_scoped_refresh_route() {
        let repair = repair_diagnostic(VfsCacheRepairAction::RefreshCache);
        let plan = VfsCacheRepairActionPlanReport::from_target_preview_repair(repair.clone());

        assert_eq!(plan.status, VfsCacheRepairActionPlanStatus::Executable);
        assert_eq!(plan.action, VfsCacheRepairAction::RefreshCache);
        assert!(plan.api_executable);
        assert_eq!(
            plan.reasons,
            vec![VfsCacheRepairActionPlanReason::RefreshCacheExecutable]
        );
        assert!(plan.boundary.refreshes_vfs_cache);
        assert_eq!(
            plan.executable_route,
            Some(VfsCacheRepairExecutableRoute::TargetRefreshCache)
        );
        assert_eq!(plan.repair, Some(repair));
    }

    #[test]
    fn vfs_cache_repair_action_plan_reports_backend_configuration_as_plan_only() {
        let repair = repair_diagnostic(VfsCacheRepairAction::FixBackendConfiguration);
        let plan = VfsCacheRepairActionPlanReport::from_repair(Some(repair.clone()));

        assert_eq!(plan.status, VfsCacheRepairActionPlanStatus::PlanOnly);
        assert_eq!(plan.action, VfsCacheRepairAction::FixBackendConfiguration);
        assert!(!plan.api_executable);
        assert_eq!(
            plan.reasons,
            vec![VfsCacheRepairActionPlanReason::BackendConfigurationRequired]
        );
        assert!(!plan.boundary.refreshes_vfs_cache);
        assert!(plan.boundary.changes_backend_configuration);
        assert!(!plan.boundary.requires_manual_failure_inspection);
        assert_eq!(plan.executable_route, None);
        assert_eq!(plan.repair, Some(repair));
    }

    #[test]
    fn vfs_cache_repair_action_plan_reports_inspect_failure_as_plan_only() {
        let repair = repair_diagnostic(VfsCacheRepairAction::InspectFailure);
        let plan = VfsCacheRepairActionPlanReport::from_repair(Some(repair.clone()));

        assert_eq!(plan.status, VfsCacheRepairActionPlanStatus::PlanOnly);
        assert_eq!(plan.action, VfsCacheRepairAction::InspectFailure);
        assert!(!plan.api_executable);
        assert_eq!(
            plan.reasons,
            vec![VfsCacheRepairActionPlanReason::ManualFailureInspectionRequired]
        );
        assert!(!plan.boundary.refreshes_vfs_cache);
        assert!(!plan.boundary.changes_backend_configuration);
        assert!(plan.boundary.requires_manual_failure_inspection);
        assert_eq!(plan.executable_route, None);
        assert_eq!(plan.repair, Some(repair));
    }

    fn repair_diagnostic(action: VfsCacheRepairAction) -> VfsCacheRepairDiagnostic {
        VfsCacheRepairDiagnostic {
            classification: match action {
                VfsCacheRepairAction::None => nako_vfs::VfsCacheRepairClassification::Healthy,
                VfsCacheRepairAction::RefreshCache => {
                    nako_vfs::VfsCacheRepairClassification::RetryableRefreshFailure
                }
                VfsCacheRepairAction::FixBackendConfiguration => {
                    nako_vfs::VfsCacheRepairClassification::OperatorActionRequired
                }
                VfsCacheRepairAction::InspectFailure => {
                    nako_vfs::VfsCacheRepairClassification::UnknownFailure
                }
            },
            recommended_action: action,
            state: None,
            operation: Some(VfsCacheOperation::Stat),
            failure_class: None,
            retryable: action == VfsCacheRepairAction::RefreshCache,
            failed_at_ms: Some(1_000),
            failure_count: Some(1),
            safe_message: Some("storage failure".to_owned()),
            operator_action: "operator guidance".to_owned(),
        }
    }

    #[tokio::test]
    async fn registry_reuses_library_backend_instances() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config.clone()],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        let registry = StorageBackendRegistry::new(&config, store);
        let library = library_from_library_config(&library_config);

        let first = registry.backend_for_library_root(&library).await.unwrap();
        let second = registry.backend_for_library_root(&library).await.unwrap();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.library_id(), library.id);
    }

    #[tokio::test]
    async fn registry_resolves_media_sources_by_library_id_only() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        let registry = StorageBackendRegistry::new(&config, store);
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: LibraryId::new(),
            item_id: MediaItemId::new(),
            locator: "local:///demo.mkv".to_owned(),
            file_name: "demo.mkv".to_owned(),
            size_bytes: None,
            fingerprint: None,
        };

        let err = registry
            .backend_for_media_source(&source)
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            NakoError::NotFound {
                entity: "library",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn library_backend_records_health_failures() {
        let temp = tempdir().unwrap();
        let store = migrated_store().await;
        let config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let backend = LibraryStorageBackend::new(
            config,
            Arc::new(FailingBackend),
            PlaybackConfig::default(),
            store,
        );
        let uri = StorageUri::parse("local:///demo.mkv").unwrap();

        assert!(backend.stat(&uri).await.is_err());
        assert!(backend.stat(&uri).await.is_err());
        assert_eq!(backend.health().consecutive_errors(), 2);
        assert!(backend.health().backoff_until_ms().is_none());
    }

    #[tokio::test]
    async fn library_backend_applies_process_local_backoff_after_retryable_storage_failure() {
        let temp = tempdir().unwrap();
        let store = migrated_store().await;
        let config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let failing = Arc::new(CountingFailingBackend::new(StorageErrorKind::Timeout));
        let backend =
            LibraryStorageBackend::new(config, failing.clone(), PlaybackConfig::default(), store);
        let uri = StorageUri::parse("local:///demo.mkv").unwrap();

        let first = backend.stat(&uri).await.unwrap_err();
        let second = backend.stat(&uri).await.unwrap_err();

        assert_eq!(
            first.storage_failure_class(),
            Some(StorageFailureClass::Timeout)
        );
        assert_eq!(
            second.storage_failure_class(),
            Some(StorageFailureClass::RateLimited)
        );
        assert_eq!(failing.stat_calls.load(Ordering::SeqCst), 1);
        assert_eq!(backend.health().consecutive_errors(), 1);
        assert!(backend.health().backoff_until_ms().is_some());
    }

    async fn migrated_store() -> NakoDatabase {
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
    }

    fn test_server_config(
        root: &std::path::Path,
        libraries: Vec<LocalLibraryConfig>,
    ) -> NakoServerConfig {
        NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: root.join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig {
                max_bytes: 100,
                ..StagingConfig::default()
            },
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries,
        }
    }

    async fn occupy_test_staging_manifest_bytes(
        store: &NakoDatabase,
        attribution: StagingAttribution,
        source_uri: &str,
        source_scheme: &str,
        size_bytes: u64,
    ) {
        let id = StagingManifestId::new();
        store
            .upsert_staging_manifest_record(NewStagingManifestRecord {
                id,
                attribution,
                source_uri: source_uri.to_owned(),
                source_scheme: source_scheme.to_owned(),
                purpose: StagingPurpose::ProbeInput,
                local_path: format!("/nako/private/staging/{source_scheme}-{id}.mkv"),
                size_bytes: Some(size_bytes),
                etag: Some("etag-secret".to_owned()),
                fingerprint: Some("fingerprint-secret".to_owned()),
                state: StagingState::Ready,
                created_at_ms: 1_000,
                updated_at_ms: 1_000,
                last_accessed_at_ms: 1_000,
                expires_at_ms: Some(10_000),
                active_leases: 0,
                validation_error: Some("raw backend error with token=secret".to_owned()),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn webdav_scan_admission_ignores_local_staging_pressure() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        };
        let config = test_server_config(temp.path(), vec![library_config.clone()]);
        let library = library_from_library_config(&library_config);
        let store = migrated_store().await;
        let registry = StorageBackendRegistry::new(&config, store.clone());
        occupy_test_staging_manifest_bytes(
            &store,
            StagingAttribution::unknown(),
            "local:///Private/scan-admission-fixture.mkv?token=secret",
            "local",
            95,
        )
        .await;

        let admission = registry
            .library_scan_admission_error(&library)
            .await
            .unwrap();
        let slices = registry.summarize_staging_budget_policy().await.unwrap();

        assert!(admission.is_none());
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == "backend:local")
                .map(|slice| slice.used_manifest_bytes),
            Some(95)
        );
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == "backend:webdav")
                .map(|slice| slice.used_manifest_bytes),
            Some(0)
        );
    }

    #[tokio::test]
    async fn webdav_scan_admission_blocks_matching_staging_pressure_without_raw_details() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        };
        let config = test_server_config(temp.path(), vec![library_config.clone()]);
        let library = library_from_library_config(&library_config);
        let store = migrated_store().await;
        let registry = StorageBackendRegistry::new(&config, store.clone());
        occupy_test_staging_manifest_bytes(
            &store,
            StagingAttribution::attributed(library.id),
            "webdav:///Movies/Private/scan-admission-fixture.mkv?token=secret",
            "webdav",
            95,
        )
        .await;

        let error = registry
            .library_scan_admission_error(&library)
            .await
            .unwrap()
            .expect("matching WebDAV staging pressure should block scan")
            .to_string();
        let slices = registry.summarize_staging_budget_policy().await.unwrap();

        assert!(
            error.contains("library scan admission blocked while staging pressure is critical")
        );
        assert!(!error.contains("webdav:///"));
        assert!(!error.contains("Private"));
        assert!(!error.contains("token=secret"));
        assert!(!error.contains("fingerprint-secret"));
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == format!("library:{}:webdav", library.id))
                .map(|slice| slice.used_manifest_bytes),
            Some(95)
        );
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == "backend:webdav")
                .map(|slice| slice.used_manifest_bytes),
            Some(95)
        );
    }

    #[tokio::test]
    async fn ambiguous_same_root_staging_pressure_does_not_inflate_library_slice() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        };
        let config = test_server_config(temp.path(), vec![library_config.clone()]);
        let library = library_from_library_config(&library_config);
        let store = migrated_store().await;
        let registry = StorageBackendRegistry::new(&config, store.clone());
        occupy_test_staging_manifest_bytes(
            &store,
            StagingAttribution::ambiguous(),
            "webdav:///Movies/Private/ambiguous-same-root.mkv?token=secret",
            "webdav",
            95,
        )
        .await;

        let slices = registry.summarize_staging_budget_policy().await.unwrap();

        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == format!("library:{}:webdav", library.id))
                .map(|slice| slice.used_manifest_bytes),
            Some(0)
        );
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == "backend:webdav")
                .map(|slice| slice.used_manifest_bytes),
            Some(95)
        );
    }

    #[tokio::test]
    async fn webdav_scan_admission_blocks_ambiguous_backend_pressure_with_library_records() {
        let temp = tempdir().unwrap();
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Remote Movies".to_owned(),
            root: temp.path().join("unused-local-root"),
            preset: LibraryPreset::Movies,
            webdav: Some(WebDavLibraryConfig {
                root: "webdav:///Movies".to_owned(),
                base_url: "https://webdav.example.test/dav".to_owned(),
                username: None,
                password_env: None,
                timeout_ms: 5_000,
                max_attempts: 1,
            }),
        };
        let config = test_server_config(temp.path(), vec![library_config.clone()]);
        let library = library_from_library_config(&library_config);
        let store = migrated_store().await;
        let registry = StorageBackendRegistry::new(&config, store.clone());
        occupy_test_staging_manifest_bytes(
            &store,
            StagingAttribution::attributed(library.id),
            "webdav:///Movies/Private/attributed-small.mkv?token=secret",
            "webdav",
            5,
        )
        .await;
        occupy_test_staging_manifest_bytes(
            &store,
            StagingAttribution::ambiguous(),
            "webdav:///Movies/Private/ambiguous-large.mkv?token=secret",
            "webdav",
            91,
        )
        .await;

        let error = registry
            .library_scan_admission_error(&library)
            .await
            .unwrap()
            .expect("backend ambiguous staging pressure should block scan")
            .to_string();
        let slices = registry.summarize_staging_budget_policy().await.unwrap();

        assert!(
            error.contains("library scan admission blocked while staging pressure is critical")
        );
        assert!(!error.contains("webdav:///"));
        assert!(!error.contains("Private"));
        assert!(!error.contains("token=secret"));
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == format!("library:{}:webdav", library.id))
                .map(|slice| slice.used_manifest_bytes),
            Some(5)
        );
        assert_eq!(
            slices
                .iter()
                .find(|slice| slice.backend_key == "backend:webdav")
                .map(|slice| slice.used_manifest_bytes),
            Some(96)
        );
    }

    #[tokio::test]
    async fn registry_diagnostics_redacts_unavailable_backend_details() {
        let temp = tempdir().unwrap();
        let missing_root = temp.path().join("missing-root");
        let library_config = LocalLibraryConfig {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            root: missing_root.clone(),
            preset: LibraryPreset::Movies,
            webdav: None,
        };
        let config = NakoServerConfig {
            database_backend: Default::default(),
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url: "sqlite::memory:".to_owned(),
            database_url_env: None,
            auth: crate::config::AuthConfig::disabled(),
            network: crate::config::NetworkAccessConfig::default(),
            ffprobe_path: PathBuf::from("ffprobe"),
            ffmpeg_path: PathBuf::from("ffmpeg"),
            scan_concurrency: 1,
            probe_concurrency: 1,
            metadata_concurrency: 1,
            remux_concurrency: 1,
            webhook_concurrency: 1,
            addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
            remux_timeout_ms: 1,
            remux_staging_root: temp.path().join("cache").join("remux"),
            metadata: Default::default(),
            transcode: Default::default(),
            staging: StagingConfig::default(),
            playback: PlaybackConfig::default(),
            artwork: crate::config::ArtworkConfig::default(),
            libraries: vec![library_config.clone()],
        };
        let store = NakoDatabase::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();
        store
            .upsert_library(&Library {
                id: library_config.id,
                name: library_config.name.clone(),
                roots: vec!["local:///".to_owned()],
                options: LibraryOptions::from_preset(library_config.preset),
            })
            .await
            .unwrap();
        let registry = StorageBackendRegistry::new(&config, store);

        let diagnostics = registry.diagnostics().await;

        assert_eq!(diagnostics.backends.len(), 1);
        let backend = &diagnostics.backends[0];
        assert_eq!(backend.status, StorageBackendStatus::Unavailable);
        assert_eq!(
            backend.reason.as_deref(),
            Some("local storage backend could not be initialized")
        );
        let serialized = serde_json::to_string(backend).unwrap();
        assert!(!serialized.contains(&missing_root.display().to_string()));
    }

    struct FailingBackend;

    #[async_trait]
    impl StorageBackend for FailingBackend {
        fn scheme(&self) -> &'static str {
            "local"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::storage_unknown(
                uri.to_string(),
                "intentional failure",
            ))
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Err(NakoError::storage_unknown(
                request.uri.to_string(),
                "intentional failure",
            ))
        }
    }

    struct CountingFailingBackend {
        kind: StorageErrorKind,
        stat_calls: AtomicU64,
    }

    impl CountingFailingBackend {
        fn new(kind: StorageErrorKind) -> Self {
            Self {
                kind,
                stat_calls: AtomicU64::new(0),
            }
        }
    }

    #[async_trait]
    impl StorageBackend for CountingFailingBackend {
        fn scheme(&self) -> &'static str {
            "local"
        }

        async fn stat(&self, uri: &StorageUri) -> Result<ObjectMetadata> {
            self.stat_calls.fetch_add(1, Ordering::SeqCst);
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn list(&self, uri: &StorageUri) -> Result<Vec<ObjectMetadata>> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn open_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<VirtualFile> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn read_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadRange> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn stream_range(
            &self,
            uri: &StorageUri,
            _range: Option<ByteRange>,
        ) -> Result<ReadStream> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn read_to_string(&self, uri: &StorageUri) -> Result<String> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn write_string(&self, uri: &StorageUri, _content: &str) -> Result<()> {
            Err(NakoError::storage(
                uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }

        async fn stage(&self, request: StageRequest) -> Result<StagedFile> {
            Err(NakoError::storage(
                request.uri.to_string(),
                self.kind,
                "counting failure",
            ))
        }
    }
}
