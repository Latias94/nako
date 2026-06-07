import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { RefreshCw, Search, Wrench, X } from "lucide-react";
import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminStorageStagingDiagnosticsResponse,
  AdminStorageStagingQuery,
  AdminVfsCacheRepairActionPlanResponse,
  AdminVfsCacheRepairRemediationPlanResponse,
  AdminVfsCacheRepairTargetListResponse,
} from "../../adminApi/types";
import {
  mockStorageStaging,
  mockVfsCacheRepairActionPlan,
  mockVfsCacheRepairRemediationPlan,
  mockVfsCacheRepairTargets,
} from "../../adminApi/mockData";
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

export type StorageStagingSearch = {
  purpose?: string;
  state?: string;
  limit: number;
  offset: number;
};

export type StorageStagingPageProps = {
  dataSource: AdminDataSource;
  search: StorageStagingSearch;
  onSearchChange(next: Partial<StorageStagingSearch>): void;
};

type StorageStagingResult = {
  value: AdminStorageStagingDiagnosticsResponse;
  source: DataSourceMode;
  error?: string;
};

type StorageRecord = AdminStorageStagingDiagnosticsResponse["records"][number];
type VfsCacheRepairActionPlanResult = {
  value: AdminVfsCacheRepairActionPlanResponse;
  source: DataSourceMode;
  error?: string;
};
type VfsCacheRepairRemediationPlanResult = {
  value: AdminVfsCacheRepairRemediationPlanResponse;
  source: DataSourceMode;
  error?: string;
};
type VfsCacheRepairTargetsResult = {
  value: AdminVfsCacheRepairTargetListResponse;
  source: DataSourceMode;
  error?: string;
};
type VfsCacheRepairTarget = AdminVfsCacheRepairTargetListResponse["targets"][number];
type VfsCacheRepairActionGroup =
  AdminVfsCacheRepairRemediationPlanResponse["action_groups"][number];
type BadgeTone = "danger" | "info" | "neutral" | "success" | "warning";

export function StorageStagingPage({
  dataSource,
  search,
  onSearchChange,
}: StorageStagingPageProps) {
  const { locale, t } = useI18n();
  const queryClient = useQueryClient();
  const [mutationMessage, setMutationMessage] = useState<string | null>(null);
  const [mutationError, setMutationError] = useState<string | null>(null);
  const query = useQuery({
    queryKey: ["admin-storage-staging", search, locale],
    queryFn: () => loadStorageStaging(dataSource, search, t("storage.dataSourceUnavailable")),
  });
  const actionPlanQuery = useQuery({
    queryKey: ["admin-storage-staging", "vfs-cache-repair", "action-plan", locale],
    queryFn: () =>
      loadVfsCacheRepairActionPlan(
        dataSource,
        t("storage.repair.actionPlanUnavailable"),
      ),
  });
  const remediationPlanQuery = useQuery({
    queryKey: ["admin-storage-staging", "vfs-cache-repair", "remediation-plan", locale],
    queryFn: () =>
      loadVfsCacheRepairRemediationPlan(
        dataSource,
        t("storage.repair.remediationPlanUnavailable"),
      ),
  });
  const repairTargetsQuery = useQuery({
    queryKey: ["admin-storage-staging", "vfs-cache-repair", "targets", locale],
    queryFn: () =>
      loadVfsCacheRepairTargets(
        dataSource,
        t("storage.repair.targetsUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: mockStorageStaging,
    source: "mock" as const,
  };
  const actionPlanResult = actionPlanQuery.data ?? {
    value: mockVfsCacheRepairActionPlan,
    source: "mock" as const,
  };
  const remediationPlanResult = remediationPlanQuery.data ?? {
    value: mockVfsCacheRepairRemediationPlan,
    source: "mock" as const,
  };
  const repairTargetsResult = repairTargetsQuery.data ?? {
    value: mockVfsCacheRepairTargets,
    source: "mock" as const,
  };
  const firstEnqueueableRepairTarget =
    repairTargetsResult.value.targets.find(isEnqueueableRepairTarget) ?? null;
  const repairReadQueries = [actionPlanQuery, remediationPlanQuery, repairTargetsQuery];
  const repairReadLoading = repairReadQueries.some((repairQuery) => repairQuery.isLoading);
  const routeReadFetching =
    query.isFetching || repairReadQueries.some((repairQuery) => repairQuery.isFetching);
  const mutationReadLoading = query.isLoading || repairReadLoading;
  const repairSource = combineSources([
    actionPlanResult.source,
    remediationPlanResult.source,
    repairTargetsResult.source,
  ]);
  const canRefreshLatest =
    result.source === "live" &&
    actionPlanResult.source === "live" &&
    Boolean(dataSource.refreshLatestVfsCacheRepair);
  const canEnqueueFirstTarget =
    result.source === "live" &&
    repairTargetsResult.source === "live" &&
    Boolean(dataSource.enqueueVfsCacheRepairTarget) &&
    firstEnqueueableRepairTarget !== null;
  const repairMutationDisabledReason = mutationReadLoading
    ? null
    : mutationDisabledReason(
        {
          actionPlanLive: actionPlanResult.source === "live",
          canEnqueueFirstTarget,
          canRefreshLatest,
          hasEnqueueRoute: Boolean(dataSource.enqueueVfsCacheRepairTarget),
          hasEnqueueableTarget: firstEnqueueableRepairTarget !== null,
          hasRefreshRoute: Boolean(dataSource.refreshLatestVfsCacheRepair),
          stagingLive: result.source === "live",
          targetsLive: repairTargetsResult.source === "live",
        },
        t,
      );
  const activeFilterCount = useMemo(
    () => [search.purpose, search.state].filter(Boolean).length,
    [search.purpose, search.state],
  );
  const table = useReactTable({
    data: result.value.records,
    columns: createColumns(t),
    getCoreRowModel: getCoreRowModel(),
  });
  const refreshLatestMutation = useMutation({
    mutationFn: async () => {
      if (result.source !== "live" || actionPlanResult.source !== "live") {
        throw new Error(t("storage.repair.notLiveError"));
      }
      if (!dataSource.refreshLatestVfsCacheRepair) {
        throw new Error(t("storage.repair.refreshUnavailable"));
      }

      return dataSource.refreshLatestVfsCacheRepair();
    },
    onMutate: () => {
      setMutationMessage(null);
      setMutationError(null);
    },
    onSuccess: (response) => {
      setMutationMessage(
        t("storage.repair.refreshSucceeded", {
          refreshed: response.refreshed ? t("storage.repair.yes") : t("storage.repair.no"),
        }),
      );
      void queryClient.invalidateQueries({ queryKey: ["admin-storage-staging"] });
    },
    onError: (error) => {
      setMutationError(errorMessage(error, t("storage.repair.operationFailed")));
    },
  });
  const enqueueMutation = useMutation({
    mutationFn: async () => {
      if (result.source !== "live" || repairTargetsResult.source !== "live") {
        throw new Error(t("storage.repair.notLiveError"));
      }
      if (!dataSource.enqueueVfsCacheRepairTarget) {
        throw new Error(t("storage.repair.enqueueUnavailable"));
      }
      if (!firstEnqueueableRepairTarget) {
        throw new Error(t("storage.repair.noEnqueueableTarget"));
      }

      return dataSource.enqueueVfsCacheRepairTarget(firstEnqueueableRepairTarget.target_ref, {
        priority: "normal",
      });
    },
    onMutate: () => {
      setMutationMessage(null);
      setMutationError(null);
    },
    onSuccess: (response) => {
      setMutationMessage(
        t("storage.repair.enqueueSucceeded", {
          jobId: response.job.id,
          status: response.job.status,
        }),
      );
      void queryClient.invalidateQueries({ queryKey: ["admin-storage-staging"] });
    },
    onError: (error) => {
      setMutationError(errorMessage(error, t("storage.repair.operationFailed")));
    },
  });

  return (
    <RoutePage
      actions={
        <Button
          disabled={routeReadFetching}
          onClick={() => {
            void query.refetch();
            void actionPlanQuery.refetch();
            void remediationPlanQuery.refetch();
            void repairTargetsQuery.refetch();
          }}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("storage.refresh")}
        </Button>
      }
      description={t("storage.description")}
      kicker={t("storage.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("storage.title")}
      titleId="storage-staging-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("storage.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("storage.filters")}>
        <FilterField label={t("storage.filter.purpose")}>
          <input
            aria-label={t("storage.filter.purposeAria")}
            placeholder="ffmpeg_input"
            value={search.purpose ?? ""}
            onChange={(event) => onSearchChange({ purpose: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("storage.filter.state")}>
          <select
            aria-label={t("storage.filter.stateAria")}
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("storage.filter.anyState")}</option>
            <option value="ready">{t("storage.state.ready")}</option>
            <option value="failed">{t("storage.state.failed")}</option>
            <option value="stale">{t("storage.state.stale")}</option>
            <option value="expired">{t("storage.state.expired")}</option>
          </select>
        </FilterField>
        <FilterField label={t("storage.filter.limit")}>
          <input
            aria-label={t("storage.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("storage.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                purpose: undefined,
                state: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("storage.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("storage.repair.description", {
          total: remediationPlanResult.value.total_unresolved_targets,
          returned: repairTargetsResult.value.page.returned,
        })}
        headerAccessory={
          <div className="routeActionGroup" role="group" aria-label={t("storage.repair.actions")}>
            <SourceLabel source={repairSource} />
            <Button
              disabled={!canRefreshLatest || refreshLatestMutation.isPending}
              onClick={() => refreshLatestMutation.mutate()}
              size="sm"
              variant="outline"
            >
              <RefreshCw size={14} />
              {refreshLatestMutation.isPending
                ? t("storage.repair.refreshing")
                : t("storage.repair.refreshLatest")}
            </Button>
            <Button
              disabled={!canEnqueueFirstTarget || enqueueMutation.isPending}
              onClick={() => enqueueMutation.mutate()}
              size="sm"
            >
              <Wrench size={14} />
              {enqueueMutation.isPending
                ? t("storage.repair.enqueueing")
                : t("storage.repair.enqueueFirstTarget")}
            </Button>
          </div>
        }
        title={t("storage.repair.title")}
      >
        {repairReadLoading ? <RowsSkeleton label={t("storage.repair.loading")} /> : null}

        {actionPlanResult.error ? (
          <RouteNotice>
            {t("storage.repair.actionPlanFallback", { error: actionPlanResult.error })}
          </RouteNotice>
        ) : null}
        {remediationPlanResult.error ? (
          <RouteNotice>
            {t("storage.repair.remediationPlanFallback", {
              error: remediationPlanResult.error,
            })}
          </RouteNotice>
        ) : null}
        {repairTargetsResult.error ? (
          <RouteNotice>
            {t("storage.repair.targetsFallback", { error: repairTargetsResult.error })}
          </RouteNotice>
        ) : null}
        {repairMutationDisabledReason ? (
          <RouteNotice>{repairMutationDisabledReason}</RouteNotice>
        ) : null}
        {mutationError ? <RouteNotice>{mutationError}</RouteNotice> : null}
        {mutationMessage ? <RouteNotice>{mutationMessage}</RouteNotice> : null}

        {!repairReadLoading ? (
          <>
            <div className="settingsPanelGrid">
              <section
                aria-label={t("storage.repair.actionPlan")}
                className="settingsRowList"
              >
                <div className="settingsRowList">
                  <RepairRow
                    badge={actionPlanResult.value.plan.status}
                    label={t("storage.repair.planStatus")}
                    tone={repairStatusTone(actionPlanResult.value.plan.status)}
                    value={actionPlanResult.value.plan.action}
                  />
                  <RepairRow
                    badge={actionPlanResult.value.plan.action}
                    label={t("storage.repair.action")}
                    tone={actionTone(actionPlanResult.value.plan.action)}
                    value={actionPlanResult.value.plan.action}
                  />
                  <RepairRow
                    badge={actionPlanResult.value.plan.readiness.status}
                    detail={listLabel(actionPlanResult.value.plan.readiness.reasons, t)}
                    label={t("storage.repair.readiness")}
                    tone={repairStatusTone(actionPlanResult.value.plan.readiness.status)}
                    value={
                      actionPlanResult.value.plan.readiness.api_executable
                        ? t("storage.repair.apiExecutable")
                        : t("storage.repair.planOnly")
                    }
                  />
                </div>
                <BoundaryList
                  rows={[
                    {
                      enabled: actionPlanResult.value.plan.boundary.refreshes_vfs_cache,
                      label: t("storage.repair.boundary.refreshesCache"),
                    },
                    {
                      enabled: actionPlanResult.value.plan.boundary.changes_backend_configuration,
                      label: t("storage.repair.boundary.backendConfig"),
                    },
                    {
                      enabled:
                        actionPlanResult.value.plan.boundary
                          .requires_manual_failure_inspection,
                      label: t("storage.repair.boundary.manualInspection"),
                    },
                    {
                      enabled: actionPlanResult.value.plan.boundary.deletes_cache_entries,
                      label: t("storage.repair.boundary.deletesCache"),
                    },
                    {
                      enabled: actionPlanResult.value.plan.boundary.writes_library_files,
                      label: t("storage.repair.boundary.writesFiles"),
                    },
                    {
                      enabled: actionPlanResult.value.plan.boundary.starts_durable_job,
                      label: t("storage.repair.boundary.startsJob"),
                    },
                  ]}
                  t={t}
                />
              </section>

              <section
                aria-label={t("storage.repair.remediationPlan")}
                className="settingsRowList"
              >
                <div className="settingsRowList">
                  <RepairRow
                    badge={t("storage.repair.targetsReturned", {
                      count: repairTargetsResult.value.page.returned,
                    })}
                    label={t("storage.repair.unresolvedTargets")}
                    tone={remediationPlanResult.value.total_unresolved_targets > 0 ? "warning" : "success"}
                    value={String(remediationPlanResult.value.total_unresolved_targets)}
                  />
                  {remediationPlanResult.value.classification_counts.map((classification) => (
                    <RepairRow
                      key={classification.classification}
                      badge={t("storage.repair.targetsReturned", { count: classification.count })}
                      label={t("storage.repair.classification")}
                      tone={classificationTone(classification.classification)}
                      value={classification.classification}
                    />
                  ))}
                  {remediationPlanResult.value.classification_counts.length === 0 ? (
                    <EmptyRouteState>{t("storage.repair.noClassificationCounts")}</EmptyRouteState>
                  ) : null}
                </div>
                <BoundaryList
                  rows={[
                    {
                      enabled: remediationPlanResult.value.boundary.read_only,
                      label: t("storage.repair.boundary.readOnly"),
                    },
                    {
                      enabled: remediationPlanResult.value.boundary.refreshes_vfs_cache,
                      label: t("storage.repair.boundary.refreshesCache"),
                    },
                    {
                      enabled:
                        remediationPlanResult.value.boundary.changes_backend_configuration,
                      label: t("storage.repair.boundary.backendConfig"),
                    },
                    {
                      enabled: remediationPlanResult.value.boundary.deletes_cache_entries,
                      label: t("storage.repair.boundary.deletesCache"),
                    },
                    {
                      enabled: remediationPlanResult.value.boundary.writes_library_files,
                      label: t("storage.repair.boundary.writesFiles"),
                    },
                    {
                      enabled: remediationPlanResult.value.boundary.starts_durable_job,
                      label: t("storage.repair.boundary.startsJob"),
                    },
                  ]}
                  t={t}
                />
              </section>
            </div>

            <RepairActionGroups groups={remediationPlanResult.value.action_groups} t={t} />
            <RepairTargetsTable targets={repairTargetsResult.value.targets} t={t} />
          </>
        ) : null}
      </DataPanel>

      <DataPanel
        description={t("storage.records.description", {
          returned: result.value.page.returned,
          used: formatBytes(result.value.summary.used_manifest_bytes, t),
          max: formatBytes(result.value.summary.configured_max_bytes, t),
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("storage.records.urlFilters")}
          </div>
        }
        title={t("storage.records.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("storage.loading")} /> : null}

        {!query.isLoading && result.value.records.length === 0 ? (
          <EmptyRouteState>{t("storage.empty")}</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.records.length > 0 ? (
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

function createColumns(t: Translate): Array<ColumnDef<StorageRecord>> {
  return [
    {
      accessorKey: "id",
      header: t("storage.column.record"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.id}</strong>
          <span>{row.original.purpose}</span>
        </div>
      ),
    },
    {
      accessorKey: "state",
      header: t("storage.column.state"),
      cell: ({ row }) => <StorageStateBadge record={row.original} />,
    },
    {
      accessorKey: "source_scheme",
      header: t("storage.column.sourceScheme"),
    },
    {
      accessorKey: "size_bytes",
      header: t("storage.column.size"),
      cell: ({ row }) => formatBytes(row.original.size_bytes, t),
    },
    {
      accessorKey: "active_leases",
      header: t("storage.column.leases"),
    },
    {
      id: "validation",
      header: t("storage.column.validation"),
      cell: ({ row }) =>
        row.original.has_validation_error
          ? t("storage.validation.failed")
          : t("storage.validation.clean"),
    },
    {
      accessorKey: "expires_at_ms",
      header: t("storage.column.expires"),
      cell: ({ row }) => timestampLabel(row.original.expires_at_ms, t),
    },
  ];
}

async function loadStorageStaging(
  dataSource: AdminDataSource,
  search: StorageStagingSearch,
  unavailableMessage: string,
): Promise<StorageStagingResult> {
  if (!dataSource.loadStorageStaging) {
    return {
      value: mockStorageStaging,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadStorageStaging(toAdminStorageStagingQuery(search));
}

function toAdminStorageStagingQuery(search: StorageStagingSearch): AdminStorageStagingQuery {
  return {
    purpose: search.purpose,
    state: search.state,
    limit: search.limit,
    offset: search.offset,
  };
}

function StorageStateBadge({ record }: { record: StorageRecord }) {
  if (record.has_validation_error || record.state === "failed") {
    return <Badge tone="danger">{record.state}</Badge>;
  }

  if (record.state === "ready") {
    return <Badge tone="success">{record.state}</Badge>;
  }

  return <Badge tone="warning">{record.state}</Badge>;
}

function RepairRow({
  badge,
  detail,
  label,
  tone,
  value,
}: {
  badge: string;
  detail?: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="settingsDiagnosticRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail ? <small>{detail}</small> : null}
      </div>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function BoundaryList({
  rows,
  t,
}: {
  rows: Array<{ enabled: boolean; label: string }>;
  t: Translate;
}) {
  return (
    <div className="settingsRowList">
      {rows.map((row) => (
        <RepairRow
          key={row.label}
          badge={row.enabled ? t("storage.repair.yes") : t("storage.repair.no")}
          label={row.label}
          tone={row.enabled ? "warning" : "neutral"}
          value={
            row.enabled
              ? t("storage.repair.boundary.included")
              : t("storage.repair.boundary.excluded")
          }
        />
      ))}
    </div>
  );
}

function RepairActionGroups({
  groups,
  t,
}: {
  groups: VfsCacheRepairActionGroup[];
  t: Translate;
}) {
  if (groups.length === 0) {
    return <EmptyRouteState>{t("storage.repair.noActionGroups")}</EmptyRouteState>;
  }

  return (
    <section aria-label={t("storage.repair.actionGroups")}>
      <div className="tableScroll">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t("storage.repair.column.actionGroup")}</TableHead>
              <TableHead>{t("storage.repair.column.status")}</TableHead>
              <TableHead>{t("storage.repair.column.readiness")}</TableHead>
              <TableHead>{t("storage.repair.column.sampleTargets")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {groups.map((group) => (
              <TableRow key={`${group.action}-${group.status}`}>
                <TableCell>
                  <div className="routePrimaryCell">
                    <strong>{group.action}</strong>
                    <span>{t("storage.repair.targetsReturned", { count: group.count })}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge tone={repairStatusTone(group.status)}>{group.status}</Badge>
                </TableCell>
                <TableCell>
                  <div className="routePrimaryCell">
                    <strong>{group.readiness.status}</strong>
                    <span>{listLabel(group.readiness.reasons, t)}</span>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="issueBadgeList">
                    {group.sample_targets.map((target) => (
                      <Badge key={target.target_ref} tone={classificationTone(target.classification)}>
                        {target.target_ref}
                      </Badge>
                    ))}
                    {group.sample_targets.length === 0 ? t("storage.repair.none") : null}
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </section>
  );
}

function RepairTargetsTable({ targets, t }: { targets: VfsCacheRepairTarget[]; t: Translate }) {
  return (
    <section aria-label={t("storage.repair.targetsTitle")}>
      {targets.length === 0 ? (
        <EmptyRouteState>{t("storage.repair.noTargets")}</EmptyRouteState>
      ) : (
        <div className="tableScroll">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("storage.repair.column.target")}</TableHead>
                <TableHead>{t("storage.repair.column.operation")}</TableHead>
                <TableHead>{t("storage.repair.column.classification")}</TableHead>
                <TableHead>{t("storage.repair.column.recommendedAction")}</TableHead>
                <TableHead>{t("storage.repair.column.retryable")}</TableHead>
                <TableHead>{t("storage.repair.column.safeMessage")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {targets.map((target) => (
                <TableRow key={target.target_ref}>
                  <TableCell>
                    <div className="routePrimaryCell">
                      <strong>{target.target_ref}</strong>
                      <span>{target.scheme}</span>
                    </div>
                  </TableCell>
                  <TableCell>{target.operation}</TableCell>
                  <TableCell>
                    <Badge tone={classificationTone(target.classification)}>
                      {target.classification}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <Badge tone={actionTone(target.recommended_action)}>
                      {target.recommended_action}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <Badge tone={target.retryable ? "success" : "neutral"}>
                      {target.retryable ? t("storage.repair.yes") : t("storage.repair.no")}
                    </Badge>
                  </TableCell>
                  <TableCell>{target.safe_message ?? t("storage.repair.none")}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </section>
  );
}

async function loadVfsCacheRepairActionPlan(
  dataSource: AdminDataSource,
  unavailableMessage: string,
): Promise<VfsCacheRepairActionPlanResult> {
  if (!dataSource.loadVfsCacheRepairActionPlan) {
    return {
      value: mockVfsCacheRepairActionPlan,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadVfsCacheRepairActionPlan();
}

async function loadVfsCacheRepairRemediationPlan(
  dataSource: AdminDataSource,
  unavailableMessage: string,
): Promise<VfsCacheRepairRemediationPlanResult> {
  if (!dataSource.loadVfsCacheRepairRemediationPlan) {
    return {
      value: mockVfsCacheRepairRemediationPlan,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadVfsCacheRepairRemediationPlan();
}

async function loadVfsCacheRepairTargets(
  dataSource: AdminDataSource,
  unavailableMessage: string,
): Promise<VfsCacheRepairTargetsResult> {
  if (!dataSource.loadVfsCacheRepairTargets) {
    return {
      value: mockVfsCacheRepairTargets,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadVfsCacheRepairTargets({ limit: 20, offset: 0 });
}

function isEnqueueableRepairTarget(target: VfsCacheRepairTarget) {
  return target.recommended_action === "refresh_cache";
}

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}

function timestampLabel(value: number | null, t: Translate) {
  if (value === null) {
    return t("storage.none");
  }

  return new Date(value).toISOString();
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("storage.unknown");
  }

  if (value < 1024) {
    return `${value} B`;
  }

  const units = ["KiB", "MiB", "GiB", "TiB"];
  let amount = value / 1024;
  let unitIndex = 0;

  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024;
    unitIndex += 1;
  }

  return `${amount.toFixed(amount >= 10 ? 1 : 2)} ${units[unitIndex]}`;
}

function listLabel(values: string[], t: Translate) {
  return values.length > 0 ? values.join(", ") : t("storage.repair.none");
}

function mutationDisabledReason(
  readiness: {
    actionPlanLive: boolean;
    canEnqueueFirstTarget: boolean;
    canRefreshLatest: boolean;
    hasEnqueueRoute: boolean;
    hasEnqueueableTarget: boolean;
    hasRefreshRoute: boolean;
    stagingLive: boolean;
    targetsLive: boolean;
  },
  t: Translate,
) {
  if (readiness.canRefreshLatest && readiness.canEnqueueFirstTarget) {
    return null;
  }

  const reasons = new Set<string>();

  if (!readiness.canRefreshLatest) {
    if (!readiness.stagingLive || !readiness.actionPlanLive) {
      reasons.add(t("storage.repair.disabled.refreshNotLive"));
    } else if (!readiness.hasRefreshRoute) {
      reasons.add(t("storage.repair.disabled.refreshRouteMissing"));
    }
  }

  if (!readiness.canEnqueueFirstTarget) {
    if (!readiness.stagingLive || !readiness.targetsLive) {
      reasons.add(t("storage.repair.disabled.enqueueNotLive"));
    } else if (!readiness.hasEnqueueRoute) {
      reasons.add(t("storage.repair.disabled.enqueueRouteMissing"));
    } else if (!readiness.hasEnqueueableTarget) {
      reasons.add(t("storage.repair.disabled.noEnqueueableTarget"));
    }
  }

  return t("storage.repair.mutationDisabled", {
    reasons: Array.from(reasons).join(" "),
  });
}

function combineSources(sources: DataSourceMode[]): DataSourceMode {
  if (sources.length === 0) {
    return "mock";
  }

  if (sources.every((source) => source === "live")) {
    return "live";
  }

  if (sources.some((source) => source === "live")) {
    return "hybrid";
  }

  return "mock";
}

function errorMessage(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback;
}

function repairStatusTone(status: string): BadgeTone {
  if (status === "executable" || status === "no_action") {
    return "success";
  }

  if (status === "plan_only") {
    return "warning";
  }

  return "neutral";
}

function actionTone(action: string): BadgeTone {
  if (action === "none") {
    return "success";
  }

  if (action === "refresh_cache") {
    return "info";
  }

  if (action === "fix_backend_configuration" || action === "inspect_failure") {
    return "warning";
  }

  return "neutral";
}

function classificationTone(classification: string): BadgeTone {
  if (classification === "healthy") {
    return "success";
  }

  if (classification === "repairable_stale_fallback") {
    return "warning";
  }

  if (classification === "retryable_refresh_failure") {
    return "info";
  }

  if (classification === "operator_action_required" || classification === "unknown_failure") {
    return "danger";
  }

  return "neutral";
}
