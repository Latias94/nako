import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "@tanstack/react-router";
import { ChevronRight, ListChecks, RefreshCw, Search, X } from "lucide-react";
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  CatalogBrowseItemSummary,
  CatalogBrowseQuery,
  CatalogBrowseSummary,
} from "../../adminApi/types";
import { mockCatalogBrowse } from "../../adminApi/mockData";
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

export type CatalogSearch = {
  q?: string;
  facet?: string;
  limit: number;
  offset: number;
};

export type CatalogBrowsePageProps = {
  dataSource: AdminDataSource;
  search: CatalogSearch;
  onSearchChange(next: Partial<CatalogSearch>): void;
};

type CatalogBrowseResult = {
  value: CatalogBrowseSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function CatalogBrowsePage({
  dataSource,
  search,
  onSearchChange,
}: CatalogBrowsePageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-catalog-browse", search, locale],
    queryFn: () => loadCatalog(dataSource, search, t("catalogBrowse.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockCatalogBrowse,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.q, search.facet].filter((value) => value !== undefined).length,
    [search.facet, search.q],
  );
  const hasPaginationDelta = search.limit !== 20 || search.offset !== 0;
  const table = useReactTable({
    data: result.value.items,
    columns: createColumns(t),
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            className="routeTextLink"
            search={{ library_id: undefined, max_confidence_milli: undefined, limit: 20, offset: 0 }}
            to="/catalog/governance"
          >
            <ListChecks size={16} />
            {t("catalogBrowse.governanceQueue")}
          </Link>
          <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("catalogBrowse.refresh")}
          </Button>
        </div>
      }
      description={t("catalogBrowse.description")}
      kicker={t("catalogBrowse.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("catalogBrowse.title")}
      titleId="catalog-browse-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("catalogBrowse.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("catalogBrowse.filters")}>
        <FilterField label={t("catalogBrowse.filter.search")}>
          <input
            aria-label={t("catalogBrowse.filter.searchAria")}
            placeholder="title or keyword"
            value={search.q ?? ""}
            onChange={(event) => onSearchChange({ q: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("catalogBrowse.filter.facet")}>
          <input
            aria-label={t("catalogBrowse.filter.facetAria")}
            placeholder="kind:movie"
            value={search.facet ?? ""}
            onChange={(event) => onSearchChange({ facet: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("catalogBrowse.filter.limit")}>
          <input
            aria-label={t("catalogBrowse.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("catalogBrowse.filter.offset")}>
          <input
            aria-label={t("catalogBrowse.filter.offsetAria")}
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={result.value.mode === "search" ? "info" : "neutral"}>
            {result.value.mode}
          </Badge>
          <Button
            disabled={activeFilterCount === 0 && !hasPaginationDelta}
            onClick={() =>
              onSearchChange({
                q: undefined,
                facet: undefined,
                limit: 20,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("catalogBrowse.reset")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("catalogBrowse.items.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("catalogBrowse.items.urlParams")}
          </div>
        }
        title={t("catalogBrowse.items.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("catalogBrowse.loading")} /> : null}

        {!query.isLoading && result.value.items.length === 0 ? (
          <EmptyRouteState>{t("catalogBrowse.empty")}</EmptyRouteState>
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

async function loadCatalog(
  dataSource: AdminDataSource,
  search: CatalogSearch,
  unavailableMessage: string,
): Promise<CatalogBrowseResult> {
  if (!dataSource.loadCatalog) {
    return {
      value: mockCatalogBrowse,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadCatalog(toCatalogBrowseQuery(search));
}

function toCatalogBrowseQuery(search: CatalogSearch): CatalogBrowseQuery {
  return {
    q: search.q,
    facet: search.facet,
    limit: search.limit,
    offset: search.offset,
  };
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<CatalogBrowseItemSummary>> {
  return [
    {
      accessorKey: "title",
      header: t("catalogBrowse.column.mediaItem"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.title}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "kind",
      header: t("catalogBrowse.column.kind"),
      cell: ({ row }) => <Badge tone={kindTone(row.original.kind)}>{row.original.kind}</Badge>,
    },
    {
      id: "canonical_metadata",
      header: t("catalogBrowse.column.canonicalMetadata"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{releaseRuntimeLabel(row.original, t)}</strong>
          <span>
            {t("catalogBrowse.canonicalCounts", {
              genres: row.original.genreCount,
              tags: row.original.tagCount,
            })}
          </span>
        </div>
      ),
    },
    {
      id: "relations",
      header: t("catalogBrowse.column.relations"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{t("catalogBrowse.creditCount", { count: row.original.creditCount })}</strong>
          <span>
            {t("catalogBrowse.relationCounts", {
              collections: row.original.collectionCount,
              studios: row.original.studioCount,
            })}
          </span>
        </div>
      ),
    },
    {
      id: "source_image_readiness",
      header: t("catalogBrowse.column.sourceImage"),
      cell: ({ row }) => (
        <div className="issueBadgeList">
          <Badge tone={row.original.sourceCount === null ? "neutral" : "info"}>
            {countLabel(row.original.sourceCount, t("catalogBrowse.sources"), t)}
          </Badge>
          <Badge tone={row.original.imageCount === null ? "neutral" : "info"}>
            {countLabel(row.original.imageCount, t("catalogBrowse.images"), t)}
          </Badge>
        </div>
      ),
    },
    {
      accessorKey: "score",
      header: t("catalogBrowse.column.search"),
      cell: ({ row }) => searchScoreLabel(row.original.score, t),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <Link
          aria-label={t("catalogBrowse.inspectAria", { title: row.original.title })}
          className="routeTextLink"
          params={{ itemId: row.original.id }}
          to="/items/$itemId"
        >
          {t("catalogBrowse.inspect")}
          <ChevronRight size={15} />
        </Link>
      ),
    },
  ];
}

function kindTone(kind: string): BadgeTone {
  if (kind === "unknown") {
    return "warning";
  }

  if (kind === "collection") {
    return "info";
  }

  return "neutral";
}

function releaseRuntimeLabel(item: CatalogBrowseItemSummary, t: Translate) {
  const release = item.releaseDate ?? t("catalogBrowse.noReleaseDate");
  const runtime =
    item.runtimeMinutes === null
      ? t("catalogBrowse.runtimeUnknown")
      : t("catalogBrowse.runtimeMinutes", { minutes: item.runtimeMinutes });

  return `${release} / ${runtime}`;
}

function countLabel(value: number | null, label: string, t: Translate) {
  return value === null
    ? t("catalogBrowse.detailRoute", { label })
    : t("catalogBrowse.countLabel", { count: value, label });
}

function searchScoreLabel(score: number | null, t: Translate) {
  return score === null ? t("catalogBrowse.scoreBrowse") : score.toFixed(2);
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
