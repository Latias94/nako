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
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

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

export function GeneratedArtifactsPage({
  dataSource,
  search,
  onSearchChange,
}: GeneratedArtifactsPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-generated-artifacts", search, locale],
    queryFn: () =>
      loadGeneratedArtifacts(dataSource, search, t("generatedArtifacts.dataSourceUnavailable")),
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
          {t("generatedArtifacts.refresh")}
        </Button>
      }
      description={t("generatedArtifacts.description")}
      kicker={t("generatedArtifacts.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("generatedArtifacts.title")}
      titleId="generated-artifacts-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("generatedArtifacts.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("generatedArtifacts.pagination")}>
        <FilterField label={t("generatedArtifacts.limit")}>
          <input
            aria-label={t("generatedArtifacts.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("generatedArtifacts.offset")}>
          <input
            aria-label={t("generatedArtifacts.offsetAria")}
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeProposalCount > 0 ? "info" : "neutral"}>
            {t("generatedArtifacts.actionableCount", { count: activeProposalCount })}
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
            {t("generatedArtifacts.reset")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("generatedArtifacts.queue.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("generatedArtifacts.queue.urlPagination")}
          </div>
        }
        title={t("generatedArtifacts.queue.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("generatedArtifacts.loading")} /> : null}

        {!query.isLoading && result.value.proposals.length === 0 ? (
          <EmptyRouteState>{t("generatedArtifacts.empty")}</EmptyRouteState>
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
  unavailableMessage: string,
): Promise<GeneratedArtifactsResult> {
  if (!dataSource.loadGeneratedArtifacts) {
    return {
      value: mockGeneratedArtifactProposals,
      source: "mock",
      error: unavailableMessage,
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

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function createColumns(t: Translate): Array<ColumnDef<AdminGeneratedArtifactProposal>> {
  return [
    {
      accessorKey: "capability",
      header: t("generatedArtifacts.column.proposal"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.capability}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "status",
      header: t("generatedArtifacts.column.status"),
      cell: ({ row }) => <Badge tone={proposalStatusTone(row.original.status)}>{row.original.status}</Badge>,
    },
    {
      accessorKey: "readiness.status",
      header: t("generatedArtifacts.column.readiness"),
      cell: ({ row }) => (
        <Badge tone={readinessTone(row.original.readiness.status, row.original.readiness.actionable)}>
          {row.original.readiness.status}
        </Badge>
      ),
    },
    {
      accessorKey: "target.kind",
      header: t("generatedArtifacts.column.target"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.target.kind}</strong>
          <span>{targetLabel(row.original, t)}</span>
        </div>
      ),
    },
    {
      accessorKey: "provenance.provider_name",
      header: t("generatedArtifacts.column.provider"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.provenance.provider_name ?? t("generatedArtifacts.unknownProvider")}</strong>
          <span>{t("generatedArtifacts.attempts", { count: row.original.provenance.attempt_count ?? 0 })}</span>
        </div>
      ),
    },
    {
      accessorKey: "payload.shape",
      header: t("generatedArtifacts.column.payload"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.payload.shape}</strong>
          <span>{formatBytes(row.original.payload.payload_bytes)}</span>
        </div>
      ),
    },
    {
      accessorKey: "payload.confidence_milli",
      header: t("generatedArtifacts.column.confidence"),
      cell: ({ row }) => confidenceLabel(row.original.payload.confidence_milli, t),
    },
    {
      id: "fingerprints",
      header: t("generatedArtifacts.column.fingerprints"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{shortFingerprint(row.original.payload.payload_fingerprint, t)}</strong>
          <span>{shortFingerprint(row.original.provenance.prompt_fingerprint, t)}</span>
        </div>
      ),
    },
    {
      accessorKey: "updated_at",
      header: t("generatedArtifacts.column.updated"),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <Link
          aria-label={t("generatedArtifacts.reviewAria", { artifactId: row.original.id })}
          className="routeTextLink"
          params={{ artifactId: row.original.id }}
          search={{ decision: "accept" }}
          to="/automation/generated-artifacts/$artifactId/review"
        >
          {t("generatedArtifacts.review")}
          <ChevronRight size={15} />
        </Link>
      ),
    },
  ];
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

function targetLabel(proposal: AdminGeneratedArtifactProposal, t: Translate) {
  return [proposal.target.library_id, proposal.target.item_id, proposal.target.source_id]
    .filter(Boolean)
    .join(" / ") || t("generatedArtifacts.noTargetId");
}

function confidenceLabel(value: number | null, t: Translate) {
  return value === null ? t("generatedArtifacts.unknown") : `${value} / 1000`;
}

function shortFingerprint(value: string | null, t: Translate) {
  if (!value) {
    return t("generatedArtifacts.noFingerprint");
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
