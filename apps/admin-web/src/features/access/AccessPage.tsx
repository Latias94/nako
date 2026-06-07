import type { FormEvent } from "react";
import { useState } from "react";
import { Ban, KeyRound, RefreshCw, ShieldCheck, UserPlus, UserRound, X } from "lucide-react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AccessInvitationCreateInput,
  AccessInvitationCreateResult,
  AccessInvitationRow,
  AccessInvitationSummary,
  AdminAccessSummaryResponse,
  AdminUserRole,
} from "../../adminApi/types";
import { mockAccessInvitationSummary, mockAccessSummary } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { AdminLocale, MessageId } from "../../i18n/messages";

const INVITATION_LIST_QUERY = { limit: 20, offset: 0 } as const;
const ACCESS_INVITATION_QUERY_KEY = ["admin-access-invitations"] as const;
const ONE_HOUR_MS = 60 * 60 * 1_000;
const ONE_DAY_MS = 24 * ONE_HOUR_MS;

const invitationRoleOptions = [
  "viewer",
  "library_manager",
  "administrator",
] satisfies AdminUserRole[];

export type AccessPageProps = {
  dataSource: AdminDataSource;
};

type AccessResult = {
  value: AdminAccessSummaryResponse;
  source: DataSourceMode;
  error?: string;
};

type InvitationResult = {
  value: AccessInvitationSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type ExpiryPreset = "default" | "24h" | "7d" | "30d" | "custom";

export function AccessPage({ dataSource }: AccessPageProps) {
  const { locale, t } = useI18n();
  const queryClient = useQueryClient();
  const [emailOrUsername, setEmailOrUsername] = useState("");
  const [role, setRole] = useState<AdminUserRole>("viewer");
  const [expiryPreset, setExpiryPreset] = useState<ExpiryPreset>("7d");
  const [customExpiryHours, setCustomExpiryHours] = useState("72");
  const [createdInvitation, setCreatedInvitation] = useState<AccessInvitationCreateResult | null>(null);
  const [revokeCandidateId, setRevokeCandidateId] = useState<string | null>(null);
  const [mutationMessage, setMutationMessage] = useState<string | null>(null);
  const [mutationError, setMutationError] = useState<string | null>(null);

  const summaryQuery = useQuery({
    queryKey: ["admin-access-summary", locale],
    queryFn: () => loadAccessSummary(dataSource, t("access.dataSourceUnavailable")),
  });
  const invitationsQuery = useQuery({
    queryKey: [...ACCESS_INVITATION_QUERY_KEY, locale],
    queryFn: () => loadAccessInvitations(dataSource, t("access.invitations.dataSourceUnavailable")),
  });
  const summaryResult = summaryQuery.data ?? {
    value: mockAccessSummary,
    source: "mock" as const,
  };
  const invitationsResult = invitationsQuery.data ?? {
    value: mockAccessInvitationSummary,
    source: "mock" as const,
  };
  const createInvitationMutation = useMutation<
    AccessInvitationCreateResult,
    Error,
    AccessInvitationCreateInput
  >({
    mutationFn: async (input) => {
      if (invitationsResult.source !== "live") {
        throw new Error(t("access.invitations.notLiveError"));
      }
      if (!dataSource.createAccessInvitation) {
        throw new Error(t("access.invitations.createUnavailable"));
      }

      return dataSource.createAccessInvitation(input);
    },
    onMutate: () => {
      setCreatedInvitation(null);
      setMutationMessage(null);
      setMutationError(null);
    },
    onSuccess: (result) => {
      setCreatedInvitation(result);
      setMutationMessage(
        t("access.invitations.createSucceeded", {
          invitationId: result.invitation.invitationId,
        }),
      );
      setEmailOrUsername("");
      void queryClient.invalidateQueries({ queryKey: ACCESS_INVITATION_QUERY_KEY });
    },
    onError: (error) => {
      setMutationError(error.message);
    },
  });
  const revokeInvitationMutation = useMutation<AccessInvitationRow, Error, string>({
    mutationFn: async (invitationId) => {
      if (invitationsResult.source !== "live") {
        throw new Error(t("access.invitations.notLiveError"));
      }
      if (!dataSource.revokeAccessInvitation) {
        throw new Error(t("access.invitations.revokeUnavailable"));
      }

      return dataSource.revokeAccessInvitation(invitationId);
    },
    onMutate: () => {
      setMutationMessage(null);
      setMutationError(null);
    },
    onSuccess: (row) => {
      setRevokeCandidateId(null);
      setMutationMessage(
        t("access.invitations.revokeSucceeded", {
          invitationId: row.invitationId,
          status: invitationStatusLabel(row.status, t),
        }),
      );
      void queryClient.invalidateQueries({ queryKey: ACCESS_INVITATION_QUERY_KEY });
    },
    onError: (error) => {
      setMutationError(error.message);
    },
  });

  const summary = summaryResult.value;
  const libraries = summary.library_access.libraries;
  const invitations = invitationsResult.value.invitations;
  const isLoading = summaryQuery.isLoading || invitationsQuery.isLoading;
  const isFetching = summaryQuery.isFetching || invitationsQuery.isFetching;
  const canCreateInvitation =
    invitationsResult.source === "live" && Boolean(dataSource.createAccessInvitation);
  const canRevokeInvitation =
    invitationsResult.source === "live" && Boolean(dataSource.revokeAccessInvitation);
  const createExpiresInMs = invitationExpiresInMs(expiryPreset, customExpiryHours);
  const canSubmitCreate =
    canCreateInvitation &&
    !createInvitationMutation.isPending &&
    !revokeInvitationMutation.isPending &&
    createExpiresInMs !== undefined;

  function submitCreateInvitation(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (createExpiresInMs === undefined) {
      return;
    }

    createInvitationMutation.mutate({
      emailOrUsername: trimmedOrNull(emailOrUsername),
      roles: [role],
      expiresInMs: createExpiresInMs,
    });
  }

  return (
    <RoutePage
      actions={
        <Button
          disabled={isFetching}
          onClick={() => {
            void summaryQuery.refetch();
            void invitationsQuery.refetch();
          }}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("access.refresh")}
        </Button>
      }
      description={t("access.description")}
      kicker={t("access.kicker")}
      status={<SourceLabel source={combinedSource(summaryResult.source, invitationsResult.source)} />}
      title={t("access.title")}
      titleId="access-route-title"
    >
      {summaryResult.error ? (
        <RouteNotice>{t("access.fallback", { error: summaryResult.error })}</RouteNotice>
      ) : null}

      {invitationsResult.error ? (
        <RouteNotice>
          {t("access.invitations.fallback", { error: invitationsResult.error })}
        </RouteNotice>
      ) : null}

      {mutationError ? <RouteNotice>{mutationError}</RouteNotice> : null}
      {mutationMessage ? <RouteNotice>{mutationMessage}</RouteNotice> : null}

      {isLoading ? <RowsSkeleton label={t("access.loading")} /> : null}

      {!isLoading ? (
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
              description={t("access.invitations.description")}
              headerAccessory={
                <div className="searchHint">
                  <KeyRound size={15} />
                  <SourceLabel source={invitationsResult.source} />
                </div>
              }
              title={t("access.invitations.title")}
            >
              <form className="accessInvitationForm" onSubmit={submitCreateInvitation}>
                <label>
                  <span>{t("access.invitations.emailOrUsername")}</span>
                  <input
                    aria-label={t("access.invitations.emailOrUsername")}
                    onChange={(event) => setEmailOrUsername(event.target.value)}
                    placeholder={t("access.invitations.emailPlaceholder")}
                    value={emailOrUsername}
                  />
                </label>
                <label>
                  <span>{t("access.invitations.role")}</span>
                  <select
                    aria-label={t("access.invitations.role")}
                    onChange={(event) => setRole(event.target.value as AdminUserRole)}
                    value={role}
                  >
                    {invitationRoleOptions.map((option) => (
                      <option key={option} value={option}>
                        {roleLabel(option, t)}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>{t("access.invitations.expiry")}</span>
                  <select
                    aria-label={t("access.invitations.expiry")}
                    onChange={(event) => setExpiryPreset(event.target.value as ExpiryPreset)}
                    value={expiryPreset}
                  >
                    <option value="default">{t("access.invitations.expiry.default")}</option>
                    <option value="24h">{t("access.invitations.expiry.24h")}</option>
                    <option value="7d">{t("access.invitations.expiry.7d")}</option>
                    <option value="30d">{t("access.invitations.expiry.30d")}</option>
                    <option value="custom">{t("access.invitations.expiry.custom")}</option>
                  </select>
                </label>
                {expiryPreset === "custom" ? (
                  <label>
                    <span>{t("access.invitations.customHours")}</span>
                    <input
                      aria-label={t("access.invitations.customHours")}
                      min="1"
                      onChange={(event) => setCustomExpiryHours(event.target.value)}
                      step="1"
                      type="number"
                      value={customExpiryHours}
                    />
                  </label>
                ) : null}
                <Button disabled={!canSubmitCreate} size="sm" type="submit">
                  <UserPlus size={14} />
                  {createInvitationMutation.isPending
                    ? t("access.invitations.creating")
                    : t("access.invitations.create")}
                </Button>
              </form>

              {!canCreateInvitation ? (
                <div className="accessInlineNotice">{t("access.invitations.mutationDisabled")}</div>
              ) : null}
              {createExpiresInMs === undefined ? (
                <div className="accessInlineNotice">{t("access.invitations.invalidCustomExpiry")}</div>
              ) : null}

              {createdInvitation ? (
                <div className="accessInvitationToken">
                  <div>
                    <strong>{t("access.invitations.oneTimeToken")}</strong>
                    <span>{t("access.invitations.oneTimeTokenCopy")}</span>
                  </div>
                  <code>{createdInvitation.token}</code>
                </div>
              ) : null}

              {invitations.length === 0 ? (
                <EmptyRouteState>{t("access.invitations.empty")}</EmptyRouteState>
              ) : (
                <div className="accessInvitationList">
                  {invitations.map((invitation) => (
                    <InvitationRow
                      canRevoke={canRevokeInvitation}
                      invitation={invitation}
                      isPending={revokeInvitationMutation.isPending}
                      key={invitation.invitationId}
                      locale={locale}
                      onCancel={() => setRevokeCandidateId(null)}
                      onConfirm={(invitationId) => revokeInvitationMutation.mutate(invitationId)}
                      onPrepare={(invitationId) => {
                        setCreatedInvitation(null);
                        setMutationMessage(null);
                        setMutationError(null);
                        setRevokeCandidateId(invitationId);
                      }}
                      pendingInvitationId={revokeCandidateId}
                      t={t}
                    />
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

async function loadAccessInvitations(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<InvitationResult> {
  if (!dataSource.loadAccessInvitations) {
    return {
      value: mockAccessInvitationSummary,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadAccessInvitations(INVITATION_LIST_QUERY);
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

function InvitationRow({
  canRevoke,
  invitation,
  isPending,
  locale,
  onCancel,
  onConfirm,
  onPrepare,
  pendingInvitationId,
  t,
}: {
  canRevoke: boolean;
  invitation: AccessInvitationRow;
  isPending: boolean;
  locale: AdminLocale;
  onCancel(): void;
  onConfirm(invitationId: string): void;
  onPrepare(invitationId: string): void;
  pendingInvitationId: string | null;
  t: Translate;
}) {
  return (
    <div className="accessInvitationRow">
      <div className="accessInvitationIdentity">
        <strong>{invitation.emailOrUsername ?? t("access.invitations.noRecipient")}</strong>
        <span>{invitation.invitationId}</span>
        <div className="accessInvitationBadges">
          <Badge tone={invitationStatusTone(invitation.status)}>
            {invitationStatusLabel(invitation.status, t)}
          </Badge>
          {invitation.roles.map((invitationRole) => (
            <Badge key={invitationRole} tone="info">
              {roleLabel(invitationRole, t)}
            </Badge>
          ))}
        </div>
      </div>
      <div className="accessInvitationFacts">
        <span>
          {t("access.invitations.createdAt", {
            value: formatTimestamp(invitation.createdAtMs, locale),
          })}
        </span>
        <span>
          {t("access.invitations.expiresAt", {
            value: formatTimestamp(invitation.expiresAtMs, locale),
          })}
        </span>
        {invitation.redeemedAtMs ? (
          <span>
            {t("access.invitations.redeemedAt", {
              value: formatTimestamp(invitation.redeemedAtMs, locale),
            })}
          </span>
        ) : null}
        {invitation.revokedAtMs ? (
          <span>
            {t("access.invitations.revokedAt", {
              value: formatTimestamp(invitation.revokedAtMs, locale),
            })}
          </span>
        ) : null}
      </div>
      <InvitationRevokeAction
        canRevoke={canRevoke}
        invitation={invitation}
        isPending={isPending}
        onCancel={onCancel}
        onConfirm={onConfirm}
        onPrepare={onPrepare}
        pendingInvitationId={pendingInvitationId}
        t={t}
      />
    </div>
  );
}

function InvitationRevokeAction({
  canRevoke,
  invitation,
  isPending,
  onCancel,
  onConfirm,
  onPrepare,
  pendingInvitationId,
  t,
}: {
  canRevoke: boolean;
  invitation: AccessInvitationRow;
  isPending: boolean;
  onCancel(): void;
  onConfirm(invitationId: string): void;
  onPrepare(invitationId: string): void;
  pendingInvitationId: string | null;
  t: Translate;
}) {
  if (invitation.status !== "pending") {
    return <Badge tone="neutral">{t("access.invitations.noRevoke")}</Badge>;
  }

  if (pendingInvitationId === invitation.invitationId) {
    return (
      <div className="accessInvitationConfirm">
        <small>{t("access.invitations.confirmRevokeCopy", { invitationId: invitation.invitationId })}</small>
        <div className="accessInvitationConfirmActions">
          <Button
            aria-label={t("access.invitations.confirmRevokeAria", {
              invitationId: invitation.invitationId,
            })}
            disabled={!canRevoke || isPending}
            onClick={() => onConfirm(invitation.invitationId)}
            size="sm"
          >
            <Ban size={14} />
            {isPending ? t("access.invitations.revoking") : t("access.invitations.confirmRevoke")}
          </Button>
          <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
            <X size={14} />
            {t("access.invitations.cancel")}
          </Button>
        </div>
      </div>
    );
  }

  return (
    <Button
      aria-label={t("access.invitations.prepareRevokeAria", {
        invitationId: invitation.invitationId,
      })}
      disabled={!canRevoke || isPending}
      onClick={() => onPrepare(invitation.invitationId)}
      size="sm"
      variant="outline"
    >
      <Ban size={14} />
      {t("access.invitations.prepareRevoke")}
    </Button>
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

function roleLabel(role: AdminUserRole, t: Translate) {
  if (role === "administrator") {
    return t("access.role.administrator");
  }
  if (role === "library_manager") {
    return t("access.role.libraryManager");
  }

  return t("access.role.viewer");
}

function invitationStatusLabel(status: AccessInvitationRow["status"], t: Translate) {
  if (status === "redeemed") {
    return t("access.invitations.status.redeemed");
  }
  if (status === "revoked") {
    return t("access.invitations.status.revoked");
  }
  if (status === "expired") {
    return t("access.invitations.status.expired");
  }

  return t("access.invitations.status.pending");
}

function invitationStatusTone(status: AccessInvitationRow["status"]): BadgeTone {
  if (status === "redeemed") {
    return "success";
  }
  if (status === "revoked") {
    return "warning";
  }
  if (status === "expired") {
    return "neutral";
  }

  return "info";
}

function invitationExpiresInMs(
  preset: ExpiryPreset,
  customExpiryHours: string,
): number | null | undefined {
  if (preset === "default") {
    return null;
  }
  if (preset === "24h") {
    return ONE_DAY_MS;
  }
  if (preset === "7d") {
    return 7 * ONE_DAY_MS;
  }
  if (preset === "30d") {
    return 30 * ONE_DAY_MS;
  }

  const parsed = Number(customExpiryHours);
  return Number.isInteger(parsed) && parsed > 0 ? parsed * ONE_HOUR_MS : undefined;
}

function trimmedOrNull(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function combinedSource(first: DataSourceMode, second: DataSourceMode): DataSourceMode {
  if (first === second) {
    return first;
  }
  if (first === "live" || second === "live") {
    return "hybrid";
  }
  if (first === "planned" || second === "planned") {
    return "planned";
  }

  return "mock";
}

function formatTimestamp(value: number, locale: AdminLocale) {
  return new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}
