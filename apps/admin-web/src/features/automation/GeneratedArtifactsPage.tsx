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
  AdminGeneratedArtifactProposal,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
} from "../../adminApi/types";
import { mockGeneratedArtifactProposals } from "../../adminApi/mockData";
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

export type GeneratedArtifactsSearch = {
  limit: number;
  offset: number;
};

export type GeneratedArtifactsPageProps = {
  dataSource: AdminDataSource;
  search: GeneratedArtifactsSearch;
  onSearchChange(next: Partial<GeneratedArtifactsSearch>): void;
};

type GeneratedArtifactsResult = {
  value: AdminGeneratedArtifactProposalListResponse;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

const columns: Array<ColumnDef<AdminGeneratedArtifactProposal>> = [
  {
    accessorKey: "capability",
    header: "Proposal",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.capability}</strong>
        <span>{row.original.id}</span>
      </div>
    ),
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <Badge tone={proposalStatusTone(row.original.status)}>{row.original.status}</Badge>,
  },
  {
    accessorKey: "readiness.status",
    header: "Readiness",
    cell: ({ row }) => (
      <Badge tone={readinessTone(row.original.readiness.status, row.original.readiness.actionable)}>
        {row.original.readiness.status}
      </Badge>
    ),
  },
  {
    accessorKey: "target.kind",
    header: "Target",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.target.kind}</strong>
        <span>{targetLabel(row.original)}</span>
      </div>
    ),
  },
  {
    accessorKey: "provenance.provider_name",
    header: "Provider",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.provenance.provider_name ?? "unknown provider"}</strong>
        <span>{row.original.provenance.attempt_count ?? 0} attempts</span>
      </div>
    ),
  },
  {
    accessorKey: "payload.shape",
    header: "Payload",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{row.original.payload.shape}</strong>
        <span>{formatBytes(row.original.payload.payload_bytes)}</span>
      </div>
    ),
  },
  {
    accessorKey: "payload.confidence_milli",
    header: "Confidence",
    cell: ({ row }) => confidenceLabel(row.original.payload.confidence_milli),
  },
  {
    id: "fingerprints",
    header: "Fingerprints",
    cell: ({ row }) => (
      <div className="routePrimaryCell">
        <strong>{shortFingerprint(row.original.payload.payload_fingerprint)}</strong>
        <span>{shortFingerprint(row.original.provenance.prompt_fingerprint)}</span>
      </div>
    ),
  },
  {
    accessorKey: "updated_at",
    header: "Updated",
  },
];

export function GeneratedArtifactsPage({
  dataSource,
  search,
  onSearchChange,
}: GeneratedArtifactsPageProps) {
  const query = useQuery({
    queryKey: ["admin-generated-artifacts", search],
    queryFn: () => loadGeneratedArtifacts(dataSource, search),
  });
  const result = query.data ?? {
    value: mockGeneratedArtifactProposals,
    source: "mock" as const,
  };
  const hasPaginationDelta = search.limit !== 20 || search.offset !== 0;
  const activeProposalCount = useMemo(
    () => result.value.proposals.filter((proposal) => proposal.readiness.actionable).length,
    [result.value.proposals],
  );
  const table = useReactTable({
    data: result.value.proposals,
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
      description="AI-assisted proposals reduced to readiness, payload shape, confidence, and fingerprints. Review mutations stay out of this route."
      kicker="Automation"
      status={<SourceLabel source={result.source} />}
      title="Generated Artifacts"
      titleId="generated-artifacts-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      <FilterBar label="Generated artifact pagination">
        <FilterField label="Limit">
          <input
            aria-label="Generated artifacts page limit"
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label="Offset">
          <input
            aria-label="Generated artifacts page offset"
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeProposalCount > 0 ? "info" : "neutral"}>
            {activeProposalCount} actionable
          </Badge>
          <Button
            disabled={!hasPaginationDelta}
            onClick={() =>
              onSearchChange({
                limit: 20,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            Reset
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={`${result.value.page.returned} returned, offset ${result.value.page.offset}, limit ${result.value.page.limit}`}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            URL pagination is authoritative
          </div>
        }
        title="Proposal queue"
      >
        {query.isLoading ? <RowsSkeleton label="Loading Generated Artifacts proposals" /> : null}

        {!query.isLoading && result.value.proposals.length === 0 ? (
          <EmptyRouteState>No Generated Artifact proposals match the current page.</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.proposals.length > 0 ? (
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

async function loadGeneratedArtifacts(
  dataSource: AdminDataSource,
  search: GeneratedArtifactsSearch,
): Promise<GeneratedArtifactsResult> {
  if (!dataSource.loadGeneratedArtifacts) {
    return {
      value: mockGeneratedArtifactProposals,
      source: "mock",
      error: "Generated Artifacts route data source is unavailable",
    };
  }

  return dataSource.loadGeneratedArtifacts(toAdminGeneratedArtifactProposalsQuery(search));
}

function toAdminGeneratedArtifactProposalsQuery(
  search: GeneratedArtifactsSearch,
): AdminGeneratedArtifactProposalsQuery {
  return {
    limit: search.limit,
    offset: search.offset,
  };
}

function proposalStatusTone(status: string): BadgeTone {
  if (status === "accepted") {
    return "success";
  }

  if (status === "rejected") {
    return "danger";
  }

  return "warning";
}

function readinessTone(status: string, actionable: boolean): BadgeTone {
  if (status === "ready" && actionable) {
    return "success";
  }

  if (status === "stale") {
    return "warning";
  }

  return "danger";
}

function targetLabel(proposal: AdminGeneratedArtifactProposal) {
  return [proposal.target.library_id, proposal.target.item_id, proposal.target.source_id]
    .filter(Boolean)
    .join(" / ") || "no target id";
}

function confidenceLabel(value: number | null) {
  return value === null ? "unknown" : `${value} / 1000`;
}

function shortFingerprint(value: string | null) {
  if (!value) {
    return "no fingerprint";
  }

  return value.length > 24 ? `${value.slice(0, 21)}...` : value;
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

function formatBytes(value: number) {
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
