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

const columns: Array<ColumnDef<AdminCatalogGovernanceItem>> = [
  {
    accessorKey: "title",
    header: "Media Item",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.title}</strong>
        <span>{row.original.item_id}</span>
      </div>
    ),
  },
  {
    accessorKey: "kind",
    header: "Kind",
    cell: ({ row }) => <Badge tone={row.original.kind === "unknown" ? "warning" : "neutral"}>{row.original.kind}</Badge>,
  },
  {
    accessorKey: "library_id",
    header: "Media Library",
  },
  {
    id: "local_inference",
    header: "Local Inference",
    cell: ({ row }) => <LocalInferenceBadge item={row.original} />,
  },
  {
    accessorKey: "issues",
    header: "Issues",
    cell: ({ row }) => <IssueList issues={row.original.issues} />,
  },
  {
    accessorKey: "source_count",
    header: "Sources",
  },
  {
    accessorKey: "provider_mapping_count",
    header: "Mappings",
    cell: ({ row }) =>
      `${row.original.accepted_provider_mapping_count}/${row.original.provider_mapping_count} accepted`,
  },
];

export function CatalogGovernancePage({
  dataSource,
  search,
  onSearchChange,
}: CatalogGovernancePageProps) {
  const query = useQuery({
    queryKey: ["admin-catalog-governance", search],
    queryFn: () => loadCatalogGovernance(dataSource, search),
  });
  const result = query.data ?? {
    value: mockCatalogGovernance,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.library_id, search.max_confidence_milli].filter((value) => value !== undefined).length,
    [search.library_id, search.max_confidence_milli],
  );
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
          Refresh
        </Button>
      }
      description="Unknown and low-confidence Media Items with route-owned filters and safe fallback."
      kicker="Catalog governance"
      status={<SourceLabel source={result.source} />}
      title="Catalog Governance"
      titleId="catalog-governance-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Catalog governance filters">
        <FilterField label="Library">
          <input
            aria-label="Catalog library filter"
            placeholder="library-id"
            value={search.library_id ?? ""}
            onChange={(event) => onSearchChange({ library_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label="Max confidence">
          <input
            aria-label="Catalog max confidence filter"
            max={1000}
            min={0}
            placeholder="500"
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
        <FilterField label="Limit">
          <input
            aria-label="Catalog page limit"
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {activeFilterCount} filters
          </Badge>
          <Button
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
        title="Governance queue"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Catalog Governance items" /> : null}

        {!query.isLoading && result.value.items.length === 0 ? (
          <EmptyRouteState>No Catalog Governance items match the current filters.</EmptyRouteState>
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
): Promise<CatalogGovernanceResult> {
  if (!dataSource.loadCatalogGovernance) {
    return {
      value: mockCatalogGovernance,
      source: "mock",
      error: "Catalog Governance route data source is unavailable",
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

function LocalInferenceBadge({ item }: { item: AdminCatalogGovernanceItem }) {
  const confidence = item.local_inference?.confidence_milli;

  if (confidence === undefined || confidence === null) {
    return <Badge tone="neutral">No inference</Badge>;
  }

  if (confidence < 500) {
    return <Badge tone="warning">{confidence} confidence</Badge>;
  }

  return <Badge tone="info">{confidence} confidence</Badge>;
}

function IssueList({ issues }: { issues: string[] }) {
  if (issues.length === 0) {
    return <Badge tone="success">none</Badge>;
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
