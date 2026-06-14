import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { RefreshCw } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type { AdminOverviewResponse } from "../../adminApi/types";
import { mockOverview } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/Table";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type OverviewPageProps = {
  dataSource: AdminDataSource;
};

type OverviewResult = {
  value: AdminOverviewResponse;
  source: DataSourceMode;
  error?: string;
};

type StorageBackend = AdminOverviewResponse["storage"]["backends"][number];
type MetadataProvider = AdminOverviewResponse["metadata"]["providers"][number];
type WatchFolderDiagnostic =
  AdminOverviewResponse["startup"]["watch_folder_runtime"]["diagnostics"][number];
type OperatorReadinessCheck =
  AdminOverviewResponse["operator_readiness"]["checks"][number];

export function OverviewPage({ dataSource }: OverviewPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-overview", locale],
    queryFn: () => loadOverview(dataSource, t("overview.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockOverview,
    source: "mock" as const,
  };
  const sourceFingerprintHash = result.value.source_fingerprint_hash;
  const watchFolderRuntime = result.value.startup.watch_folder_runtime;
  const watchFolderStats = watchFolderRuntimeStats(watchFolderRuntime.diagnostics);
  const storageColumns = createStorageColumns(t);
  const metadataColumns = createMetadataColumns(t);
  const watchFolderColumns = createWatchFolderColumns(t);
  const storageTable = useReactTable({
    data: result.value.storage.backends,
    columns: storageColumns,
    getCoreRowModel: getCoreRowModel(),
  });
  const metadataTable = useReactTable({
    data: result.value.metadata.providers,
    columns: metadataColumns,
    getCoreRowModel: getCoreRowModel(),
  });
  const watchFolderTable = useReactTable({
    data: watchFolderRuntime.diagnostics,
    columns: watchFolderColumns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <RoutePage
      actions={
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("overview.refresh")}
        </Button>
      }
      description={t("overview.description")}
      kicker={t("overview.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("overview.title")}
      titleId="overview-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("overview.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("overview.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="overviewMetricGrid">
            <OverviewMetric
              badge={
                result.value.status === "healthy"
                  ? t("overview.metric.serverStatus.healthy")
                  : t("overview.metric.serverStatus.degraded")
              }
              label={t("overview.metric.serverStatus.label")}
              value={result.value.status}
              tone={result.value.status === "healthy" ? "success" : "warning"}
            />
            <OverviewMetric
              badge={storageBadge(result.value, t)}
              label={t("overview.metric.storage.label")}
              value={t("overview.metric.storage.value", {
                ready: result.value.storage.ready_backends,
                total: result.value.storage.total_backends,
              })}
              tone={storageTone(result.value)}
            />
            <OverviewMetric
              badge={t("overview.metric.activeTasks.badge")}
              label={t("overview.metric.activeTasks.label")}
              value={result.value.runtime.active_tasks.toString()}
              tone="info"
            />
            <OverviewMetric
              badge={
                result.value.runtime.failed_jobs > 0
                  ? t("overview.metric.failedJobs.attention")
                  : t("overview.metric.failedJobs.clear")
              }
              label={t("overview.metric.failedJobs.label")}
              value={result.value.runtime.failed_jobs.toString()}
              tone={result.value.runtime.failed_jobs > 0 ? "warning" : "success"}
            />
            <OverviewMetric
              badge={t("overview.metric.configuredLibraries.badge")}
              label={t("overview.metric.configuredLibraries.label")}
              value={result.value.startup.configured_libraries.toString()}
              tone="neutral"
            />
            <OverviewMetric
              badge={t("overview.metric.recoveredJobs.badge")}
              label={t("overview.metric.recoveredJobs.label")}
              value={result.value.startup.recovered_jobs.toString()}
              tone="neutral"
            />
          </div>

          <DataPanel
            description={t("overview.operatorReadiness.description", {
              status: result.value.operator_readiness.status,
            })}
            title={t("overview.operatorReadiness.title")}
          >
            <div className="overviewReadinessGrid">
              {result.value.operator_readiness.checks.map((check) => (
                <OverviewReadinessItem
                  check={check}
                  key={check.area}
                  t={t}
                />
              ))}
            </div>
          </DataPanel>

          <DataPanel
            description={t("overview.sourceFingerprint.description", {
              fingerprinted: sourceFingerprintHash.fingerprinted_sources,
              total: sourceFingerprintHash.total_sources,
              contentHash: sourceFingerprintHash.content_hash_sources,
            })}
            title={t("overview.sourceFingerprint.title")}
          >
            <div className="overviewDiagnosticGrid">
              <OverviewDiagnosticStat
                detail={t("overview.sourceFingerprint.coverage.detail", {
                  contentHash: sourceFingerprintHash.content_hash_sources,
                })}
                label={t("overview.sourceFingerprint.coverage.label")}
                value={t("overview.sourceFingerprint.coverage.value", {
                  fingerprinted: sourceFingerprintHash.fingerprinted_sources,
                  total: sourceFingerprintHash.total_sources,
                })}
              />
              <OverviewDiagnosticStat
                detail={t("overview.sourceFingerprint.queue.detail", {
                  claimable: sourceFingerprintHash.claimable_jobs,
                  delayed: sourceFingerprintHash.delayed_retry_jobs,
                })}
                label={t("overview.sourceFingerprint.queue.label")}
                value={sourceFingerprintHash.queued_jobs.toString()}
              />
              <OverviewDiagnosticStat
                detail={t("overview.sourceFingerprint.failures.detail", {
                  running: sourceFingerprintHash.running_jobs,
                  succeeded: sourceFingerprintHash.succeeded_jobs,
                })}
                label={t("overview.sourceFingerprint.failures.label")}
                value={sourceFingerprintHash.failed_jobs.toString()}
              />
              <OverviewDiagnosticStat
                detail={t("overview.sourceFingerprint.retry.detail", {
                  oldest: formatOptionalTimestamp(sourceFingerprintHash.oldest_queued_at, t),
                })}
                label={t("overview.sourceFingerprint.retry.label")}
                value={formatOptionalTimestamp(sourceFingerprintHash.next_retry_at, t)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("overview.watchFolder.description", {
              realtime: watchFolderRuntime.realtime_enabled_libraries,
              skipped: watchFolderRuntime.skipped_libraries,
              started: watchFolderRuntime.started_libraries,
            })}
            title={t("overview.watchFolder.title")}
          >
            <div className="overviewDiagnosticGrid">
              <OverviewDiagnosticStat
                detail={t("overview.watchFolder.coverage.detail", {
                  skipped: watchFolderRuntime.skipped_libraries,
                })}
                label={t("overview.watchFolder.coverage.label")}
                value={t("overview.watchFolder.coverage.value", {
                  realtime: watchFolderRuntime.realtime_enabled_libraries,
                  started: watchFolderRuntime.started_libraries,
                })}
              />
              <OverviewDiagnosticStat
                detail={t("overview.watchFolder.tickCoverage.detail", {
                  never: watchFolderStats.neverTicked,
                })}
                label={t("overview.watchFolder.tickCoverage.label")}
                value={t("overview.watchFolder.tickCoverage.value", {
                  started: watchFolderRuntime.started_libraries,
                  ticked: watchFolderStats.ticked,
                })}
              />
              <OverviewDiagnosticStat
                detail={t("overview.watchFolder.admission.detail", {
                  notAdmitted: watchFolderStats.notAdmitted,
                  reused: watchFolderStats.reused,
                })}
                label={t("overview.watchFolder.admission.label")}
                value={watchFolderStats.enqueued.toString()}
              />
              <OverviewDiagnosticStat
                detail={t("overview.watchFolder.intake.detail", {
                  observed: watchFolderStats.observed,
                  suppressed: watchFolderStats.suppressed,
                })}
                label={t("overview.watchFolder.intake.label")}
                value={watchFolderStats.newlyReady.toString()}
              />
            </div>
            <OverviewTable table={watchFolderTable} />
          </DataPanel>

          <DataPanel
            description={t("overview.storage.description", {
              ready: result.value.storage.ready_backends,
              degraded: result.value.storage.degraded_backends,
              unavailable: result.value.storage.unavailable_backends,
            })}
            title={t("overview.metric.storage.label")}
          >
            <OverviewTable table={storageTable} />
          </DataPanel>

          <DataPanel
            description={t("overview.metadata.description", {
              available: result.value.metadata.available_providers,
              disabled: result.value.metadata.disabled_providers,
              unavailable: result.value.metadata.unavailable_providers,
            })}
            title={t("overview.metadata.title")}
          >
            <OverviewTable table={metadataTable} />
          </DataPanel>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadOverview(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<OverviewResult> {
  if (!dataSource.loadOverview) {
    return {
      value: mockOverview,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadOverview();
}

function OverviewMetric({
  badge,
  label,
  tone,
  value,
}: {
  badge: string;
  label: string;
  tone: "neutral" | "success" | "warning" | "danger" | "info";
  value: string;
}) {
  return (
    <div className="overviewMetric">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function OverviewDiagnosticStat({
  detail,
  label,
  value,
}: {
  detail: string;
  label: string;
  value: string;
}) {
  return (
    <div className="overviewDiagnosticStat">
      <span>{label}</span>
      <strong>{value}</strong>
      <small>{detail}</small>
    </div>
  );
}

function OverviewReadinessItem({
  check,
  t,
}: {
  check: OperatorReadinessCheck;
  t: Translate;
}) {
  const actionLabel = check.action
    ? operatorReadinessActionLabel(check.action.route_key, t)
    : null;
  const sourceReason = check.source_reason
    ? safeOperatorReadinessSourceReason(check.source_reason, t)
    : null;

  return (
    <div className="overviewReadinessItem">
      <div className="overviewReadinessHeader">
        <strong>{t(OPERATOR_READINESS_AREA_LABELS[check.area])}</strong>
        <Badge tone={operatorReadinessTone(check.status)}>
          {t(OPERATOR_READINESS_STATUS_LABELS[check.status])}
        </Badge>
      </div>
      <span>
        {t(OPERATOR_READINESS_REASON_LABELS[check.reason], {
          count: check.attention_count,
        })}
      </span>
      {sourceReason ? (
        <small>
          {t("overview.operatorReadiness.sourceReason", {
            reason: sourceReason,
          })}
        </small>
      ) : null}
      {actionLabel ? (
        <small>
          {t("overview.operatorReadiness.action", { route: actionLabel })}
        </small>
      ) : null}
    </div>
  );
}

function storageTone(
  overview: AdminOverviewResponse,
): "success" | "warning" | "danger" {
  if (overview.storage.unavailable_backends > 0) {
    return "danger";
  }

  if (overview.storage.degraded_backends > 0) {
    return "warning";
  }

  return "success";
}

function formatOptionalTimestamp(value: string | null, t: Translate): string {
  return value ?? t("overview.sourceFingerprint.timestamp.none");
}

function storageBadge(overview: AdminOverviewResponse, t: Translate) {
  if (overview.storage.unavailable_backends > 0) {
    return t("overview.metric.storage.unavailable");
  }

  if (overview.storage.degraded_backends > 0) {
    return t("overview.metric.storage.degraded");
  }

  return t("overview.metric.storage.ready");
}

function StatusBadge({ status }: { status: string }) {
  if (status === "ready" || status === "available" || status === "healthy") {
    return <Badge tone="success">{status}</Badge>;
  }

  if (status === "disabled" || status === "degraded") {
    return <Badge tone="warning">{status}</Badge>;
  }

  return <Badge tone="danger">{status}</Badge>;
}

function WatchFolderStatusBadge({
  label,
  tone,
}: {
  label: string;
  tone: "success" | "warning" | "danger";
}) {
  return <Badge tone={tone}>{label}</Badge>;
}

function OverviewTable<T>({ table }: { table: ReturnType<typeof useReactTable<T>> }) {
  return (
    <div className="tableScroll">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead key={header.id}>
                  {header.isPlaceholder
                    ? null
                    : flexRender(header.column.columnDef.header, header.getContext())}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows.map((row) => (
            <TableRow key={row.id}>
              {row.getVisibleCells().map((cell) => (
                <TableCell key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

const OPERATOR_READINESS_AREA_LABELS: Record<
  OperatorReadinessCheck["area"],
  MessageId
> = {
  setup: "overview.operatorReadiness.area.setup",
  media_library_scan: "overview.operatorReadiness.area.mediaLibraryScan",
  playback: "overview.operatorReadiness.area.playback",
  storage: "overview.operatorReadiness.area.storage",
  network: "overview.operatorReadiness.area.network",
  backup: "overview.operatorReadiness.area.backup",
};

const OPERATOR_READINESS_STATUS_LABELS: Record<
  OperatorReadinessCheck["status"],
  MessageId
> = {
  ready: "overview.operatorReadiness.status.ready",
  degraded: "overview.operatorReadiness.status.degraded",
  unavailable: "overview.operatorReadiness.status.unavailable",
};

const OPERATOR_READINESS_REASON_LABELS: Record<
  OperatorReadinessCheck["reason"],
  MessageId
> = {
  auth_configured: "overview.operatorReadiness.reason.authConfigured",
  auth_token_reference_missing:
    "overview.operatorReadiness.reason.authTokenReferenceMissing",
  auth_disabled_local_only:
    "overview.operatorReadiness.reason.authDisabledLocalOnly",
  auth_disabled_remote_exposure:
    "overview.operatorReadiness.reason.authDisabledRemoteExposure",
  media_library_configured:
    "overview.operatorReadiness.reason.mediaLibraryConfigured",
  no_media_library_configured:
    "overview.operatorReadiness.reason.noMediaLibraryConfigured",
  scan_work_pending: "overview.operatorReadiness.reason.scanWorkPending",
  scan_repair_pressure:
    "overview.operatorReadiness.reason.scanRepairPressure",
  watch_folder_runtime_coverage_gap:
    "overview.operatorReadiness.reason.watchFolderRuntimeCoverageGap",
  playback_ready: "overview.operatorReadiness.reason.playbackReady",
  playback_degraded: "overview.operatorReadiness.reason.playbackDegraded",
  playback_unavailable: "overview.operatorReadiness.reason.playbackUnavailable",
  storage_ready: "overview.operatorReadiness.reason.storageReady",
  storage_degraded: "overview.operatorReadiness.reason.storageDegraded",
  storage_unavailable: "overview.operatorReadiness.reason.storageUnavailable",
  vfs_cache_repair_pressure:
    "overview.operatorReadiness.reason.vfsCacheRepairPressure",
  network_ready: "overview.operatorReadiness.reason.networkReady",
  network_degraded: "overview.operatorReadiness.reason.networkDegraded",
  network_unavailable: "overview.operatorReadiness.reason.networkUnavailable",
  backup_runbook_available:
    "overview.operatorReadiness.reason.backupRunbookAvailable",
  backup_needs_durable_database:
    "overview.operatorReadiness.reason.backupNeedsDurableDatabase",
};

const OPERATOR_READINESS_ACTION_LABELS: Partial<Record<string, MessageId>> = {
  jobs: "overview.operatorReadiness.action.jobs",
  playbackRuntime: "overview.operatorReadiness.action.playbackRuntime",
  storageVfsCacheRepairTargets:
    "overview.operatorReadiness.action.storageRepair",
  systemConfig: "overview.operatorReadiness.action.systemConfig",
};

function operatorReadinessTone(
  status: OperatorReadinessCheck["status"],
): "success" | "warning" | "danger" {
  if (status === "ready") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

function operatorReadinessActionLabel(routeKey: string, t: Translate): string {
  const label = OPERATOR_READINESS_ACTION_LABELS[routeKey];
  return label ? t(label) : routeKey;
}

function safeOperatorReadinessSourceReason(value: string, t: Translate): string {
  if (/^[a-z0-9_.-]+$/.test(value)) {
    return value;
  }

  return t("overview.operatorReadiness.sourceReason.redacted");
}

function createStorageColumns(t: Translate): Array<ColumnDef<StorageBackend>> {
  return [
    {
      accessorKey: "library_name",
      header: t("overview.column.mediaLibrary"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.library_name}</strong>
          <span>{row.original.library_id}</span>
        </div>
      ),
    },
    {
      accessorKey: "backend_kind",
      header: t("overview.column.backend"),
    },
    {
      accessorKey: "status",
      header: t("overview.column.status"),
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
  ];
}

function createMetadataColumns(t: Translate): Array<ColumnDef<MetadataProvider>> {
  return [
    {
      accessorKey: "provider",
      header: t("overview.column.provider"),
      cell: ({ row }) => row.original.provider.toUpperCase(),
    },
    {
      accessorKey: "status",
      header: t("overview.column.status"),
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
  ];
}

function createWatchFolderColumns(t: Translate): Array<ColumnDef<WatchFolderDiagnostic>> {
  return [
    {
      accessorKey: "library_name",
      header: t("overview.column.mediaLibrary"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.library_name}</strong>
          <span>{row.original.library_id}</span>
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: t("overview.watchFolder.column.runtime"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <WatchFolderStatusBadge
            label={watchFolderCoverageStatusLabel(row.original.status, t)}
            tone={watchFolderCoverageTone(row.original.status)}
          />
          <span>
            {t("overview.watchFolder.root", {
              root: row.original.root_ref_redacted,
              scheme: row.original.root_scheme ?? t("overview.watchFolder.none"),
            })}
          </span>
          <span>
            {t("overview.watchFolder.safeReason", {
              reason: row.original.safe_reason,
            })}
          </span>
        </div>
      ),
    },
    {
      id: "last_tick",
      header: t("overview.watchFolder.column.lastTick"),
      cell: ({ row }) => {
        const tick = row.original.last_tick;
        if (!tick) {
          return t("overview.watchFolder.neverTicked");
        }

        return (
          <div className="routePrimaryCell">
            <strong>{watchFolderEnqueueReasonLabel(tick.enqueue_reason, t)}</strong>
            <span>
              {t("overview.watchFolder.counts", {
                failures: tick.failure_count,
                inspecting: tick.inspecting_candidates,
                ready: tick.ready_candidates,
                suppressed: tick.suppressed_candidates,
              })}
            </span>
          </div>
        );
      },
    },
    {
      id: "admission",
      header: t("overview.watchFolder.column.admission"),
      cell: ({ row }) => {
        const tick = row.original.last_tick;
        if (!tick) {
          return t("overview.watchFolder.none");
        }

        return (
          <div className="routePrimaryCell">
            <WatchFolderStatusBadge
              label={watchFolderAdmissionStatusLabel(tick.scan_admission_status, t)}
              tone={watchFolderAdmissionTone(tick.scan_admission_status)}
            />
            <span>
              {t("overview.watchFolder.scanJob", {
                job: tick.scan_job_id ?? t("overview.watchFolder.none"),
              })}
            </span>
            <span>
              {t("overview.watchFolder.reusedBackoff", {
                backoff: yesNo(tick.backoff_required, t),
                reused: yesNo(tick.reused_existing_scan, t),
              })}
            </span>
          </div>
        );
      },
    },
  ];
}

function watchFolderRuntimeStats(diagnostics: WatchFolderDiagnostic[]) {
  return diagnostics.reduce(
    (stats, diagnostic) => {
      const tick = diagnostic.last_tick;
      if (!tick) {
        stats.neverTicked += diagnostic.status === "started" ? 1 : 0;
        return stats;
      }

      stats.ticked += 1;
      stats.newlyReady += tick.newly_ready_candidates;
      stats.observed += tick.observed_candidates;
      stats.suppressed += tick.suppressed_candidates;
      if (tick.scan_admission_status === "enqueued") {
        stats.enqueued += 1;
      } else if (
        tick.scan_admission_status === "reused_queued"
        || tick.scan_admission_status === "reused_running"
      ) {
        stats.reused += 1;
      } else {
        stats.notAdmitted += 1;
      }

      return stats;
    },
    {
      enqueued: 0,
      neverTicked: 0,
      newlyReady: 0,
      notAdmitted: 0,
      observed: 0,
      reused: 0,
      suppressed: 0,
      ticked: 0,
    },
  );
}

function yesNo(value: boolean, t: Translate): string {
  return value ? t("overview.watchFolder.yes") : t("overview.watchFolder.no");
}

function watchFolderCoverageTone(
  status: WatchFolderDiagnostic["status"],
): "success" | "warning" | "danger" {
  if (status === "started") {
    return "success";
  }

  if (status === "disabled") {
    return "warning";
  }

  return "danger";
}

function watchFolderAdmissionTone(
  status: NonNullable<WatchFolderDiagnostic["last_tick"]>["scan_admission_status"],
): "success" | "warning" | "danger" {
  if (status === "enqueued" || status === "reused_queued" || status === "reused_running") {
    return "success";
  }

  return "warning";
}

function watchFolderCoverageStatusLabel(
  status: WatchFolderDiagnostic["status"],
  t: Translate,
): string {
  return t(WATCH_FOLDER_COVERAGE_STATUS_LABELS[status]);
}

function watchFolderAdmissionStatusLabel(
  status: NonNullable<WatchFolderDiagnostic["last_tick"]>["scan_admission_status"],
  t: Translate,
): string {
  return t(WATCH_FOLDER_ADMISSION_STATUS_LABELS[status]);
}

function watchFolderEnqueueReasonLabel(
  reason: NonNullable<WatchFolderDiagnostic["last_tick"]>["enqueue_reason"],
  t: Translate,
): string {
  return t(WATCH_FOLDER_ENQUEUE_REASON_LABELS[reason]);
}

const WATCH_FOLDER_COVERAGE_STATUS_LABELS: Record<
  WatchFolderDiagnostic["status"],
  MessageId
> = {
  disabled: "overview.watchFolder.coverageStatus.disabled",
  missing_root: "overview.watchFolder.coverageStatus.missingRoot",
  started: "overview.watchFolder.coverageStatus.started",
  unsupported_root: "overview.watchFolder.coverageStatus.unsupportedRoot",
};

const WATCH_FOLDER_ADMISSION_STATUS_LABELS: Record<
  NonNullable<WatchFolderDiagnostic["last_tick"]>["scan_admission_status"],
  MessageId
> = {
  enqueued: "overview.watchFolder.admissionStatus.enqueued",
  not_admitted: "overview.watchFolder.admissionStatus.notAdmitted",
  reused_queued: "overview.watchFolder.admissionStatus.reusedQueued",
  reused_running: "overview.watchFolder.admissionStatus.reusedRunning",
};

const WATCH_FOLDER_ENQUEUE_REASON_LABELS: Record<
  NonNullable<WatchFolderDiagnostic["last_tick"]>["enqueue_reason"],
  MessageId
> = {
  blocked_candidates: "overview.watchFolder.enqueueReason.blockedCandidates",
  discovery_failures: "overview.watchFolder.enqueueReason.discoveryFailures",
  new_stable_candidates:
    "overview.watchFolder.enqueueReason.newStableCandidates",
  no_new_stable_candidates:
    "overview.watchFolder.enqueueReason.noNewStableCandidates",
  suppressed_candidates:
    "overview.watchFolder.enqueueReason.suppressedCandidates",
  waiting_for_stability:
    "overview.watchFolder.enqueueReason.waitingForStability",
};
