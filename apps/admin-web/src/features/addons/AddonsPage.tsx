import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { RefreshCw, Search, ShieldCheck, X } from "lucide-react";
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AddonStatus,
  AddonsRouteRow,
  AddonsRouteSummary,
} from "../../adminApi/types";
import { mockAddonsRouteSummary } from "../../adminApi/mockData";
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

export type AddonsSearch = {
  status?: AddonStatus;
};

export type AddonsPageProps = {
  dataSource: AdminDataSource;
  search: AddonsSearch;
  onSearchChange(next: Partial<AddonsSearch>): void;
};

type AddonsResult = {
  value: AddonsRouteSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function AddonsPage({
  dataSource,
  search,
  onSearchChange,
}: AddonsPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-addons", search, locale],
    queryFn: () => loadAddons(dataSource, search, t("addons.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockAddonsRouteSummary,
    source: "mock" as const,
  };
  const summary = result.value;
  const activeFilterCount = useMemo(() => (search.status ? 1 : 0), [search.status]);
  const table = useReactTable({
    data: summary.addons,
    columns: createColumns(t),
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
          {t("addons.refresh")}
        </Button>
      }
      description={t("addons.description")}
      kicker={t("addons.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("addons.title")}
      titleId="addons-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("addons.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("addons.filters")}>
        <FilterField label={t("addons.filter.status")}>
          <select
            aria-label={t("addons.filter.statusAria")}
            value={search.status ?? ""}
            onChange={(event) =>
              onSearchChange({
                status: addonStatusInput(event.target.value),
              })
            }
          >
            <option value="">{t("addons.filter.anyStatus")}</option>
            <option value="enabled">{t("addons.status.enabled")}</option>
            <option value="disabled">{t("addons.status.disabled")}</option>
            <option value="unregistered">{t("addons.status.unregistered")}</option>
          </select>
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("addons.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() => onSearchChange({ status: undefined })}
            variant="ghost"
          >
            <X size={16} />
            {t("addons.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      {query.isLoading ? <RowsSkeleton label={t("addons.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="addonsSummaryGrid">
            <SummaryCard
              badge={t("addons.summary.registered.badge", { count: summary.statusCounts.enabled })}
              label={t("addons.summary.registered.label")}
              tone={summary.statusCounts.enabled > 0 ? "success" : "neutral"}
              value={summary.addons.length.toString()}
            />
            <SummaryCard
              badge={summary.health?.status ?? t("addons.summary.health.notChecked")}
              label={t("addons.summary.health.label")}
              tone={healthTone(summary.health?.status)}
              value={summary.selectedAddon?.name ?? t("addons.summary.health.none")}
            />
            <SummaryCard
              badge={t("addons.summary.tokens.badge", { count: summary.tokens.length })}
              label={t("addons.summary.tokens.label")}
              tone={summary.tokens.some((token) => token.status === "active") ? "success" : "warning"}
              value={t("addons.summary.tokens.active", {
                count: summary.tokens.filter((token) => token.status === "active").length,
              })}
            />
            <SummaryCard
              badge={t("addons.summary.permissions.badge", { count: summary.grants.length })}
              label={t("addons.summary.permissions.label")}
              tone={summary.grants.length > 0 ? "info" : "warning"}
              value={t("addons.summary.permissions.value", {
                count: summary.selectedAddon?.grantedScopeCount ?? 0,
              })}
            />
          </div>

          <DataPanel
            description={t("addons.registry.description", { count: summary.addons.length })}
            headerAccessory={
              <div className="searchHint">
                <Search size={15} />
                {t("addons.registry.urlFilters")}
              </div>
            }
            title={t("addons.registry.title")}
          >
            {summary.addons.length === 0 ? (
              <EmptyRouteState>{t("addons.registry.empty")}</EmptyRouteState>
            ) : null}

            {summary.addons.length > 0 ? (
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

          <div className="addonsRouteGrid">
            <DataPanel
              description={t("addons.selected.description")}
              title={t("addons.selected.title")}
            >
              {summary.selectedAddon ? (
                <div className="addonsFactList">
                  <FactRow
                    badge={summary.selectedAddon.status}
                    label={t("addons.selected.registration")}
                    tone={addonStatusTone(summary.selectedAddon.status)}
                    value={summary.selectedAddon.manifestId}
                  />
                  <FactRow
                    detail={summary.selectedAddon.resourceKinds.join(", ") || t("addons.selected.noneDeclared")}
                    label={t("addons.selected.resources")}
                    value={t("addons.selected.declaredCount", { count: summary.selectedAddon.resourceCount })}
                  />
                  <FactRow
                    detail={summary.selectedAddon.grantedScopes.join(", ") || t("addons.selected.noneGranted")}
                    label={t("addons.selected.grantedScopes")}
                    value={t("addons.selected.scopeCount", { count: summary.selectedAddon.grantedScopeCount })}
                  />
                  <FactRow
                    detail={runtimePolicy(summary.selectedAddon, t)}
                    label={t("addons.selected.runtimeAuth")}
                    value={summary.selectedAddon.authMode}
                  />
                </div>
              ) : (
                <EmptyRouteState>{t("addons.selected.empty")}</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description={t("addons.health.description")}
              title={t("addons.health.title")}
            >
              {summary.health ? (
                <div className="addonsFactList">
                  <FactRow
                    badge={summary.health.status}
                    label={t("addons.health.status")}
                    tone={healthTone(summary.health.status)}
                    value={`${summary.health.latencyMs} ms`}
                  />
                  <FactRow
                    label={t("addons.health.protocolVersion")}
                    value={summary.health.protocolVersion ?? t("addons.health.notReported")}
                  />
                  <FactRow
                    label={t("addons.health.addonVersion")}
                    value={summary.health.addonVersion ?? t("addons.health.notReported")}
                  />
                  <FactRow
                    label={t("addons.health.safeErrorCode")}
                    value={summary.health.safeErrorCode ?? t("addons.none")}
                  />
                </div>
              ) : (
                <EmptyRouteState>{t("addons.health.empty")}</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description={t("addons.surface.description")}
              title={t("addons.surface.title")}
            >
              {summary.surfaceSummary ? (
                <div className="addonsCountGrid">
                  <CountTile label={t("addons.surface.entryPoints")} value={summary.surfaceSummary.entryPointCount} />
                  <CountTile label={t("addons.surface.hostedPages")} value={summary.surfaceSummary.hostedPageCount} />
                  <CountTile label={t("addons.surface.tasks")} value={summary.surfaceSummary.taskCount} />
                  <CountTile
                    label={t("addons.surface.eventSubscriptions")}
                    value={summary.surfaceSummary.eventSubscriptionCount}
                  />
                  <CountTile
                    label={t("addons.surface.secretReferenceFields")}
                    value={summary.surfaceSummary.secretReferenceFieldCount}
                  />
                  <CountTile
                    label={t("addons.surface.configurationSchema")}
                    value={
                      summary.surfaceSummary.configurationSchemaDeclared
                        ? t("addons.surface.declared")
                        : t("addons.surface.notDeclared")
                    }
                  />
                </div>
              ) : (
                <EmptyRouteState>{t("addons.surface.empty")}</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description={t("addons.credentials.description")}
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  {t("addons.credentials.redacted")}
                </div>
              }
              title={t("addons.credentials.title")}
            >
              <div className="addonsCredentialGrid">
                <section aria-label={t("addons.credentials.tokenSectionAria")}>
                  <h3>{t("addons.credentials.tokenPrefixes")}</h3>
                  <div className="addonsFactList">
                    {summary.tokens.map((token) => (
                      <FactRow
                        badge={token.status}
                        detail={
                          token.lastUsedAt
                            ? t("addons.credentials.lastUsed", { time: timestampLabel(token.lastUsedAt) })
                            : t("addons.credentials.neverUsed")
                        }
                        key={token.id}
                        label={token.label}
                        tone={token.status === "active" ? "success" : "neutral"}
                        value={token.tokenPrefix}
                      />
                    ))}
                    {summary.tokens.length === 0 ? (
                      <EmptyRouteState>{t("addons.credentials.tokensEmpty")}</EmptyRouteState>
                    ) : null}
                  </div>
                </section>
                <section aria-label={t("addons.credentials.grantsSectionAria")}>
                  <h3>{t("addons.credentials.acceptedGrants")}</h3>
                  <div className="addonsFactList">
                    {summary.grants.map((grant) => (
                      <FactRow
                        badge={
                          grant.libraryId
                            ? t("addons.credentials.libraryScoped")
                            : t("addons.credentials.global")
                        }
                        detail={grant.id}
                        key={grant.id}
                        label={grant.permission}
                        tone="info"
                        value={grant.libraryId ?? t("addons.credentials.allLibraries")}
                      />
                    ))}
                    {summary.grants.length === 0 ? (
                      <EmptyRouteState>{t("addons.credentials.grantsEmpty")}</EmptyRouteState>
                    ) : null}
                  </div>
                </section>
              </div>
            </DataPanel>

            <DataPanel
              description={t("addons.install.description")}
              title={t("addons.install.title")}
            >
              {summary.installBoundary ? (
                <div className="addonsFactList">
                  <p className="addonsBoundaryMessage">
                    {installBoundaryMessage(summary.installBoundary, t)}
                  </p>
                  <FactRow
                    badge={
                      summary.installBoundary.nakoManagesContainers
                        ? t("addons.install.managed")
                        : t("addons.install.operatorOwned")
                    }
                    label={t("addons.install.containers")}
                    tone={summary.installBoundary.nakoManagesContainers ? "warning" : "neutral"}
                    value={
                      summary.installBoundary.nakoManagesContainers
                        ? t("addons.install.nakoManaged")
                        : t("addons.install.externalLifecycle")
                    }
                  />
                  <FactRow
                    badge={
                      summary.installBoundary.nakoManagesProcesses
                        ? t("addons.install.managed")
                        : t("addons.install.operatorOwned")
                    }
                    label={t("addons.install.processes")}
                    tone={summary.installBoundary.nakoManagesProcesses ? "warning" : "neutral"}
                    value={
                      summary.installBoundary.nakoManagesProcesses
                        ? t("addons.install.nakoManaged")
                        : t("addons.install.externalLifecycle")
                    }
                  />
                  <FactRow
                    label={t("addons.install.secretReferences")}
                    value={t("addons.install.secretReferencesValue", {
                      count: summary.installBoundary.secretReferenceCount,
                    })}
                  />
                  <FactRow
                    detail={t("addons.install.registrationChecks", {
                      count: summary.installBoundary.registrationVerificationStepCount,
                    })}
                    label={t("addons.install.verificationSteps")}
                    value={t("addons.install.healthChecks", {
                      count: summary.installBoundary.healthCheckStepCount,
                    })}
                  />
                </div>
              ) : (
                <EmptyRouteState>{t("addons.install.empty")}</EmptyRouteState>
              )}
            </DataPanel>
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadAddons(
  dataSource: AdminDataSource,
  search: AddonsSearch,
  unavailableMessage: string,
): Promise<AddonsResult> {
  if (!dataSource.loadAddons) {
    return {
      value: mockAddonsRouteSummary,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadAddons(toAdminAddonsQuery(search));
}

function toAdminAddonsQuery(search: AddonsSearch) {
  return {
    status: search.status,
  };
}

function addonStatusInput(value: string): AddonStatus | undefined {
  if (value === "enabled" || value === "disabled" || value === "unregistered") {
    return value;
  }

  return undefined;
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<AddonsRouteRow>> {
  return [
    {
      accessorKey: "name",
      header: t("addons.column.addon"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.name}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: t("addons.column.status"),
      cell: ({ row }) => <AddonStatusBadge status={row.original.status} />,
    },
    {
      accessorKey: "version",
      header: t("addons.column.version"),
    },
    {
      accessorKey: "protocolVersion",
      header: t("addons.column.protocol"),
    },
    {
      accessorKey: "grantedScopeCount",
      header: t("addons.column.grantedScopes"),
    },
    {
      accessorKey: "updatedAt",
      header: t("addons.column.updated"),
      cell: ({ row }) => timestampLabel(row.original.updatedAt),
    },
  ];
}

function SummaryCard({
  badge,
  label,
  tone,
  value,
}: {
  badge: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="settingsSummaryCard">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function FactRow({
  badge,
  detail,
  label,
  tone = "neutral",
  value,
}: {
  badge?: string;
  detail?: string;
  label: string;
  tone?: BadgeTone;
  value: string;
}) {
  return (
    <div className="addonsFactRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail ? <small>{detail}</small> : null}
      </div>
      {badge ? <Badge tone={tone}>{badge}</Badge> : null}
    </div>
  );
}

function CountTile({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="addonsCountTile">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function AddonStatusBadge({ status }: { status: string }) {
  return <Badge tone={addonStatusTone(status)}>{status}</Badge>;
}

function addonStatusTone(status: string): BadgeTone {
  if (status === "enabled") {
    return "success";
  }

  if (status === "disabled") {
    return "warning";
  }

  return "neutral";
}

function healthTone(status: string | undefined): BadgeTone {
  if (status === "reachable") {
    return "success";
  }

  if (status === "degraded" || status === "protocol_mismatch" || status === "invalid_manifest") {
    return "warning";
  }

  if (status === "unhealthy" || status === "unreachable") {
    return "danger";
  }

  return "neutral";
}

function runtimePolicy(addon: AddonsRouteSummary["selectedAddon"], t: Translate) {
  if (!addon) {
    return t("addons.runtime.notAvailable");
  }

  const timeout = addon.defaultTimeoutMs
    ? t("addons.runtime.timeoutMs", { count: addon.defaultTimeoutMs })
    : t("addons.runtime.defaultTimeout");
  const attempts = addon.defaultMaxAttempts
    ? t("addons.runtime.attempts", { count: addon.defaultMaxAttempts })
    : t("addons.runtime.defaultAttempts");
  return `${timeout} / ${attempts}`;
}

function installBoundaryMessage(boundary: AddonsRouteSummary["installBoundary"], t: Translate) {
  if (!boundary) {
    return t("addons.install.empty");
  }

  const managedCount = [
    boundary.nakoManagesContainers,
    boundary.nakoManagesProcesses,
    boundary.nakoManagesPackages,
  ].filter(Boolean).length;

  if (managedCount === 0) {
    return t("addons.install.message.operatorOwned");
  }

  if (managedCount === 3) {
    return t("addons.install.message.nakoManaged");
  }

  return t("addons.install.message.partial");
}

function timestampLabel(value: string) {
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) {
    return value;
  }

  return new Date(parsed).toISOString();
}
