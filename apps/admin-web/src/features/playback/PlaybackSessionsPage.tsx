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
  AdminPlaybackSessionListItem,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSessionsQuery,
} from "../../adminApi/types";
import { mockPlaybackSessions } from "../../adminApi/mockData";
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

export type PlaybackSessionsSearch = {
  source_id?: string;
  kind?: string;
  state?: string;
  limit: number;
  offset: number;
};

export type PlaybackSessionsPageProps = {
  dataSource: AdminDataSource;
  search: PlaybackSessionsSearch;
  onSearchChange(next: Partial<PlaybackSessionsSearch>): void;
};

type PlaybackSessionsResult = {
  value: AdminPlaybackSessionListResponse;
  source: DataSourceMode;
  error?: string;
};

export function PlaybackSessionsPage({
  dataSource,
  search,
  onSearchChange,
}: PlaybackSessionsPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-playback-sessions", search, locale],
    queryFn: () => loadPlaybackSessions(dataSource, search, t("playback.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockPlaybackSessions,
    source: "mock" as const,
  };
  const activeFilterCount = useMemo(
    () => [search.source_id, search.kind, search.state].filter(Boolean).length,
    [search.kind, search.source_id, search.state],
  );
  const table = useReactTable({
    data: result.value.sessions,
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
          {t("playback.refresh")}
        </Button>
      }
      description={t("playback.description")}
      kicker={t("playback.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("playback.title")}
      titleId="playback-sessions-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("playback.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("playback.filters")}>
        <FilterField label={t("playback.filter.source")}>
          <input
            aria-label={t("playback.filter.sourceAria")}
            placeholder="source-id"
            value={search.source_id ?? ""}
            onChange={(event) => onSearchChange({ source_id: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("playback.filter.kind")}>
          <input
            aria-label={t("playback.filter.kindAria")}
            placeholder="hls_transcode"
            value={search.kind ?? ""}
            onChange={(event) => onSearchChange({ kind: event.target.value || undefined, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("playback.filter.state")}>
          <select
            aria-label={t("playback.filter.stateAria")}
            value={search.state ?? ""}
            onChange={(event) => onSearchChange({ state: event.target.value || undefined, offset: 0 })}
          >
            <option value="">{t("playback.filter.anyState")}</option>
            <option value="starting">{t("playback.state.starting")}</option>
            <option value="running">{t("playback.state.running")}</option>
            <option value="failed">{t("playback.state.failed")}</option>
            <option value="completed">{t("playback.state.completed")}</option>
            <option value="cancelled">{t("playback.state.cancelled")}</option>
          </select>
        </FilterField>
        <FilterField label={t("playback.filter.limit")}>
          <input
            aria-label={t("playback.filter.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("playback.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                source_id: undefined,
                kind: undefined,
                state: undefined,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("playback.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      <DataPanel
        description={t("playback.queue.description", {
          returned: result.value.page.returned,
          offset: result.value.page.offset,
          limit: result.value.page.limit,
        })}
        headerAccessory={
          <div className="searchHint">
            <Search size={15} />
            {t("playback.queue.urlFilters")}
          </div>
        }
        title={t("playback.queue.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("playback.loading")} /> : null}

        {!query.isLoading && result.value.sessions.length === 0 ? (
          <EmptyRouteState>{t("playback.empty")}</EmptyRouteState>
        ) : null}

        {!query.isLoading && result.value.sessions.length > 0 ? (
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

function createColumns(t: Translate): Array<ColumnDef<AdminPlaybackSessionListItem>> {
  return [
    {
      accessorKey: "kind",
      header: t("playback.column.session"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.kind}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "state",
      header: t("playback.column.state"),
      cell: ({ row }) => <SessionStateBadge session={row.original} />,
    },
    {
      accessorKey: "source_id",
      header: t("playback.column.mediaSource"),
    },
    {
      id: "lifecycle",
      header: t("playback.column.lifecycle"),
      cell: ({ row }) => lifecycleLabel(row.original, t),
    },
    {
      accessorKey: "failure_category",
      header: t("playback.column.failure"),
      cell: ({ row }) => row.original.failure_category ?? t("playback.none"),
    },
    {
      accessorKey: "updated_at",
      header: t("playback.column.updated"),
    },
  ];
}

async function loadPlaybackSessions(
  dataSource: AdminDataSource,
  search: PlaybackSessionsSearch,
  unavailableMessage: string,
): Promise<PlaybackSessionsResult> {
  if (!dataSource.loadPlaybackSessions) {
    return {
      value: mockPlaybackSessions,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadPlaybackSessions(toAdminPlaybackSessionsQuery(search));
}

function toAdminPlaybackSessionsQuery(search: PlaybackSessionsSearch): AdminPlaybackSessionsQuery {
  return {
    source_id: search.source_id,
    kind: search.kind,
    state: search.state,
    limit: search.limit,
    offset: search.offset,
  };
}

function SessionStateBadge({ session }: { session: AdminPlaybackSessionListItem }) {
  if (session.failure_category || session.state === "failed") {
    return <Badge tone="danger">{session.state}</Badge>;
  }

  if (session.state === "running") {
    return <Badge tone="info">{session.state}</Badge>;
  }

  if (session.state === "starting") {
    return <Badge tone="warning">{session.state}</Badge>;
  }

  return <Badge tone={session.terminal ? "success" : "neutral"}>{session.state}</Badge>;
}

function lifecycleLabel(session: AdminPlaybackSessionListItem, t: Translate) {
  if (session.terminal) {
    return t("playback.lifecycle.terminal");
  }

  if (session.active) {
    return t("playback.lifecycle.active");
  }

  return t("playback.lifecycle.inactive");
}

function numberInput(value: string) {
  if (value.trim() === "") {
    return undefined;
  }

  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : undefined;
}
