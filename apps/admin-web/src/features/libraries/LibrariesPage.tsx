import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";
import { Link } from "@tanstack/react-router";
import { ChevronRight, RefreshCw, ShieldCheck } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type { AdminServerConfigDiagnosticsResponse } from "../../adminApi/types";
import { mockSystemConfig } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/Table";

export type LibrariesPageProps = {
  dataSource: AdminDataSource;
};

type LibrariesResult = {
  value: AdminServerConfigDiagnosticsResponse;
  source: DataSourceMode;
  error?: string;
};

type LibraryConfigDiagnostics = AdminServerConfigDiagnosticsResponse["libraries"][number];
type Translate = ReturnType<typeof useI18n>["t"];

export function LibrariesPage({ dataSource }: LibrariesPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-libraries", locale],
    queryFn: () => loadLibraries(dataSource, t),
  });
  const result = query.data ?? {
    value: mockSystemConfig,
    source: "mock" as const,
  };
  const libraries = result.value.libraries;
  const table = useReactTable({
    data: libraries,
    columns: libraryColumns(t),
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
          {t("libraries.refresh")}
        </Button>
      }
      description={t("libraries.description")}
      kicker={t("libraries.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("libraries.title")}
      titleId="libraries-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("libraries.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <DataPanel
        description={t("libraries.configured.description", { count: libraries.length })}
        headerAccessory={
          <div className="searchHint">
            <ShieldCheck size={15} />
            {t("libraries.redactionHint")}
          </div>
        }
        title={t("libraries.configured.title")}
      >
        {query.isLoading ? <RowsSkeleton label={t("libraries.loading")} /> : null}

        {!query.isLoading && libraries.length === 0 ? (
          <EmptyRouteState>{t("libraries.empty")}</EmptyRouteState>
        ) : null}

        {!query.isLoading && libraries.length > 0 ? (
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

async function loadLibraries(
  dataSource: AdminDataSource,
  t: Translate,
): Promise<LibrariesResult> {
  if (!dataSource.loadLibraries) {
    return {
      value: mockSystemConfig,
      source: "mock",
      error: t("libraries.dataSourceUnavailable"),
    };
  }

  return dataSource.loadLibraries();
}

function libraryColumns(t: Translate): Array<ColumnDef<LibraryConfigDiagnostics>> {
  return [
    {
      accessorKey: "name",
      header: t("libraries.column.mediaLibrary"),
      cell: ({ row }) => (
        <div className="routePrimaryCell">
          <strong>{row.original.name}</strong>
          <span>{row.original.id}</span>
        </div>
      ),
    },
    {
      accessorKey: "preset",
      header: t("libraries.column.preset"),
    },
    {
      accessorKey: "backend_kind",
      header: t("libraries.column.backend"),
      cell: ({ row }) => <Badge tone="info">{row.original.backend_kind}</Badge>,
    },
    {
      accessorKey: "root_scheme",
      header: t("libraries.column.rootScheme"),
    },
    {
      accessorKey: "has_webdav_password_env",
      header: t("libraries.column.secretReference"),
      cell: ({ row }) => <SecretReferenceBadge library={row.original} t={t} />,
    },
    {
      id: "runtime",
      header: t("libraries.column.runtimePolicy"),
      cell: ({ row }) => runtimePolicyLabel(row.original, t),
    },
    {
      id: "actions",
      header: "",
      cell: ({ row }) => (
        <Link
          aria-label={t("libraries.manageAria", { name: row.original.name })}
          className="routeTextLink"
          params={{ libraryId: row.original.id }}
          to="/libraries/$libraryId"
        >
          {t("libraries.manage")}
          <ChevronRight size={15} />
        </Link>
      ),
    },
  ];
}

function SecretReferenceBadge({ library, t }: { library: LibraryConfigDiagnostics; t: Translate }) {
  if (library.backend_kind !== "webdav") {
    return <Badge tone="neutral">{t("libraries.secret.notRequired")}</Badge>;
  }

  if (library.has_webdav_password_env) {
    return <Badge tone="success">{t("libraries.secret.configured")}</Badge>;
  }

  return <Badge tone="warning">{t("libraries.secret.missing")}</Badge>;
}

function runtimePolicyLabel(library: LibraryConfigDiagnostics, t: Translate) {
  const timeout = library.webdav_timeout_ms
    ? `${library.webdav_timeout_ms} ms`
    : t("libraries.runtime.defaultTimeout");
  const attempts = library.webdav_max_attempts
    ? `${library.webdav_max_attempts} attempts`
    : t("libraries.runtime.defaultAttempts");

  if (library.backend_kind !== "webdav") {
    return t("libraries.runtime.localPolicy");
  }

  return `${timeout} / ${attempts}`;
}
