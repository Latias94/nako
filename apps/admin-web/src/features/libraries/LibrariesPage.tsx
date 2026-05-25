import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { RefreshCw, ShieldCheck } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type { AdminServerConfigDiagnosticsResponse } from "../../adminApi/types";
import { mockSystemConfig } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
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

export type LibrariesPageProps = {
  dataSource: AdminDataSource;
};

type LibrariesResult = {
  value: AdminServerConfigDiagnosticsResponse;
  source: DataSourceMode;
  error?: string;
};

type LibraryConfigDiagnostics = AdminServerConfigDiagnosticsResponse["libraries"][number];

const columns: Array<ColumnDef<LibraryConfigDiagnostics>> = [
  {
    accessorKey: "name",
    header: "Media Library",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.name}</strong>
        <span>{row.original.id}</span>
      </div>
    ),
  },
  {
    accessorKey: "preset",
    header: "Preset",
  },
  {
    accessorKey: "backend_kind",
    header: "Backend",
    cell: ({ row }) => <Badge tone="info">{row.original.backend_kind}</Badge>,
  },
  {
    accessorKey: "root_scheme",
    header: "Root Scheme",
  },
  {
    accessorKey: "has_webdav_password_env",
    header: "Secret Reference",
    cell: ({ row }) => <SecretReferenceBadge library={row.original} />,
  },
  {
    id: "runtime",
    header: "Runtime Policy",
    cell: ({ row }) => runtimePolicyLabel(row.original),
  },
];

export function LibrariesPage({ dataSource }: LibrariesPageProps) {
  const query = useQuery({
    queryKey: ["admin-libraries"],
    queryFn: () => loadLibraries(dataSource),
  });
  const result = query.data ?? {
    value: mockSystemConfig,
    source: "mock" as const,
  };
  const libraries = result.value.libraries;
  const table = useReactTable({
    data: libraries,
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
      description="Configured Media Library boundaries from redacted Admin system diagnostics."
      kicker="Library operations"
      status={<SourceLabel source={result.source} />}
      title="Media Libraries"
      titleId="libraries-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <DataPanel
        description={`${libraries.length} configured from Admin system config diagnostics`}
        headerAccessory={
          <div className="searchHint">
            <ShieldCheck size={15} />
            Root references stay redacted
          </div>
        }
        title="Configured libraries"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Media Libraries" /> : null}

        {!query.isLoading && libraries.length === 0 ? (
          <EmptyRouteState>No Media Libraries are configured.</EmptyRouteState>
        ) : null}

        {!query.isLoading && libraries.length > 0 ? (
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

async function loadLibraries(dataSource: AdminDataSource): Promise<LibrariesResult> {
  if (!dataSource.loadLibraries) {
    return {
      value: mockSystemConfig,
      source: "mock",
      error: "Media Libraries route data source is unavailable",
    };
  }

  return dataSource.loadLibraries();
}

function SecretReferenceBadge({ library }: { library: LibraryConfigDiagnostics }) {
  if (library.backend_kind !== "webdav") {
    return <Badge tone="neutral">Not required</Badge>;
  }

  if (library.has_webdav_password_env) {
    return <Badge tone="success">Secret Reference configured</Badge>;
  }

  return <Badge tone="warning">Secret Reference missing</Badge>;
}

function runtimePolicyLabel(library: LibraryConfigDiagnostics) {
  const timeout = library.webdav_timeout_ms ? `${library.webdav_timeout_ms} ms` : "default timeout";
  const attempts = library.webdav_max_attempts ? `${library.webdav_max_attempts} attempts` : "default attempts";

  if (library.backend_kind !== "webdav") {
    return "local policy";
  }

  return `${timeout} / ${attempts}`;
}
