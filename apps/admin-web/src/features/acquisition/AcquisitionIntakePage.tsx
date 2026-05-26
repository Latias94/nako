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
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

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

export function AcquisitionIntakePage({
  dataSource,
  search,
  onSearchChange,
}: AcquisitionIntakePageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-acquisition-intake", search, locale],
    queryFn: () =>
      loadAcquisitionIntake(dataSource, search, t("acquisition.dataSourceUnavailable")),
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
          {t("acquisition.refresh")}
        </Button>
      }
      description={t("acquisition.description")}
      kicker={t("acquisition.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("acquisition.title")}
      titleId="acquisition-intake-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("acquisition.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("acquisition.filters")}>
        <FilterField label={t("acquisition.filter.library")}>
          <input
            aria-label={t("acquisition.filter.libraryAria")}
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("acquisition.filter.state")}>
          <select
            aria-label={t("acquisition.filter.stateAria")}
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("acquisition.filter.anyState")}</option>
            <option value="ready">{t("acquisition.state.ready")}</option>
            <option value="blocked">{t("acquisition.state.blocked")}</option>
            <option value="incomplete">{t("acquisition.state.incomplete")}</option>
            <option value="unsupported">{t("acquisition.state.unsupported")}</option>
          </select>
        </FilterField>
        <FilterField label={t("acquisition.filter.sourceKind")}>
          <input
            aria-label={t("acquisition.filter.sourceKindAria")}
            placeholder="watch_folder"
            value={search.source_kind ?? ""}
            onChange={(event) => onSearchChange({ source_kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("acquisition.filter.managedImport")}>
          <input
            aria-label={t("acquisition.filter.managedImportAria")}
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
        <FilterField label={t("acquisition.filter.limit")}>
          <input
            aria-label={t("acquisition.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("acquisition.filter.offset")}>
          <input
            aria-label={t("acquisition.filter.offsetAria")}
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("acquisition.filter.active", { count: activeFilterCount })}
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
            {t("acquisition.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("acquisition.candidates.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("acquisition.candidates.urlFilters")}
          </div>
        }
        title={t("acquisition.candidates.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("acquisition.loading")} /> : null}

        {!query.isLoading && result.value.candidates.length === 0 ? (
          <EmptyRouteState>{t("acquisition.empty")}</EmptyRouteState>
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
  unavailableMessage: string,
): Promise<AcquisitionIntakeResult> {
  if (!dataSource.loadAcquisitionIntake) {
    return {
      value: mockAcquisitionIntakeCandidates,
      source: "mock",
      error: unavailableMessage,
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

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<AdminAcquisitionIntakeCandidateDiagnostic>> {
  return [
    {
      accessorKey: "id",
      header: t("acquisition.column.candidate"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.id}</strong>
          <span>{row.original.target_library_id}</span>
        </div>
      ),
    },
    {
      accessorKey: "state",
      header: t("acquisition.column.state"),
      cell: ({ row }) => <Badge tone={candidateStateTone(row.original.state)}>{row.original.state}</Badge>,
    },
    {
      accessorKey: "source_kind",
      header: t("acquisition.column.source"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.source_kind}</strong>
          <span>{row.original.source_scheme ?? t("acquisition.unknownScheme")}</span>
        </div>
      ),
    },
    {
      accessorKey: "size_bytes",
      header: t("acquisition.column.size"),
      cell: ({ row }) => formatBytes(row.original.size_bytes, t),
    },
    {
      id: "diagnostics",
      header: t("acquisition.column.diagnostics"),
      cell: ({ row }) => (
        <Badge tone={row.original.has_diagnostics ? "info" : "neutral"}>
          {row.original.has_diagnostics ? t("acquisition.diagnostics.available") : t("acquisition.none")}
        </Badge>
      ),
    },
    {
      accessorKey: "managed_import_artifact_id",
      header: t("acquisition.column.managedImport"),
      cell: ({ row }) => row.original.managed_import_artifact_id ?? t("acquisition.notLinked"),
    },
    {
      accessorKey: "first_seen_at_ms",
      header: t("acquisition.column.firstSeen"),
      cell: ({ row }) => timestampLabel(row.original.first_seen_at_ms),
    },
    {
      accessorKey: "updated_at_ms",
      header: t("acquisition.column.updated"),
      cell: ({ row }) => timestampLabel(row.original.updated_at_ms),
    },
  ];
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

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("acquisition.unknown");
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
