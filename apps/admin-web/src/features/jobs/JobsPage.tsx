import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Fingerprint, Play, RefreshCw, RotateCcw, Search, Wrench, X } from "lucide-react";
import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminJobListItem,
  AdminJobQueuePressureSummary,
  AdminJobListResponse,
  AdminJobsQuery,
} from "../../adminApi/types";
import { mockJobs } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
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

export type JobsSearch = {
  status?: string;
  kind?: string;
  resource_class?: string;
  library_id?: string;
  source_id?: string;
  limit: number;
  offset: number;
};

export type JobsPageProps = {
  dataSource: AdminDataSource;
  search: JobsSearch;
  onSearchChange(next: Partial<JobsSearch>): void;
};

type JobsResult = {
  value: AdminJobListResponse;
  source: DataSourceMode;
  error?: string;
};

type VfsCacheRepairJobAction = "execute" | "retry";
type VfsCacheRepairJobCommand = {
  action: VfsCacheRepairJobAction;
  job: AdminJobListItem;
};

const SOURCE_FINGERPRINT_HASH_JOB_KIND = "source_fingerprint_hash";
const SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS = "disk.scan.source_fingerprint_hash";
const VFS_CACHE_REPAIR_JOB_KIND = "vfs_cache_repair";
const VFS_CACHE_REPAIR_RESOURCE_CLASS = "storage.vfs.cache_repair";

export function JobsPage({ dataSource, search, onSearchChange }: JobsPageProps) {
  const { locale, t } = useI18n();
  const queryClient = useQueryClient();
  const [commandMessage, setCommandMessage] = useState<string | null>(null);
  const [commandError, setCommandError] = useState<string | null>(null);
  const query = useQuery({
    queryKey: ["admin-jobs", search, locale],
    queryFn: () => loadJobs(dataSource, search, t("jobs.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockJobs,
    source: "mock" as const,
  };
  const commandMutation = useMutation({
    mutationFn: async (command: VfsCacheRepairJobCommand) => {
      if (result.source !== "live") {
        throw new Error(t("jobs.vfsCacheRepair.notLiveError"));
      }
      if (!isVfsCacheRepairJob(command.job)) {
        throw new Error(t("jobs.vfsCacheRepair.notRepairJob"));
      }

      if (command.action === "execute") {
        if (command.job.status !== "queued") {
          throw new Error(t("jobs.vfsCacheRepair.executeInvalidState"));
        }
        if (!dataSource.executeVfsCacheRepairJob) {
          throw new Error(t("jobs.vfsCacheRepair.executeUnavailable"));
        }

        return {
          action: command.action,
          job: await dataSource.executeVfsCacheRepairJob(command.job.id),
        };
      }

      if (command.job.status !== "failed") {
        throw new Error(t("jobs.vfsCacheRepair.retryInvalidState"));
      }
      if (!dataSource.retryVfsCacheRepairJob) {
        throw new Error(t("jobs.vfsCacheRepair.retryUnavailable"));
      }

      return {
        action: command.action,
        job: await dataSource.retryVfsCacheRepairJob(command.job.id),
      };
    },
    onMutate: () => {
      setCommandMessage(null);
      setCommandError(null);
    },
    onSuccess: (response) => {
      if (response.action === "execute") {
        setCommandMessage(
          t("jobs.vfsCacheRepair.executeSucceeded", {
            jobId: response.job.job.id,
            status: response.job.job.status,
          }),
        );
      } else {
        setCommandMessage(
          t("jobs.vfsCacheRepair.retrySucceeded", {
            jobId: response.job.id,
            status: response.job.status,
          }),
        );
      }
      void queryClient.invalidateQueries({ queryKey: ["admin-jobs"] });
    },
    onError: (error) => {
      setCommandError(errorMessage(error, t("jobs.vfsCacheRepair.operationFailed")));
    },
  });
  const columns = createColumns(t, {
    canExecute: result.source === "live" && Boolean(dataSource.executeVfsCacheRepairJob),
    canRetry: result.source === "live" && Boolean(dataSource.retryVfsCacheRepairJob),
    isPending: commandMutation.isPending,
    pendingAction: commandMutation.variables?.action ?? null,
    pendingJobId: commandMutation.variables?.job.id ?? null,
    runCommand: (command) => commandMutation.mutate(command),
  });

  const table = useReactTable({
    data: result.value.jobs,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });
  const activeFilterCount = useMemo(
    () =>
      [
        search.status,
        search.kind,
        search.resource_class,
        search.library_id,
        search.source_id,
      ].filter(Boolean).length,
    [search],
  );
  const sourceHashFilterActive =
    search.kind === SOURCE_FINGERPRINT_HASH_JOB_KIND &&
    search.resource_class === SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS;
  const vfsCacheRepairFilterActive =
    search.kind === VFS_CACHE_REPAIR_JOB_KIND &&
    search.resource_class === VFS_CACHE_REPAIR_RESOURCE_CLASS;

  return (
    <RoutePage
      actions={
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("jobs.refresh")}
        </Button>
      }
      description={t("jobs.description")}
      kicker={t("jobs.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("jobs.title")}
      titleId="jobs-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("jobs.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}
      {!query.isLoading && result.source !== "live" ? (
        <RouteNotice>{t("jobs.vfsCacheRepair.actionsDisabled")}</RouteNotice>
      ) : null}
      {commandError ? <RouteNotice>{commandError}</RouteNotice> : null}
      {commandMessage ? <RouteNotice>{commandMessage}</RouteNotice> : null}

      <FilterBar label={t("jobs.filters")}>
        <FilterField label={t("jobs.filter.status")}>
          <select
            aria-label={t("jobs.filter.statusAria")}
            value={search.status ?? ""}
            onChange={(event) => onSearchChange({ status: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("jobs.filter.anyStatus")}</option>
            <option value="queued">{t("jobs.status.queued")}</option>
            <option value="running">{t("jobs.status.running")}</option>
            <option value="failed">{t("jobs.status.failed")}</option>
            <option value="succeeded">{t("jobs.status.succeeded")}</option>
            <option value="cancelled">{t("jobs.status.cancelled")}</option>
          </select>
        </FilterField>
        <FilterField label={t("jobs.filter.kind")}>
          <input
            aria-label={t("jobs.filter.kindAria")}
            placeholder="metadata_refresh"
            value={search.kind ?? ""}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("jobs.filter.resource")}>
          <input
            aria-label={t("jobs.filter.resourceAria")}
            placeholder="library"
            value={search.resource_class ?? ""}
            onChange={(event) =>
              onSearchChange({ resource_class: event.target.value || undefined, offset: 0 })
            }
          />
        </FilterField>
        <FilterField label={t("jobs.filter.library")}>
          <input
            aria-label={t("jobs.filter.libraryAria")}
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("jobs.filter.source")}>
          <input
            aria-label={t("jobs.filter.sourceAria")}
            placeholder="source-id"
            value={search.source_id ?? ""}
            onChange={(event) => onSearchChange({ source_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Button
            aria-pressed={sourceHashFilterActive}
            onClick={() =>
              onSearchChange({
                kind: SOURCE_FINGERPRINT_HASH_JOB_KIND,
                resource_class: SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS,
                offset: 0,
              })
            }
            variant={sourceHashFilterActive ? "default" : "outline"}
          >
            <Fingerprint size={16} />
            {t("jobs.filter.sourceHash")}
          </Button>
          <Button
            aria-pressed={vfsCacheRepairFilterActive}
            onClick={() =>
              onSearchChange({
                kind: VFS_CACHE_REPAIR_JOB_KIND,
                resource_class: VFS_CACHE_REPAIR_RESOURCE_CLASS,
                source_id: undefined,
                offset: 0,
              })
            }
            variant={vfsCacheRepairFilterActive ? "default" : "outline"}
          >
            <Wrench size={16} />
            {t("jobs.filter.vfsCacheRepair")}
          </Button>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("jobs.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                status: undefined,
                kind: undefined,
                resource_class: undefined,
                library_id: undefined,
                source_id: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("jobs.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <QueuePressureSummary
        pressure={result.value.queue_pressure}
        t={t}
      />

      <DataPanel
        description={t("jobs.queue.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("jobs.queue.urlFilters")}
          </div>
        }
        title={t("jobs.queue.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("jobs.loading")} /> : null}

        {!query.isLoading && result.value.jobs.length === 0 ? (
          <EmptyRouteState>{t("jobs.empty")}</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.jobs.length > 0 ? (
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
        ) : null}
      </DataPanel>
    </RoutePage>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;
type BadgeTone = "danger" | "info" | "neutral" | "success" | "warning";

function QueuePressureSummary({
  pressure,
  t,
}: {
  pressure: AdminJobQueuePressureSummary[];
  t: Translate;
}) {
  const queuedCount = pressure
    .filter((item) => item.status === "queued")
    .reduce((total, item) => total + item.count, 0);
  const claimableCount = pressure.reduce(
    (total, item) => total + item.claimable_count,
    0,
  );
  const delayedRetryCount = pressure.reduce(
    (total, item) => total + item.delayed_retry_count,
    0,
  );

  return (
    <DataPanel
      description={t("jobs.queuePressure.description", {
        groups: pressure.length,
        queued: queuedCount,
      })}
      headerAccessory={
        <div className="issueBadgeList">
          <Badge tone={claimableCount > 0 ? "warning" : "neutral"}>
            {t("jobs.queuePressure.claimable", { count: claimableCount })}
          </Badge>
          <Badge tone={delayedRetryCount > 0 ? "info" : "neutral"}>
            {t("jobs.queuePressure.delayed", { count: delayedRetryCount })}
          </Badge>
        </div>
      }
      title={t("jobs.queuePressure.title")}
    >
      {pressure.length === 0 ? (
        <EmptyRouteState>{t("jobs.queuePressure.empty")}</EmptyRouteState>
      ) : (
        <div className="jobQueuePressureGrid">
          {pressure.map((item) => (
            <div
              className="jobQueuePressureTile"
              key={`${item.kind}:${item.status}:${item.resource_class}`}
            >
              <div className="jobQueuePressureHeader">
                <strong>{item.kind}</strong>
                <Badge tone={jobStatusTone(item.status)}>{item.status}</Badge>
              </div>
              <span>{item.resource_class}</span>
              <div className="jobQueuePressureCounts">
                <Badge tone="info">
                  {t("jobs.queuePressure.count", { count: item.count })}
                </Badge>
                <Badge tone={item.claimable_count > 0 ? "warning" : "neutral"}>
                  {t("jobs.queuePressure.claimableShort", {
                    count: item.claimable_count,
                  })}
                </Badge>
                <Badge tone={item.delayed_retry_count > 0 ? "info" : "neutral"}>
                  {t("jobs.queuePressure.delayedShort", {
                    count: item.delayed_retry_count,
                  })}
                </Badge>
              </div>
              <small>
                {t("jobs.queuePressure.oldestQueued", {
                  time: item.oldest_queued_at ?? t("jobs.queuePressure.none"),
                })}
              </small>
              <small>
                {t("jobs.queuePressure.nextAttempt", {
                  time: item.next_attempt_at ?? t("jobs.queuePressure.none"),
                })}
              </small>
            </div>
          ))}
        </div>
      )}
    </DataPanel>
  );
}

function createColumns(
  t: Translate,
  actions: JobActionColumnOptions,
): Array<ColumnDef<AdminJobListItem>> {
  return [
    {
      accessorKey: "kind",
      header: t("jobs.column.kind"),
      cell: ({ row }) => (
        <div className="jobsPrimaryCell">
          <strong>{row.original.kind}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: t("jobs.column.status"),
      cell: ({ row }) => <JobStatusBadge status={row.original.status} hasError={row.original.has_error} />,
    },
    {
      id: "lifecycle",
      header: t("jobs.column.lifecycle"),
      cell: ({ row }) => <JobLifecycle job={row.original} t={t} />,
    },
    {
      accessorKey: "resource_class",
      header: t("jobs.column.resource"),
    },
    {
      accessorKey: "library_id",
      header: t("jobs.column.mediaLibrary"),
      cell: ({ row }) => row.original.library_id ?? t("jobs.none"),
    },
    {
      accessorKey: "source_id",
      header: t("jobs.column.mediaSource"),
      cell: ({ row }) => row.original.source_id ?? t("jobs.none"),
    },
    {
      accessorKey: "queued_at",
      header: t("jobs.column.queued"),
    },
    {
      id: "actions",
      header: t("jobs.column.actions"),
      cell: ({ row }) => <JobActions job={row.original} actions={actions} t={t} />,
    },
  ];
}

async function loadJobs(
  dataSource: AdminDataSource,
  search: JobsSearch,
  unavailableMessage: string,
): Promise<JobsResult> {
  if (!dataSource.loadJobs) {
    return {
      value: mockJobs,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadJobs(toAdminJobsQuery(search));
}

function toAdminJobsQuery(search: JobsSearch): AdminJobsQuery {
  return {
    status: search.status,
    kind: search.kind,
    resource_class: search.resource_class,
    library_id: search.library_id,
    source_id: search.source_id,
    limit: search.limit,
    offset: search.offset,
  };
}

type JobActionColumnOptions = {
  canExecute: boolean;
  canRetry: boolean;
  isPending: boolean;
  pendingAction: VfsCacheRepairJobAction | null;
  pendingJobId: string | null;
  runCommand(command: VfsCacheRepairJobCommand): void;
};

function JobActions({
  actions,
  job,
  t,
}: {
  actions: JobActionColumnOptions;
  job: AdminJobListItem;
  t: Translate;
}) {
  if (!isVfsCacheRepairJob(job)) {
    return <span>{t("jobs.vfsCacheRepair.notApplicable")}</span>;
  }

  if (job.status === "queued") {
    const pending =
      actions.isPending &&
      actions.pendingAction === "execute" &&
      actions.pendingJobId === job.id;

    return (
      <Button
        aria-label={t("jobs.vfsCacheRepair.executeAria", { jobId: job.id })}
        disabled={!actions.canExecute || actions.isPending}
        onClick={() => actions.runCommand({ action: "execute", job })}
        size="sm"
      >
        <Play size={14} />
        {pending ? t("jobs.vfsCacheRepair.executing") : t("jobs.vfsCacheRepair.execute")}
      </Button>
    );
  }

  if (job.status === "failed") {
    const pending =
      actions.isPending &&
      actions.pendingAction === "retry" &&
      actions.pendingJobId === job.id;

    return (
      <Button
        aria-label={t("jobs.vfsCacheRepair.retryAria", { jobId: job.id })}
        disabled={!actions.canRetry || actions.isPending}
        onClick={() => actions.runCommand({ action: "retry", job })}
        size="sm"
        variant="outline"
      >
        <RotateCcw size={14} />
        {pending ? t("jobs.vfsCacheRepair.retrying") : t("jobs.vfsCacheRepair.retry")}
      </Button>
    );
  }

  return <span>{t("jobs.vfsCacheRepair.noStateAction")}</span>;
}

function JobLifecycle({ job, t }: { job: AdminJobListItem; t: Translate }) {
  return (
    <div className="jobsLifecycleCell">
      <div className="issueBadgeList">
        <Badge tone={jobPriorityTone(job.priority)}>
          {t("jobs.lifecycle.priority", { priority: job.priority })}
        </Badge>
        <Badge tone={job.attempt > 1 ? "warning" : "neutral"}>
          {t("jobs.lifecycle.attempts", {
            attempt: job.attempt,
            max: job.max_attempts,
          })}
        </Badge>
      </div>
      {job.retry_of_job_id ? (
        <span>{t("jobs.lifecycle.retryOf", { jobId: job.retry_of_job_id })}</span>
      ) : null}
      {job.next_attempt_at ? (
        <span>{t("jobs.lifecycle.nextAttemptAt", { time: job.next_attempt_at })}</span>
      ) : null}
    </div>
  );
}

function jobPriorityTone(priority: string): BadgeTone {
  if (priority === "high") {
    return "warning";
  }

  if (priority === "low") {
    return "neutral";
  }

  return "info";
}

function jobStatusTone(status: string): BadgeTone {
  if (status === "failed") {
    return "danger";
  }

  if (status === "running") {
    return "info";
  }

  if (status === "queued") {
    return "warning";
  }

  return "success";
}

function JobStatusBadge({ status, hasError }: { status: string; hasError: boolean }) {
  if (hasError || status === "failed") {
    return <Badge tone="danger">{status}</Badge>;
  }

  if (status === "running") {
    return <Badge tone="info">{status}</Badge>;
  }

  if (status === "queued") {
    return <Badge tone="warning">{status}</Badge>;
  }

  return <Badge tone="success">{status}</Badge>;
}

function isVfsCacheRepairJob(job: AdminJobListItem) {
  return (
    job.kind === VFS_CACHE_REPAIR_JOB_KIND &&
    job.resource_class === VFS_CACHE_REPAIR_RESOURCE_CLASS
  );
}

function errorMessage(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback;
}
