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
  AdminStorageStagingDiagnosticsResponse,
  AdminStorageStagingQuery,
} from "../../adminApi/types";
import { mockStorageStaging } from "../../adminApi/mockData";
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

const columns: Array<ColumnDef<StorageRecord>> = [
  {
    accessorKey: "id",
    header: "Record",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.id}</strong>
        <span>{row.original.purpose}</span>
      </div>
    ),
  },
  {
    accessorKey: "state",
    header: "State",
    cell: ({ row }) => <StorageStateBadge record={row.original} />,
  },
  {
    accessorKey: "source_scheme",
    header: "Source Scheme",
  },
  {
    accessorKey: "size_bytes",
    header: "Size",
    cell: ({ row }) => formatBytes(row.original.size_bytes),
  },
  {
    accessorKey: "active_leases",
    header: "Leases",
  },
  {
    id: "validation",
    header: "Validation",
    cell: ({ row }) => (row.original.has_validation_error ? "failed" : "clean"),
  },
  {
    accessorKey: "expires_at_ms",
    header: "Expires",
    cell: ({ row }) => timestampLabel(row.original.expires_at_ms),
  },
];

export function StorageStagingPage({
  dataSource,
  search,
  onSearchChange,
}: StorageStagingPageProps) {
  const query = useQuery({
    queryKey: ["admin-storage-staging", search],
    queryFn: () => loadStorageStaging(dataSource, search),
  });
  const result = query.data ?? {
    value: mockStorageStaging,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.purpose, search.state].filter(Boolean).length,
    [search.purpose, search.state],
  );
  const table = useReactTable({
    data: result.value.records,
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
      description="Staging records and VFS cache health without roots, Source Locators, cache URIs, or credentials."
      kicker="Storage operations"
      status={<SourceLabel source={result.source} />}
      title="Storage Staging"
      titleId="storage-staging-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Storage staging filters">
        <FilterField label="Purpose">
          <input
            aria-label="Storage purpose filter"
            placeholder="ffmpeg_input"
            value={search.purpose ?? ""}
            onChange={(event) => onSearchChange({ purpose: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="State">
          <select
            aria-label="Storage state filter"
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">Any state</option>
            <option value="ready">Ready</option>
            <option value="failed">Failed</option>
            <option value="stale">Stale</option>
            <option value="expired">Expired</option>
          </select>
        </FilterField>
        <FilterField label="Limit">
          <input
            aria-label="Storage page limit"
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
                purpose: undefined,
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
        description={`${result.value.page.returned} returned, ${formatBytes(result.value.summary.used_manifest_bytes)} used of ${formatBytes(result.value.summary.configured_max_bytes)}`}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            URL filters are authoritative
          </div>
        }
        title="Staging records"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Storage Staging records" /> : null}

        {!query.isLoading && result.value.records.length === 0 ? (
          <EmptyRouteState>No Storage Staging records match the current filters.</EmptyRouteState>
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

async function loadStorageStaging(
  dataSource: AdminDataSource,
  search: StorageStagingSearch,
): Promise<StorageStagingResult> {
  if (!dataSource.loadStorageStaging) {
    return {
      value: mockStorageStaging,
      source: "mock",
      error: "Storage Staging route data source is unavailable",
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

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}

function timestampLabel(value: number | null) {
  if (value === null) {
    return "none";
  }

  return new Date(value).toISOString();
}

function formatBytes(value: number | null) {
  if (value === null) {
    return "unknown";
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
