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
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

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

export function OverviewPage({ dataSource }: OverviewPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-overview", locale],
    queryFn: () => loadOverview(dataSource, t("overview.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockOverview,
    source: "mock" as const,
  };
  const storageColumns = createStorageColumns(t);
  const metadataColumns = createMetadataColumns(t);
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
          {t("overview.refresh")}
        </Button>
      }
      description={t("overview.description")}
      kicker={t("overview.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("overview.title")}
      titleId="overview-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("overview.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("overview.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="overviewMetricGrid">
            <OverviewMetric
              badge={
                result.value.status === "healthy"
                  ? t("overview.metric.serverStatus.healthy")
                  : t("overview.metric.serverStatus.degraded")
              }
              label={t("overview.metric.serverStatus.label")}
              value={result.value.status}
              tone={result.value.status === "healthy" ? "success" : "warning"}
            />
            <OverviewMetric
              badge={storageBadge(result.value, t)}
              label={t("overview.metric.storage.label")}
              value={t("overview.metric.storage.value", {
                ready: result.value.storage.ready_backends,
                total: result.value.storage.total_backends,
              })}
              tone={storageTone(result.value)}
            />
            <OverviewMetric
              badge={t("overview.metric.activeTasks.badge")}
              label={t("overview.metric.activeTasks.label")}
              value={result.value.runtime.active_tasks.toString()}
              tone="info"
            />
            <OverviewMetric
              badge={
                result.value.runtime.failed_jobs > 0
                  ? t("overview.metric.failedJobs.attention")
                  : t("overview.metric.failedJobs.clear")
              }
              label={t("overview.metric.failedJobs.label")}
              value={result.value.runtime.failed_jobs.toString()}
              tone={result.value.runtime.failed_jobs > 0 ? "warning" : "success"}
            />
            <OverviewMetric
              badge={t("overview.metric.configuredLibraries.badge")}
              label={t("overview.metric.configuredLibraries.label")}
              value={result.value.startup.configured_libraries.toString()}
              tone="neutral"
            />
            <OverviewMetric
              badge={t("overview.metric.recoveredJobs.badge")}
              label={t("overview.metric.recoveredJobs.label")}
              value={result.value.startup.recovered_jobs.toString()}
              tone="neutral"
            />
          </div>

          <DataPanel
            description={t("overview.storage.description", {
              ready: result.value.storage.ready_backends,
              degraded: result.value.storage.degraded_backends,
              unavailable: result.value.storage.unavailable_backends,
            })}
            title={t("overview.metric.storage.label")}
          >
            <OverviewTable table={storageTable} />
          </DataPanel>

          <DataPanel
            description={t("overview.metadata.description", {
              available: result.value.metadata.available_providers,
              disabled: result.value.metadata.disabled_providers,
              unavailable: result.value.metadata.unavailable_providers,
            })}
            title={t("overview.metadata.title")}
          >
            <OverviewTable table={metadataTable} />
          </DataPanel>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadOverview(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<OverviewResult> {
  if (!dataSource.loadOverview) {
    return {
      value: mockOverview,
      source: "mock",
      error: missingDataSourceMessage,
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

function storageBadge(overview: AdminOverviewResponse, t: Translate) {
  if (overview.storage.unavailable_backends > 0) {
    return t("overview.metric.storage.unavailable");
  }

  if (overview.storage.degraded_backends > 0) {
    return t("overview.metric.storage.degraded");
  }

  return t("overview.metric.storage.ready");
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

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createStorageColumns(t: Translate): Array<ColumnDef<StorageBackend>> {
  return [
    {
      accessorKey: "library_name",
      header: t("overview.column.mediaLibrary"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.library_name}</strong>
          <span>{row.original.library_id}</span>
        </div>
      ),
    },
    {
      accessorKey: "backend_kind",
      header: t("overview.column.backend"),
    },
    {
      accessorKey: "status",
      header: t("overview.column.status"),
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
  ];
}

function createMetadataColumns(t: Translate): Array<ColumnDef<MetadataProvider>> {
  return [
    {
      accessorKey: "provider",
      header: t("overview.column.provider"),
      cell: ({ row }) => row.original.provider.toUpperCase(),
    },
    {
      accessorKey: "status",
      header: t("overview.column.status"),
      cell: ({ row }) => <StatusBadge status={row.original.status} />,
    },
  ];
}
