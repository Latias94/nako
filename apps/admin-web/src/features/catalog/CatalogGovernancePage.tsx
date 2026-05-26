import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "@tanstack/react-router";
import { ChevronRight, RefreshCw, Search, X } from "lucide-react";
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminCatalogGovernanceItem,
  AdminCatalogGovernanceItemListResponse,
  AdminCatalogGovernanceItemsQuery,
} from "../../adminApi/types";
import { mockCatalogGovernance } from "../../adminApi/mockData";
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

export type CatalogGovernanceSearch = {
  library_id?: string;
  max_confidence_milli?: number;
  limit: number;
  offset: number;
};

export type CatalogGovernancePageProps = {
  dataSource: AdminDataSource;
  search: CatalogGovernanceSearch;
  onSearchChange(next: Partial<CatalogGovernanceSearch>): void;
};

type CatalogGovernanceResult = {
  value: AdminCatalogGovernanceItemListResponse;
  source: DataSourceMode;
  error?: string;
};

export function CatalogGovernancePage({
  dataSource,
  search,
  onSearchChange,
}: CatalogGovernancePageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-catalog-governance", search, locale],
    queryFn: () =>
      loadCatalogGovernance(
        dataSource,
        search,
        t("catalogGovernance.detail.routeUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: mockCatalogGovernance,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.library_id, search.max_confidence_milli].filter((value) => value !== undefined).length,
    [search.library_id, search.max_confidence_milli],
  );
  const columns = createColumns(t);
  const table = useReactTable({
    data: result.value.items,
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
          {t("catalogGovernance.refresh")}
        </Button>
      }
      description={t("catalogGovernance.description")}
      kicker={t("catalogGovernance.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("catalogGovernance.title")}
      titleId="catalog-governance-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("catalogGovernance.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      <FilterBar label={t("catalogGovernance.filters")}>
        <FilterField label={t("catalogGovernance.filter.library")}>
          <input
            aria-label={t("catalogGovernance.filter.libraryAria")}
            placeholder={t("catalogGovernance.filter.libraryPlaceholder")}
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("catalogGovernance.filter.confidence")}>
          <input
            aria-label={t("catalogGovernance.filter.confidenceAria")}
            max={1000}
            min={0}
            placeholder={t("catalogGovernance.filter.confidencePlaceholder")}
            type="number"
            value={search.max_confidence_milli ?? ""}
            onChange={(event) =>
              onSearchChange({
                max_confidence_milli: numberInput(event.target.value),
                offset: 0,
              })
            }
          />
        </FilterField>
        <FilterField label={t("catalogGovernance.filter.limit")}>
          <input
            aria-label={t("catalogGovernance.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("catalogGovernance.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            aria-label={t("catalogGovernance.clearAria")}
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                library_id: undefined,
                max_confidence_milli: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("catalogGovernance.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("catalogGovernance.queue.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("catalogGovernance.queue.redacted")}
          </div>
        }
        title={t("catalogGovernance.queue.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("catalogGovernance.loading")} /> : null}

        {!query.isLoading && result.value.items.length === 0 ? (
          <EmptyRouteState>{t("catalogGovernance.empty")}</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.items.length > 0 ? (
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

async function loadCatalogGovernance(
  dataSource: AdminDataSource,
  search: CatalogGovernanceSearch,
  missingDataSourceMessage: string,
): Promise<CatalogGovernanceResult> {
  if (!dataSource.loadCatalogGovernance) {
    return {
      value: mockCatalogGovernance,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadCatalogGovernance(toAdminCatalogGovernanceItemsQuery(search));
}

function toAdminCatalogGovernanceItemsQuery(
  search: CatalogGovernanceSearch,
): AdminCatalogGovernanceItemsQuery {
  return {
    library_id: search.library_id,
    max_confidence_milli: search.max_confidence_milli,
    limit: search.limit,
    offset: search.offset,
  };
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<AdminCatalogGovernanceItem>> {
  return [
    {
      accessorKey: "title",
      header: t("catalogGovernance.column.mediaItem"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.title}</strong>
          <span>{row.original.item_id}</span>
        </div>
      ),
    },
    {
      accessorKey: "kind",
      header: t("catalogGovernance.column.kind"),
      cell: ({ row }) => (
        <Badge tone={row.original.kind === "unknown" ? "warning" : "neutral"}>
          {row.original.kind}
        </Badge>
      ),
    },
    {
      accessorKey: "library_id",
      header: t("catalogGovernance.column.mediaLibrary"),
    },
    {
      id: "local_inference",
      header: t("catalogGovernance.column.localInference"),
      cell: ({ row }) => <LocalInferenceBadge item={row.original} t={t} />,
    },
    {
      accessorKey: "issues",
      header: t("catalogGovernance.column.issues"),
      cell: ({ row }) => <IssueList issues={row.original.issues} t={t} />,
    },
    {
      accessorKey: "source_count",
      header: t("catalogGovernance.column.sources"),
    },
    {
      accessorKey: "provider_mapping_count",
      header: t("catalogGovernance.column.mappings"),
      cell: ({ row }) =>
        t("catalogGovernance.relations.accepted", {
          accepted: row.original.accepted_provider_mapping_count,
          total: row.original.provider_mapping_count,
        }),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <Link
          aria-label={t("catalogGovernance.reviewLink", { itemId: row.original.item_id })}
          className="routeTextLink"
          params={{ itemId: row.original.item_id }}
          search={{ decision: "accept" }}
          to="/catalog/governance/$itemId"
        >
          {t("catalogGovernance.review")}
          <ChevronRight size={15} />
        </Link>
      ),
    },
  ];
}

function LocalInferenceBadge({ item, t }: { item: AdminCatalogGovernanceItem; t: Translate }) {
  const confidence = item.local_inference?.confidence_milli;

  if (confidence === undefined || confidence === null) {
    return <Badge tone="neutral">{t("catalogGovernance.inference.none")}</Badge>;
  }

  if (confidence < 500) {
    return (
      <Badge tone="warning">
        {t("catalogGovernance.inference.confidence", { confidence })}
      </Badge>
    );
  }

  return (
    <Badge tone="info">
      {t("catalogGovernance.inference.confidence", { confidence })}
    </Badge>
  );
}

function IssueList({ issues, t }: { issues: string[]; t: Translate }) {
  if (issues.length === 0) {
    return <Badge tone="success">{t("catalogGovernance.issues.none")}</Badge>;
  }

  return (
    <div className="issueBadgeList">
      {issues.map((issue) => (
        <Badge key={issue} tone="warning">
          {issue}
        </Badge>
      ))}
    </div>
  );
}

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : undefined;
}
