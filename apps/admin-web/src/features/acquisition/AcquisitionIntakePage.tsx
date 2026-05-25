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
  AdminAcquisitionIntakeCandidateDiagnostic,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
} from "../../adminApi/types";
import { mockAcquisitionIntakeCandidates } from "../../adminApi/mockData";
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

export type AcquisitionIntakeSearch = {
  library_id?: string;
  state?: string;
  source_kind?: string;
  managed_import_artifact_id?: string;
  limit: number;
  offset: number;
};

export type AcquisitionIntakePageProps = {
  dataSource: AdminDataSource;
  search: AcquisitionIntakeSearch;
  onSearchChange(next: Partial<AcquisitionIntakeSearch>): void;
};

type AcquisitionIntakeResult = {
  value: AdminAcquisitionIntakeCandidateListResponse;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

const columns: Array<ColumnDef<AdminAcquisitionIntakeCandidateDiagnostic>> = [
  {
    accessorKey: "id",
    header: "Candidate",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.id}</strong>
        <span>{row.original.target_library_id}</span>
      </div>
    ),
  },
  {
    accessorKey: "state",
    header: "State",
    cell: ({ row }) => <Badge tone={candidateStateTone(row.original.state)}>{row.original.state}</Badge>,
  },
  {
    accessorKey: "source_kind",
    header: "Source",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.source_kind}</strong>
        <span>{row.original.source_scheme ?? "unknown scheme"}</span>
      </div>
    ),
  },
  {
    accessorKey: "size_bytes",
    header: "Size",
    cell: ({ row }) => formatBytes(row.original.size_bytes),
  },
  {
    id: "diagnostics",
    header: "Diagnostics",
    cell: ({ row }) => (
      <Badge tone={row.original.has_diagnostics ? "info" : "neutral"}>
        {row.original.has_diagnostics ? "available" : "none"}
      </Badge>
    ),
  },
  {
    accessorKey: "managed_import_artifact_id",
    header: "Managed Import",
    cell: ({ row }) => row.original.managed_import_artifact_id ?? "not linked",
  },
  {
    accessorKey: "first_seen_at_ms",
    header: "First Seen",
    cell: ({ row }) => timestampLabel(row.original.first_seen_at_ms),
  },
  {
    accessorKey: "updated_at_ms",
    header: "Updated",
    cell: ({ row }) => timestampLabel(row.original.updated_at_ms),
  },
];

export function AcquisitionIntakePage({
  dataSource,
  search,
  onSearchChange,
}: AcquisitionIntakePageProps) {
  const query = useQuery({
    queryKey: ["admin-acquisition-intake", search],
    queryFn: () => loadAcquisitionIntake(dataSource, search),
  });
  const result = query.data ?? {
    value: mockAcquisitionIntakeCandidates,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () =>
      [
        search.library_id,
        search.state,
        search.source_kind,
        search.managed_import_artifact_id,
      ].filter(Boolean).length,
    [search],
  );
  const table = useReactTable({
    data: result.value.candidates,
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
      description="Watch-folder candidates before Managed Import and promotion apply, without raw locators or filesystem paths."
      kicker="Acquisition"
      status={<SourceLabel source={result.source} />}
      title="Acquisition Intake"
      titleId="acquisition-intake-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Acquisition intake filters">
        <FilterField label="Library">
          <input
            aria-label="Intake library filter"
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="State">
          <select
            aria-label="Intake state filter"
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">Any state</option>
            <option value="ready">Ready</option>
            <option value="blocked">Blocked</option>
            <option value="incomplete">Incomplete</option>
            <option value="unsupported">Unsupported</option>
          </select>
        </FilterField>
        <FilterField label="Source Kind">
          <input
            aria-label="Intake source kind filter"
            placeholder="watch_folder"
            value={search.source_kind ?? ""}
            onChange={(event) => onSearchChange({ source_kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="Managed Import">
          <input
            aria-label="Intake managed import filter"
            placeholder="artifact-id"
            value={search.managed_import_artifact_id ?? ""}
            onChange={(event) =>
              onSearchChange({
                managed_import_artifact_id: event.target.value || undefined,
                offset: 0,
              })
            }
          />
        </FilterField>
        <FilterField label="Limit">
          <input
            aria-label="Intake page limit"
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label="Offset">
          <input
            aria-label="Intake page offset"
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {activeFilterCount} filters
          </Badge>
          <Button
            disabled={activeFilterCount === 0 && search.offset === 0}
            onClick={() =>
              onSearchChange({
                library_id: undefined,
                state: undefined,
                source_kind: undefined,
                managed_import_artifact_id: undefined,
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
        title="Intake candidates"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Acquisition Intake candidates" /> : null}

        {!query.isLoading && result.value.candidates.length === 0 ? (
          <EmptyRouteState>No Acquisition Intake candidates match the current filters.</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.candidates.length > 0 ? (
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

async function loadAcquisitionIntake(
  dataSource: AdminDataSource,
  search: AcquisitionIntakeSearch,
): Promise<AcquisitionIntakeResult> {
  if (!dataSource.loadAcquisitionIntake) {
    return {
      value: mockAcquisitionIntakeCandidates,
      source: "mock",
      error: "Acquisition Intake route data source is unavailable",
    };
  }

  return dataSource.loadAcquisitionIntake(toAdminAcquisitionIntakeQuery(search));
}

function toAdminAcquisitionIntakeQuery(
  search: AcquisitionIntakeSearch,
): AdminAcquisitionIntakeCandidatesQuery {
  return {
    library_id: search.library_id,
    state: search.state,
    source_kind: search.source_kind,
    managed_import_artifact_id: search.managed_import_artifact_id,
    limit: search.limit,
    offset: search.offset,
  };
}

function candidateStateTone(state: string): BadgeTone {
  if (state === "ready") {
    return "success";
  }

  if (state === "blocked" || state === "unsupported") {
    return "danger";
  }

  return "warning";
}

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}

function nonNegativeNumberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : undefined;
}

function timestampLabel(value: number) {
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
