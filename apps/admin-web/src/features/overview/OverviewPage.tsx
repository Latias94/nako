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

const storageColumns: Array<ColumnDef<StorageBackend>> = [
  {
    accessorKey: "library_name",
    header: "Media Library",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.library_name}</strong>
        <span>{row.original.library_id}</span>
      </div>
    ),
  },
  {
    accessorKey: "backend_kind",
    header: "Backend",
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
];

const metadataColumns: Array<ColumnDef<MetadataProvider>> = [
  {
    accessorKey: "provider",
    header: "Provider",
    cell: ({ row }) => row.original.provider.toUpperCase(),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <StatusBadge status={row.original.status} />,
  },
];

export function OverviewPage({ dataSource }: OverviewPageProps) {
  const query = useQuery({
    queryKey: ["admin-overview"],
    queryFn: () => loadOverview(dataSource),
  });
  const result = query.data ?? {
    value: mockOverview,
    source: "mock" as const,
  };
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
      description="Server health, runtime counters, Media Library storage, and provider availability from the Admin overview read model."
      kicker="Operations"
      status={<SourceLabel source={result.source} />}
      title="Overview"
      titleId="overview-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label="Loading Overview" /> : null}

      {!query.isLoading ? (
        <>
          <div className="overviewMetricGrid">
            <OverviewMetric
              badge={result.value.status === "healthy" ? "Healthy" : "Degraded"}
              label="Server status"
              value={result.value.status}
              tone={result.value.status === "healthy" ? "success" : "warning"}
            />
            <OverviewMetric
              badge={storageBadge(result.value)}
              label="Storage backends"
              value={`${result.value.storage.ready_backends}/${result.value.storage.total_backends} ready`}
              tone={storageTone(result.value)}
            />
            <OverviewMetric
              badge="Running"
              label="Active tasks"
              value={result.value.runtime.active_tasks.toString()}
              tone="info"
            />
            <OverviewMetric
              badge={result.value.runtime.failed_jobs > 0 ? "Attention" : "Clear"}
              label="Failed jobs"
              value={result.value.runtime.failed_jobs.toString()}
              tone={result.value.runtime.failed_jobs > 0 ? "warning" : "success"}
            />
            <OverviewMetric
              badge="Configured"
              label="Configured libraries"
              value={result.value.startup.configured_libraries.toString()}
              tone="neutral"
            />
            <OverviewMetric
              badge="Recovered"
              label="Recovered jobs"
              value={result.value.startup.recovered_jobs.toString()}
              tone="neutral"
            />
          </div>

          <DataPanel
            description={`${result.value.storage.ready_backends} ready, ${result.value.storage.degraded_backends} degraded, ${result.value.storage.unavailable_backends} unavailable`}
            title="Storage backends"
          >
            <OverviewTable table={storageTable} />
          </DataPanel>

          <DataPanel
            description={`${result.value.metadata.available_providers} available, ${result.value.metadata.disabled_providers} disabled, ${result.value.metadata.unavailable_providers} unavailable`}
            title="Metadata providers"
          >
            <OverviewTable table={metadataTable} />
          </DataPanel>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadOverview(dataSource: AdminDataSource): Promise<OverviewResult> {
  if (!dataSource.loadOverview) {
    return {
      value: mockOverview,
      source: "mock",
      error: "Overview route data source is unavailable",
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

function storageBadge(overview: AdminOverviewResponse) {
  if (overview.storage.unavailable_backends > 0) {
    return "Unavailable";
  }

  if (overview.storage.degraded_backends > 0) {
    return "Degraded";
  }

  return "Ready";
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
