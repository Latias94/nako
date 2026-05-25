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
  AdminJobListItem,
  AdminJobListResponse,
  AdminJobsQuery,
} from "../../adminApi/types";
import { mockJobs } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/Table";

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

const columns: Array<ColumnDef<AdminJobListItem>> = [
  {
    accessorKey: "kind",
    header: "Kind",
    cell: ({ row }) => (
      <div className="jobsPrimaryCell">
        <strong>{row.original.kind}</strong>
        <span>{row.original.id}</span>
      </div>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <JobStatusBadge status={row.original.status} hasError={row.original.has_error} />,
  },
  {
    accessorKey: "resource_class",
    header: "Resource",
  },
  {
    accessorKey: "library_id",
    header: "Media Library",
    cell: ({ row }) => row.original.library_id ?? "none",
  },
  {
    accessorKey: "source_id",
    header: "Media Source",
    cell: ({ row }) => row.original.source_id ?? "none",
  },
  {
    accessorKey: "queued_at",
    header: "Queued",
  },
];

export function JobsPage({ dataSource, search, onSearchChange }: JobsPageProps) {
  const query = useQuery({
    queryKey: ["admin-jobs", search],
    queryFn: () => loadJobs(dataSource, search),
  });
  const result = query.data ?? {
    value: mockJobs,
    source: "mock" as const,
  };

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

  return (
    <section className="routePage" aria-labelledby="jobs-route-title">
      <div className="routeHeader">
        <div>
          <p className="routeKicker">Operations</p>
          <div className="routeTitleLine">
            <h1 id="jobs-route-title">Jobs</h1>
            <SourceLabel source={result.source} />
          </div>
          <p>
            Durable background work with route-owned filters, generated Admin API
            DTOs, and section-local fallback.
          </p>
        </div>
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          Refresh
        </Button>
      </div>

      {result.error ? (
        <div className="routeNotice" role="status">
          {result.error}. Showing deterministic mock fallback data.
        </div>
      ) : null}

      <div className="filterBar" aria-label="Job filters">
        <label className="filterField">
          <span>Status</span>
          <select
            aria-label="Job status filter"
            value={search.status ?? ""}
            onChange={(event) => onSearchChange({ status: event.target.value || undefined, offset: 0 })}
          >
            <option value="">Any status</option>
            <option value="queued">Queued</option>
            <option value="running">Running</option>
            <option value="failed">Failed</option>
            <option value="succeeded">Succeeded</option>
            <option value="cancelled">Cancelled</option>
          </select>
        </label>
        <label className="filterField">
          <span>Kind</span>
          <input
            aria-label="Job kind filter"
            placeholder="metadata_refresh"
            value={search.kind ?? ""}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
          />
        </label>
        <label className="filterField">
          <span>Resource</span>
          <input
            aria-label="Job resource class filter"
            placeholder="library"
            value={search.resource_class ?? ""}
            onChange={(event) =>
              onSearchChange({ resource_class: event.target.value || undefined, offset: 0 })
            }
          />
        </label>
        <label className="filterField">
          <span>Library</span>
          <input
            aria-label="Job library filter"
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </label>
        <div className="filterActions">
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {activeFilterCount} filters
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
            Clear
          </Button>
        </div>
      </div>

      <div className="dataPanel">
        <div className="dataPanelHeader">
          <div>
            <h2>Job queue</h2>
            <p>
              {result.value.page.returned} returned, offset {result.value.page.offset},
              limit {result.value.page.limit}
            </p>
          </div>
          <div className="searchHint">
            <Search size={15} />
            URL filters are authoritative
          </div>
        </div>

        {query.isLoading ? <JobsSkeleton /> : null}

        {!query.isLoading && result.value.jobs.length === 0 ? (
          <div className="emptyRouteState">No jobs match the current filters.</div>
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
      </div>
    </section>
  );
}

async function loadJobs(dataSource: AdminDataSource, search: JobsSearch): Promise<JobsResult> {
  if (!dataSource.loadJobs) {
    return {
      value: mockJobs,
      source: "mock",
      error: "Jobs route data source is unavailable",
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

function JobsSkeleton() {
  return (
    <div className="jobsSkeleton" role="status">
      <span />
      <span />
      <span />
    </div>
  );
}
