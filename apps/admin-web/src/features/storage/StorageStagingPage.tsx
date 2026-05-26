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
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

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

export function StorageStagingPage({
  dataSource,
  search,
  onSearchChange,
}: StorageStagingPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-storage-staging", search, locale],
    queryFn: () => loadStorageStaging(dataSource, search, t("storage.dataSourceUnavailable")),
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
          {t("storage.refresh")}
        </Button>
      }
      description={t("storage.description")}
      kicker={t("storage.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("storage.title")}
      titleId="storage-staging-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("storage.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("storage.filters")}>
        <FilterField label={t("storage.filter.purpose")}>
          <input
            aria-label={t("storage.filter.purposeAria")}
            placeholder="ffmpeg_input"
            value={search.purpose ?? ""}
            onChange={(event) => onSearchChange({ purpose: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("storage.filter.state")}>
          <select
            aria-label={t("storage.filter.stateAria")}
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("storage.filter.anyState")}</option>
            <option value="ready">{t("storage.state.ready")}</option>
            <option value="failed">{t("storage.state.failed")}</option>
            <option value="stale">{t("storage.state.stale")}</option>
            <option value="expired">{t("storage.state.expired")}</option>
          </select>
        </FilterField>
        <FilterField label={t("storage.filter.limit")}>
          <input
            aria-label={t("storage.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("storage.filter.active", { count: activeFilterCount })}
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
            {t("storage.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("storage.records.description", {
          returned: result.value.page.returned,
          used: formatBytes(result.value.summary.used_manifest_bytes, t),
          max: formatBytes(result.value.summary.configured_max_bytes, t),
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("storage.records.urlFilters")}
          </div>
        }
        title={t("storage.records.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("storage.loading")} /> : null}

        {!query.isLoading && result.value.records.length === 0 ? (
          <EmptyRouteState>{t("storage.empty")}</EmptyRouteState>
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

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<StorageRecord>> {
  return [
    {
      accessorKey: "id",
      header: t("storage.column.record"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.id}</strong>
          <span>{row.original.purpose}</span>
        </div>
      ),
    },
    {
      accessorKey: "state",
      header: t("storage.column.state"),
      cell: ({ row }) => <StorageStateBadge record={row.original} />,
    },
    {
      accessorKey: "source_scheme",
      header: t("storage.column.sourceScheme"),
    },
    {
      accessorKey: "size_bytes",
      header: t("storage.column.size"),
      cell: ({ row }) => formatBytes(row.original.size_bytes, t),
    },
    {
      accessorKey: "active_leases",
      header: t("storage.column.leases"),
    },
    {
      id: "validation",
      header: t("storage.column.validation"),
      cell: ({ row }) =>
        row.original.has_validation_error
          ? t("storage.validation.failed")
          : t("storage.validation.clean"),
    },
    {
      accessorKey: "expires_at_ms",
      header: t("storage.column.expires"),
      cell: ({ row }) => timestampLabel(row.original.expires_at_ms, t),
    },
  ];
}

async function loadStorageStaging(
  dataSource: AdminDataSource,
  search: StorageStagingSearch,
  unavailableMessage: string,
): Promise<StorageStagingResult> {
  if (!dataSource.loadStorageStaging) {
    return {
      value: mockStorageStaging,
      source: "mock",
      error: unavailableMessage,
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

function timestampLabel(value: number | null, t: Translate) {
  if (value === null) {
    return t("storage.none");
  }

  return new Date(value).toISOString();
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("storage.unknown");
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
