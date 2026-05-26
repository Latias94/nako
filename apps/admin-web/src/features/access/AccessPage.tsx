import { RefreshCw, ShieldCheck, UserRound } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type { AdminAccessSummaryResponse } from "../../adminApi/types";
import { mockAccessSummary } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type AccessPageProps = {
  dataSource: AdminDataSource;
};

type AccessResult = {
  value: AdminAccessSummaryResponse;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function AccessPage({ dataSource }: AccessPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-access-summary", locale],
    queryFn: () => loadAccessSummary(dataSource, t("access.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockAccessSummary,
    source: "mock" as const,
  };
  const summary = result.value;
  const libraries = summary.library_access.libraries;

  return (
    <RoutePage
      actions={
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("access.refresh")}
        </Button>
      }
      description={t("access.description")}
      kicker={t("access.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("access.title")}
      titleId="access-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("access.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("access.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="accessSummaryGrid">
            <AccessSummaryCard
              badge={t("access.summary.mode.badge")}
              label={t("access.summary.mode.label")}
              tone="success"
              value={modeLabel(summary.mode, t)}
            />
            <AccessSummaryCard
              badge={
                summary.auth.token_reference_configured
                  ? t("access.summary.auth.tokenConfigured")
                  : t("access.summary.auth.noToken")
              }
              label={t("access.summary.auth.label")}
              tone={summary.auth.enabled ? "success" : "warning"}
              value={
                summary.auth.enabled
                  ? t("access.summary.auth.enabled")
                  : t("access.summary.auth.disabled")
              }
            />
            <AccessSummaryCard
              badge={t("access.summary.library.badge", {
                count: summary.library_access.configured_libraries,
              })}
              label={t("access.summary.library.label")}
              tone="info"
              value={t("access.summary.library.value")}
            />
            <AccessSummaryCard
              badge={t("access.summary.role.badge")}
              label={t("access.summary.role.label")}
              tone="warning"
              value={capabilityLabel(summary.readiness.roles, t)}
            />
          </div>

          <div className="accessPanelGrid">
            <DataPanel
              description={t("access.principal.description")}
              headerAccessory={
                <div className="searchHint">
                  <UserRound size={15} />
                  {t("access.principal.headerAccessory")}
                </div>
              }
              title={t("access.principal.title")}
            >
              <div className="accessPrincipalPanel">
                <div>
                  <span>{t("access.principal.label")}</span>
                  <strong>{summary.principal.display_name}</strong>
                  <small>{summary.principal.principal_id}</small>
                </div>
                <Badge tone="success">
                  {principalKindLabel(summary.principal.principal_kind, t)}
                </Badge>
              </div>
              <div className="accessReadinessList">
                <AccessReadinessRow
                  label={t("access.readiness.singleAdminMode")}
                  state={summary.readiness.single_admin_mode}
                  t={t}
                />
                <AccessReadinessRow
                  label={t("access.readiness.userAccounts")}
                  state={summary.readiness.user_accounts}
                  t={t}
                />
                <AccessReadinessRow
                  label={t("access.readiness.roles")}
                  state={summary.readiness.roles}
                  t={t}
                />
                <AccessReadinessRow
                  label={t("access.readiness.libraryAccessPolicy")}
                  state={summary.readiness.library_access_policy}
                  t={t}
                />
              </div>
            </DataPanel>

            <DataPanel
              description={t("access.library.description")}
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  {t("access.library.headerAccessory")}
                </div>
              }
              title={t("access.library.title")}
            >
              {libraries.length === 0 ? (
                <EmptyRouteState>{t("access.library.empty")}</EmptyRouteState>
              ) : (
                <div className="accessLibraryList">
                  {libraries.map((library) => (
                    <div className="accessLibraryRow" key={library.library_id}>
                      <div>
                        <strong>{library.library_name}</strong>
                        <span>{library.library_id}</span>
                      </div>
                      <div className="accessLibraryMeta">
                        <Badge tone="info">{library.backend_kind}</Badge>
                        <Badge tone="neutral">{library.preset}</Badge>
                        <Badge tone="success">{libraryAccessLabel(library.access, t)}</Badge>
                        <small>{libraryReasonLabel(library.reason, t)}</small>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </DataPanel>

            <DataPanel
              description={t("access.mutation.description")}
              title={t("access.mutation.title")}
            >
              <div className="accessMutationNotice">
                <strong>{t("access.mutation.heading")}</strong>
                <span>{t("access.mutation.body")}</span>
              </div>
            </DataPanel>
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadAccessSummary(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<AccessResult> {
  if (!dataSource.loadAccessSummary) {
    return {
      value: mockAccessSummary,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadAccessSummary();
}

function AccessSummaryCard({
  badge,
  label,
  tone,
  value,
}: {
  badge: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="accessSummaryCard">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function AccessReadinessRow({
  label,
  state,
  t,
}: {
  label: string;
  state: AdminAccessSummaryResponse["readiness"]["roles"];
  t: Translate;
}) {
  const active = state === "active";

  return (
    <div className="accessReadinessRow">
      <span>{label}</span>
      <Badge tone={active ? "success" : "warning"}>{capabilityLabel(state, t)}</Badge>
    </div>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function modeLabel(mode: AdminAccessSummaryResponse["mode"], t: Translate) {
  return mode === "single_admin" ? t("access.mode.singleAdmin") : mode;
}

function principalKindLabel(
  kind: AdminAccessSummaryResponse["principal"]["principal_kind"],
  t: Translate,
) {
  return kind === "local_admin" ? t("access.principalKind.localAdmin") : kind;
}

function capabilityLabel(
  state: AdminAccessSummaryResponse["readiness"]["roles"],
  t: Translate,
) {
  return state === "active" ? t("access.capability.active") : t("access.capability.planned");
}

function libraryAccessLabel(
  access: AdminAccessSummaryResponse["library_access"]["libraries"][number]["access"],
  t: Translate,
) {
  return access === "manage" ? t("access.libraryAccess.manage") : access;
}

function libraryReasonLabel(
  reason: AdminAccessSummaryResponse["library_access"]["libraries"][number]["reason"],
  t: Translate,
) {
  return reason === "single_admin_mode" ? t("access.libraryReason.singleAdminMode") : reason;
}
