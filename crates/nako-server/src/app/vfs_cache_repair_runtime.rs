use std::time::Duration;

use nako_core::{JobPriority, NakoError, Result};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::config::VfsCacheRepairAutomationRuntimeConfig;

use super::{
    jobs::{LibraryScanAppService, LibraryScanScheduleOutcome},
    runtime::RuntimeSupervisor,
    storage::{
        StorageDiagnosticsAppService, VfsCacheRepairAutomationBoundary,
        VfsCacheRepairAutomationEnqueueReport, VfsCacheRepairAutomationPolicy,
    },
};

const VFS_CACHE_REPAIR_AUTOMATION_RUNTIME_RESOURCE_CLASS: &str =
    "storage.vfs.cache_repair.automation";

#[derive(Clone, Debug)]
pub(crate) struct VfsCacheRepairAutomationRuntimeAppService {
    storage: StorageDiagnosticsAppService,
    library_scan: LibraryScanAppService,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VfsCacheRepairAutomationRuntimeTickDiagnostic {
    pub(crate) enabled: bool,
    pub(crate) total_unresolved_targets: u32,
    pub(crate) eligible_targets: u32,
    pub(crate) blocked_targets: u32,
    pub(crate) enqueued_count: u32,
    pub(crate) already_queued_count: u32,
    pub(crate) scheduler_outcome: Option<LibraryScanScheduleOutcome>,
    pub(crate) boundary: VfsCacheRepairAutomationBoundary,
}

impl VfsCacheRepairAutomationRuntimeAppService {
    pub(crate) fn new(
        storage: StorageDiagnosticsAppService,
        library_scan: LibraryScanAppService,
    ) -> Self {
        Self {
            storage,
            library_scan,
        }
    }

    pub(super) fn start_recurring_automation(
        &self,
        config: VfsCacheRepairAutomationRuntimeConfig,
        runtime: &RuntimeSupervisor,
    ) -> bool {
        if !config.enabled {
            return false;
        }

        let service = self.clone();
        let shutdown = runtime.shutdown_token();
        runtime.spawn(
            "vfs_cache_repair_automation_runtime",
            VFS_CACHE_REPAIR_AUTOMATION_RUNTIME_RESOURCE_CLASS,
            async move {
                run_vfs_cache_repair_automation_loop(service, config, shutdown).await;
            },
        );

        true
    }

    pub(crate) async fn run_vfs_cache_repair_automation_tick(
        &self,
        config: VfsCacheRepairAutomationRuntimeConfig,
    ) -> Result<VfsCacheRepairAutomationRuntimeTickDiagnostic> {
        if !config.enabled {
            return Ok(VfsCacheRepairAutomationRuntimeTickDiagnostic::disabled());
        }

        let report = self
            .storage
            .enqueue_vfs_cache_repair_automation(
                VfsCacheRepairAutomationPolicy { enabled: true },
                Some(JobPriority::Low),
            )
            .await?;
        let should_schedule = report.enqueued_count > 0 || report.already_queued_count > 0;
        let mut diagnostic = VfsCacheRepairAutomationRuntimeTickDiagnostic::from(report);
        if should_schedule {
            diagnostic.scheduler_outcome =
                Some(self.library_scan.schedule_queued_library_scans().await?);
        }

        Ok(diagnostic)
    }
}

async fn run_vfs_cache_repair_automation_loop(
    service: VfsCacheRepairAutomationRuntimeAppService,
    config: VfsCacheRepairAutomationRuntimeConfig,
    shutdown: CancellationToken,
) {
    while !shutdown.is_cancelled() {
        let sleep_for = match service.run_vfs_cache_repair_automation_tick(config).await {
            Ok(diagnostic) => {
                if diagnostic.enqueued_count > 0 || diagnostic.already_queued_count > 0 {
                    info!(
                        total_unresolved_targets = diagnostic.total_unresolved_targets,
                        eligible_targets = diagnostic.eligible_targets,
                        blocked_targets = diagnostic.blocked_targets,
                        enqueued_count = diagnostic.enqueued_count,
                        already_queued_count = diagnostic.already_queued_count,
                        scheduler_outcome = ?diagnostic.scheduler_outcome,
                        "VFS cache repair automation enqueued durable repair jobs"
                    );
                }
                vfs_cache_repair_automation_runtime_interval(config)
            }
            Err(err) => {
                let safe_error = vfs_cache_repair_automation_runtime_safe_error_message(&err);
                warn!(
                    error = %safe_error,
                    "VFS cache repair automation tick failed"
                );
                vfs_cache_repair_automation_runtime_error_backoff(config)
            }
        };

        tokio::select! {
            () = shutdown.cancelled() => break,
            () = tokio::time::sleep(sleep_for) => {}
        }
    }
}

pub(super) fn vfs_cache_repair_automation_runtime_interval(
    config: VfsCacheRepairAutomationRuntimeConfig,
) -> Duration {
    Duration::from_millis(config.interval_ms.max(1))
}

pub(super) fn vfs_cache_repair_automation_runtime_error_backoff(
    config: VfsCacheRepairAutomationRuntimeConfig,
) -> Duration {
    Duration::from_millis(config.error_backoff_ms.max(1))
}

pub(super) fn vfs_cache_repair_automation_runtime_safe_error_message(err: &NakoError) -> String {
    match err {
        NakoError::NotFound { entity, .. } => format!("{entity} was not found"),
        NakoError::InvalidInput { .. } => "invalid VFS cache repair automation input".to_owned(),
        NakoError::Conflict { .. } => "VFS cache repair automation conflict".to_owned(),
        NakoError::Unauthorized { .. } => {
            "VFS cache repair automation access is unauthorized".to_owned()
        }
        NakoError::Forbidden { .. } => "VFS cache repair automation access is forbidden".to_owned(),
        NakoError::Unsupported(_) => {
            "VFS cache repair automation operation is unsupported".to_owned()
        }
        NakoError::Provider { .. } => "provider error".to_owned(),
        NakoError::Storage { kind, .. } => format!("storage error: {kind:?}"),
        NakoError::Database { .. } => "database error".to_owned(),
    }
}

impl VfsCacheRepairAutomationRuntimeTickDiagnostic {
    fn disabled() -> Self {
        Self {
            enabled: false,
            total_unresolved_targets: 0,
            eligible_targets: 0,
            blocked_targets: 0,
            enqueued_count: 0,
            already_queued_count: 0,
            scheduler_outcome: None,
            boundary: VfsCacheRepairAutomationBoundary {
                reads_repair_targets: false,
                may_start_durable_jobs: false,
                refreshes_vfs_cache: false,
                changes_backend_configuration: false,
                deletes_cache_entries: false,
                writes_library_files: false,
            },
        }
    }
}

impl From<VfsCacheRepairAutomationEnqueueReport> for VfsCacheRepairAutomationRuntimeTickDiagnostic {
    fn from(value: VfsCacheRepairAutomationEnqueueReport) -> Self {
        Self {
            enabled: value.policy_report.policy.enabled,
            total_unresolved_targets: value.policy_report.total_unresolved_targets,
            eligible_targets: value.policy_report.eligible_targets.len() as u32,
            blocked_targets: value.policy_report.blocked_targets.len() as u32,
            enqueued_count: value.enqueued_count,
            already_queued_count: value.already_queued_count,
            scheduler_outcome: None,
            boundary: value.policy_report.boundary,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nako_core::{NakoError, StorageErrorKind};

    use crate::config::VfsCacheRepairAutomationRuntimeConfig;

    use super::{
        vfs_cache_repair_automation_runtime_error_backoff,
        vfs_cache_repair_automation_runtime_interval,
        vfs_cache_repair_automation_runtime_safe_error_message,
    };

    #[test]
    fn vfs_cache_repair_automation_runtime_delays_are_bounded_to_at_least_one_ms() {
        let config = VfsCacheRepairAutomationRuntimeConfig {
            enabled: true,
            interval_ms: 0,
            error_backoff_ms: 0,
        };

        assert_eq!(
            vfs_cache_repair_automation_runtime_interval(config),
            Duration::from_millis(1)
        );
        assert_eq!(
            vfs_cache_repair_automation_runtime_error_backoff(config),
            Duration::from_millis(1)
        );
    }

    #[test]
    fn vfs_cache_repair_automation_runtime_error_summary_redacts_storage_details() {
        let error = NakoError::storage(
            "local:///Users/ExampleUser/Secret Path/Hidden Movie.mkv?token=secret",
            StorageErrorKind::Network,
            "failed to inspect Secret Path token=secret",
        );

        let safe = vfs_cache_repair_automation_runtime_safe_error_message(&error);

        assert_eq!(safe, "storage error: Network");
        assert!(!safe.contains("Hidden Movie"));
        assert!(!safe.contains("Secret Path"));
        assert!(!safe.contains("ExampleUser"));
        assert!(!safe.contains("token"));
        assert!(!safe.contains("local:///"));
    }
}
