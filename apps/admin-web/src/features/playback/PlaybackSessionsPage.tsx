import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { RefreshCw, Search, X } from "lucide-react";
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminPlaybackSessionListItem,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSessionsQuery,
} from "../../adminApi/types";
import { mockPlaybackSessions } from "../../adminApi/mockData";
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

export type PlaybackSessionsSearch = {
  source_id?: string;
  kind?: string;
  state?: string;
  limit: number;
  offset: number;
};

export type PlaybackSessionsPageProps = {
  dataSource: AdminDataSource;
  search: PlaybackSessionsSearch;
  onSearchChange(next: Partial<PlaybackSessionsSearch>): void;
};

type PlaybackSessionsResult = {
  value: AdminPlaybackSessionListResponse;
  source: DataSourceMode;
  error?: string;
};

const columns: Array<ColumnDef<AdminPlaybackSessionListItem>> = [
  {
    accessorKey: "kind",
    header: "Session",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.kind}</strong>
        <span>{row.original.id}</span>
      </div>
    ),
  },
  {
    accessorKey: "state",
    header: "State",
    cell: ({ row }) => <SessionStateBadge session={row.original} />,
  },
  {
    accessorKey: "source_id",
    header: "Media Source",
  },
  {
    id: "lifecycle",
    header: "Lifecycle",
    cell: ({ row }) => lifecycleLabel(row.original),
  },
  {
    accessorKey: "failure_category",
    header: "Failure",
    cell: ({ row }) => row.original.failure_category ?? "none",
  },
  {
    accessorKey: "updated_at",
    header: "Updated",
  },
];

export function PlaybackSessionsPage({
  dataSource,
  search,
  onSearchChange,
}: PlaybackSessionsPageProps) {
  const query = useQuery({
    queryKey: ["admin-playback-sessions", search],
    queryFn: () => loadPlaybackSessions(dataSource, search),
  });
  const result = query.data ?? {
    value: mockPlaybackSessions,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.source_id, search.kind, search.state].filter(Boolean).length,
    [search.kind, search.source_id, search.state],
  );
  const table = useReactTable({
    data: result.value.sessions,
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
      description="Playback and transcode sessions with route-owned filters and support evidence deferred to detail workflows."
      kicker="Playback operations"
      status={<SourceLabel source={result.source} />}
      title="Playback Sessions"
      titleId="playback-sessions-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Playback session filters">
        <FilterField label="Source">
          <input
            aria-label="Playback source filter"
            placeholder="source-id"
            value={search.source_id ?? ""}
            onChange={(event) => onSearchChange({ source_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="Kind">
          <input
            aria-label="Playback kind filter"
            placeholder="hls_transcode"
            value={search.kind ?? ""}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="State">
          <select
            aria-label="Playback state filter"
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">Any state</option>
            <option value="starting">Starting</option>
            <option value="running">Running</option>
            <option value="failed">Failed</option>
            <option value="completed">Completed</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </FilterField>
        <FilterField label="Limit">
          <input
            aria-label="Playback page limit"
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {activeFilterCount} filters
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                source_id: undefined,
                kind: undefined,
                state: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            Clear
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={`${result.value.page.returned} returned, offset ${result.value.page.offset}, limit ${result.value.page.limit}`}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            URL filters are authoritative
          </div>
        }
        title="Session queue"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Playback Sessions" /> : null}

        {!query.isLoading && result.value.sessions.length === 0 ? (
          <EmptyRouteState>No Playback Sessions match the current filters.</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.sessions.length > 0 ? (
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

async function loadPlaybackSessions(
  dataSource: AdminDataSource,
  search: PlaybackSessionsSearch,
): Promise<PlaybackSessionsResult> {
  if (!dataSource.loadPlaybackSessions) {
    return {
      value: mockPlaybackSessions,
      source: "mock",
      error: "Playback Sessions route data source is unavailable",
    };
  }

  return dataSource.loadPlaybackSessions(toAdminPlaybackSessionsQuery(search));
}

function toAdminPlaybackSessionsQuery(search: PlaybackSessionsSearch): AdminPlaybackSessionsQuery {
  return {
    source_id: search.source_id,
    kind: search.kind,
    state: search.state,
    limit: search.limit,
    offset: search.offset,
  };
}

function SessionStateBadge({ session }: { session: AdminPlaybackSessionListItem }) {
  if (session.failure_category || session.state === "failed") {
    return <Badge tone="danger">{session.state}</Badge>;
  }

  if (session.state === "running") {
    return <Badge tone="info">{session.state}</Badge>;
  }

  if (session.state === "starting") {
    return <Badge tone="warning">{session.state}</Badge>;
  }

  return <Badge tone={session.terminal ? "success" : "neutral"}>{session.state}</Badge>;
}

function lifecycleLabel(session: AdminPlaybackSessionListItem) {
  if (session.terminal) {
    return "terminal";
  }

  if (session.active) {
    return "active";
  }

  return "inactive";
}

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}
