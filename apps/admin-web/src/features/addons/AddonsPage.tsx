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

const columns: Array<ColumnDef<AddonsRouteRow>> = [
  {
    accessorKey: "name",
    header: "Addon",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.name}</strong>
        <span>{row.original.id}</span>
      </div>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <AddonStatusBadge status={row.original.status} />,
  },
  {
    accessorKey: "version",
    header: "Addon Version",
  },
  {
    accessorKey: "protocolVersion",
    header: "Protocol",
  },
  {
    accessorKey: "grantedScopeCount",
    header: "Granted Scopes",
  },
  {
    accessorKey: "updatedAt",
    header: "Updated",
    cell: ({ row }) => timestampLabel(row.original.updatedAt),
  },
];

export function AddonsPage({
  dataSource,
  search,
  onSearchChange,
}: AddonsPageProps) {
  const query = useQuery({
    queryKey: ["admin-addons", search],
    queryFn: () => loadAddons(dataSource, search),
  });
  const result = query.data ?? {
    value: mockAddonsRouteSummary,
    source: "mock" as const,
  };
  const summary = result.value;
  const activeFilterCount = useMemo(() => (search.status ? 1 : 0), [search.status]);
  const table = useReactTable({
    data: summary.addons,
    columns,
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
          Refresh
        </Button>
      }
      description="Registered Addons, health, grants, token prefixes, declared surfaces, and install ownership without credential material."
      kicker="Addon operations"
      status={<SourceLabel source={result.source} />}
      title="Addons"
      titleId="addons-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Addon filters">
        <FilterField label="Status">
          <select
            aria-label="Addon status filter"
            value={search.status ?? ""}
            onChange={(event) =>
              onSearchChange({
                status: addonStatusInput(event.target.value),
              })
            }
          >
            <option value="">Any status</option>
            <option value="enabled">Enabled</option>
            <option value="disabled">Disabled</option>
            <option value="unregistered">Unregistered</option>
          </select>
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {activeFilterCount} filters
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() => onSearchChange({ status: undefined })}
            variant="ghost"
          >
            <X size={16} />
            Clear
          </Button>
        </FilterActions>
      </FilterBar>

      {query.isLoading ? <RowsSkeleton label="Loading Addons" /> : null}

      {!query.isLoading ? (
        <>
          <div className="addonsSummaryGrid">
            <SummaryCard
              badge={`${summary.statusCounts.enabled} enabled`}
              label="Registered Addons"
              tone={summary.statusCounts.enabled > 0 ? "success" : "neutral"}
              value={summary.addons.length.toString()}
            />
            <SummaryCard
              badge={summary.health?.status ?? "not checked"}
              label="Selected health"
              tone={healthTone(summary.health?.status)}
              value={summary.selectedAddon?.name ?? "None"}
            />
            <SummaryCard
              badge={`${summary.tokens.length} tracked`}
              label="Token prefixes"
              tone={summary.tokens.some((token) => token.status === "active") ? "success" : "warning"}
              value={`${summary.tokens.filter((token) => token.status === "active").length} active`}
            />
            <SummaryCard
              badge={`${summary.grants.length} accepted`}
              label="Addon Permissions"
              tone={summary.grants.length > 0 ? "info" : "warning"}
              value={`${summary.selectedAddon?.grantedScopeCount ?? 0} scopes`}
            />
          </div>

          <DataPanel
            description={`${summary.addons.length} Addons returned by the route-owned status filter`}
            headerAccessory={
              <div className="searchHint">
                <Search size={15} />
                URL filters are authoritative
              </div>
            }
            title="Addon registry"
          >
            {summary.addons.length === 0 ? (
              <EmptyRouteState>No Addons match the current filters.</EmptyRouteState>
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
              description="Manifest and registration facts after stripping runtime endpoints, paths, and raw manifest payloads."
              title="Selected Addon"
            >
              {summary.selectedAddon ? (
                <div className="addonsFactList">
                  <FactRow
                    badge={summary.selectedAddon.status}
                    label="Registration"
                    tone={addonStatusTone(summary.selectedAddon.status)}
                    value={summary.selectedAddon.manifestId}
                  />
                  <FactRow
                    detail={summary.selectedAddon.resourceKinds.join(", ") || "none declared"}
                    label="Addon Resources"
                    value={`${summary.selectedAddon.resourceCount} declared`}
                  />
                  <FactRow
                    detail={summary.selectedAddon.grantedScopes.join(", ") || "none granted"}
                    label="Granted scopes"
                    value={`${summary.selectedAddon.grantedScopeCount} scopes`}
                  />
                  <FactRow
                    detail={runtimePolicy(summary.selectedAddon)}
                    label="Runtime auth"
                    value={summary.selectedAddon.authMode}
                  />
                </div>
              ) : (
                <EmptyRouteState>No selected Addon is available.</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description="Reachability and protocol facts from the Addon Health Check response."
              title="Health"
            >
              {summary.health ? (
                <div className="addonsFactList">
                  <FactRow
                    badge={summary.health.status}
                    label="Health status"
                    tone={healthTone(summary.health.status)}
                    value={`${summary.health.latencyMs} ms`}
                  />
                  <FactRow
                    label="Protocol version"
                    value={summary.health.protocolVersion ?? "not reported"}
                  />
                  <FactRow
                    label="Addon version"
                    value={summary.health.addonVersion ?? "not reported"}
                  />
                  <FactRow
                    label="Safe error code"
                    value={summary.health.safeErrorCode ?? "none"}
                  />
                </div>
              ) : (
                <EmptyRouteState>No Addon Health Check response is available.</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description="Declared Addon Entry Points, Addon Hosted Pages, tasks, events, and schema presence as counts only."
              title="Surface declarations"
            >
              {summary.surfaceSummary ? (
                <div className="addonsCountGrid">
                  <CountTile label="Entry Points" value={summary.surfaceSummary.entryPointCount} />
                  <CountTile label="Hosted Pages" value={summary.surfaceSummary.hostedPageCount} />
                  <CountTile label="Addon Tasks" value={summary.surfaceSummary.taskCount} />
                  <CountTile label="Event Subscriptions" value={summary.surfaceSummary.eventSubscriptionCount} />
                  <CountTile label="Secret Reference Fields" value={summary.surfaceSummary.secretReferenceFieldCount} />
                  <CountTile
                    label="Configuration Schema"
                    value={summary.surfaceSummary.configurationSchemaDeclared ? "declared" : "not declared"}
                  />
                </div>
              ) : (
                <EmptyRouteState>No Addon surface declarations are available.</EmptyRouteState>
              )}
            </DataPanel>

            <DataPanel
              description="Token summaries expose prefixes only. Raw one-time tokens and credential-producing actions stay in follow-ons."
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  Credentials redacted
                </div>
              }
              title="Credentials and grants"
            >
              <div className="addonsCredentialGrid">
                <section aria-label="Addon token prefixes">
                  <h3>Token prefixes</h3>
                  <div className="addonsFactList">
                    {summary.tokens.map((token) => (
                      <FactRow
                        badge={token.status}
                        detail={token.lastUsedAt ? `last used ${timestampLabel(token.lastUsedAt)}` : "never used"}
                        key={token.id}
                        label={token.label}
                        tone={token.status === "active" ? "success" : "neutral"}
                        value={token.tokenPrefix}
                      />
                    ))}
                    {summary.tokens.length === 0 ? (
                      <EmptyRouteState>No Addon Token summaries are available.</EmptyRouteState>
                    ) : null}
                  </div>
                </section>
                <section aria-label="Addon permission grants">
                  <h3>Accepted grants</h3>
                  <div className="addonsFactList">
                    {summary.grants.map((grant) => (
                      <FactRow
                        badge={grant.libraryId ? "library scoped" : "global"}
                        detail={grant.id}
                        key={grant.id}
                        label={grant.permission}
                        tone="info"
                        value={grant.libraryId ?? "all libraries"}
                      />
                    ))}
                    {summary.grants.length === 0 ? (
                      <EmptyRouteState>No Addon Permission grants are configured.</EmptyRouteState>
                    ) : null}
                  </div>
                </section>
              </div>
            </DataPanel>

            <DataPanel
              description="Install Guide ownership summary without snippets, shell commands, env var names, URLs, or paths."
              title="Install boundary"
            >
              {summary.installBoundary ? (
                <div className="addonsFactList">
                  <p className="addonsBoundaryMessage">
                    {installBoundaryMessage(summary.installBoundary)}
                  </p>
                  <FactRow
                    badge={summary.installBoundary.nakoManagesContainers ? "managed" : "operator owned"}
                    label="Containers"
                    tone={summary.installBoundary.nakoManagesContainers ? "warning" : "neutral"}
                    value={summary.installBoundary.nakoManagesContainers ? "Nako managed" : "External lifecycle"}
                  />
                  <FactRow
                    badge={summary.installBoundary.nakoManagesProcesses ? "managed" : "operator owned"}
                    label="Processes"
                    tone={summary.installBoundary.nakoManagesProcesses ? "warning" : "neutral"}
                    value={summary.installBoundary.nakoManagesProcesses ? "Nako managed" : "External lifecycle"}
                  />
                  <FactRow
                    label="Secret References"
                    value={`${summary.installBoundary.secretReferenceCount} declared`}
                  />
                  <FactRow
                    detail={`${summary.installBoundary.registrationVerificationStepCount} registration checks`}
                    label="Verification steps"
                    value={`${summary.installBoundary.healthCheckStepCount} health checks`}
                  />
                </div>
              ) : (
                <EmptyRouteState>No Addon Install Guide boundary is available.</EmptyRouteState>
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
): Promise<AddonsResult> {
  if (!dataSource.loadAddons) {
    return {
      value: mockAddonsRouteSummary,
      source: "mock",
      error: "Addons route data source is unavailable",
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

function runtimePolicy(addon: AddonsRouteSummary["selectedAddon"]) {
  if (!addon) {
    return "not available";
  }

  const timeout = addon.defaultTimeoutMs ? `${addon.defaultTimeoutMs} ms` : "default timeout";
  const attempts = addon.defaultMaxAttempts ? `${addon.defaultMaxAttempts} attempts` : "default attempts";
  return `${timeout} / ${attempts}`;
}

function installBoundaryMessage(boundary: AddonsRouteSummary["installBoundary"]) {
  if (!boundary) {
    return "No Addon Install Guide boundary is available.";
  }

  const managedCount = [
    boundary.nakoManagesContainers,
    boundary.nakoManagesProcesses,
    boundary.nakoManagesPackages,
  ].filter(Boolean).length;

  if (managedCount === 0) {
    return "Nako reports the operator owns Addon Sidecar installation, start/stop, upgrades, logs, and removal outside Nako.";
  }

  if (managedCount === 3) {
    return "Nako reports Addon Sidecar lifecycle is managed inside Nako.";
  }

  return "Nako reports partial Addon Sidecar lifecycle management. Review dedicated mutation workflows before changing lifecycle state.";
}

function timestampLabel(value: string) {
  const parsed = Date.parse(value);
  if (Number.isNaN(parsed)) {
    return value;
  }

  return new Date(parsed).toISOString();
}
