import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Fingerprint, RefreshCw, Search, X } from "lucide-react";
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

const SOURCE_FINGERPRINT_HASH_JOB_KIND = "source_fingerprint_hash";
const SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS = "disk.scan.source_fingerprint_hash";

export function JobsPage({ dataSource, search, onSearchChange }: JobsPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-jobs", search, locale],
    queryFn: () => loadJobs(dataSource, search, t("jobs.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockJobs,
    source: "mock" as const,
  };
  const columns = createColumns(t);

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
  const sourceHashFilterActive =
    search.kind === SOURCE_FINGERPRINT_HASH_JOB_KIND &&
    search.resource_class === SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS;

  return (
    <RoutePage
      actions={
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("jobs.refresh")}
        </Button>
      }
      description={t("jobs.description")}
      kicker={t("jobs.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("jobs.title")}
      titleId="jobs-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("jobs.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("jobs.filters")}>
        <FilterField label={t("jobs.filter.status")}>
          <select
            aria-label={t("jobs.filter.statusAria")}
            value={search.status ?? ""}
            onChange={(event) => onSearchChange({ status: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("jobs.filter.anyStatus")}</option>
            <option value="queued">{t("jobs.status.queued")}</option>
            <option value="running">{t("jobs.status.running")}</option>
            <option value="failed">{t("jobs.status.failed")}</option>
            <option value="succeeded">{t("jobs.status.succeeded")}</option>
            <option value="cancelled">{t("jobs.status.cancelled")}</option>
          </select>
        </FilterField>
        <FilterField label={t("jobs.filter.kind")}>
          <input
            aria-label={t("jobs.filter.kindAria")}
            placeholder="metadata_refresh"
            value={search.kind ?? ""}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("jobs.filter.resource")}>
          <input
            aria-label={t("jobs.filter.resourceAria")}
            placeholder="library"
            value={search.resource_class ?? ""}
            onChange={(event) =>
              onSearchChange({ resource_class: event.target.value || undefined, offset: 0 })
            }
          />
        </FilterField>
        <FilterField label={t("jobs.filter.library")}>
          <input
            aria-label={t("jobs.filter.libraryAria")}
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("jobs.filter.source")}>
          <input
            aria-label={t("jobs.filter.sourceAria")}
            placeholder="source-id"
            value={search.source_id ?? ""}
            onChange={(event) => onSearchChange({ source_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Button
            aria-pressed={sourceHashFilterActive}
            onClick={() =>
              onSearchChange({
                kind: SOURCE_FINGERPRINT_HASH_JOB_KIND,
                resource_class: SOURCE_FINGERPRINT_HASH_RESOURCE_CLASS,
                offset: 0,
              })
            }
            variant={sourceHashFilterActive ? "default" : "outline"}
          >
            <Fingerprint size={16} />
            {t("jobs.filter.sourceHash")}
          </Button>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("jobs.filter.active", { count: activeFilterCount })}
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
            {t("jobs.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("jobs.queue.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("jobs.queue.urlFilters")}
          </div>
        }
        title={t("jobs.queue.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("jobs.loading")} /> : null}

        {!query.isLoading && result.value.jobs.length === 0 ? (
          <EmptyRouteState>{t("jobs.empty")}</EmptyRouteState>
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
      </DataPanel>
    </RoutePage>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<AdminJobListItem>> {
  return [
    {
      accessorKey: "kind",
      header: t("jobs.column.kind"),
      cell: ({ row }) => (
        <div className="jobsPrimaryCell">
          <strong>{row.original.kind}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: t("jobs.column.status"),
      cell: ({ row }) => <JobStatusBadge status={row.original.status} hasError={row.original.has_error} />,
    },
    {
      accessorKey: "resource_class",
      header: t("jobs.column.resource"),
    },
    {
      accessorKey: "library_id",
      header: t("jobs.column.mediaLibrary"),
      cell: ({ row }) => row.original.library_id ?? t("jobs.none"),
    },
    {
      accessorKey: "source_id",
      header: t("jobs.column.mediaSource"),
      cell: ({ row }) => row.original.source_id ?? t("jobs.none"),
    },
    {
      accessorKey: "queued_at",
      header: t("jobs.column.queued"),
    },
  ];
}

async function loadJobs(
  dataSource: AdminDataSource,
  search: JobsSearch,
  unavailableMessage: string,
): Promise<JobsResult> {
  if (!dataSource.loadJobs) {
    return {
      value: mockJobs,
      source: "mock",
      error: unavailableMessage,
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
