import { Link } from "@tanstack/react-router";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState, type ReactNode } from "react";
import {
  ArrowLeft,
  Download,
  Pencil,
  Play,
  RefreshCw,
  Save,
  ShieldCheck,
  Upload,
  X,
} from "lucide-react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  AdminMetadataProfile,
  LibraryCommandAction,
  LibraryConfigDiagnostics,
  LibraryManagementDetail,
  LibrarySourceInventorySummary,
} from "../../adminApi/types";
import { mockLibraryMetadataProfile, mockSystemConfig } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";

export type LibraryDetailPageProps = {
  dataSource: AdminDataSource;
  libraryId: string;
};

type LibraryDetailResult = {
  value: LibraryManagementDetail;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type Translate = ReturnType<typeof useI18n>["t"];
type MetadataProfileForm = {
  itemKinds: string;
  localReaders: string;
  metadataProviders: string;
  imageProviders: string;
  language: string;
  country: string;
  refreshMode: AdminMetadataProfile["refresh_mode"];
  localMetadataPolicy: AdminMetadataProfile["local_metadata_policy"];
  scanEnabled: boolean;
  addonScrape: boolean;
  addonWriteback: boolean;
};

export function LibraryDetailPage({ dataSource, libraryId }: LibraryDetailPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-library-detail", libraryId, locale],
    queryFn: () => loadLibraryDetail(dataSource, libraryId, t),
  });
  const result = query.data ?? {
    value: mockLibraryDetail(libraryId),
    source: "mock" as const,
  };
  const { library, metadataProfile, sourceInventory } = result.value;
  const [editingProfile, setEditingProfile] = useState(false);
  const [profileDraft, setProfileDraft] = useState(() =>
    profileToForm(metadataProfile.profile),
  );
  const [pendingCommand, setPendingCommand] = useState<LibraryCommandAction | null>(null);
  const updateProfileMutation = useMutation({
    mutationFn: async () => {
      if (!dataSource.updateLibraryMetadataProfile) {
        throw new Error(t("libraryDetail.metadata.updateUnavailable"));
      }

      return dataSource.updateLibraryMetadataProfile(
        libraryId,
        formToProfile(profileDraft, metadataProfile.profile),
      );
    },
    onSuccess: () => {
      setEditingProfile(false);
      void query.refetch();
    },
  });
  const commandMutation = useMutation({
    mutationFn: async (action: LibraryCommandAction) => {
      if (!dataSource.runLibraryCommand) {
        throw new Error(t("libraryDetail.commands.unavailable"));
      }

      return dataSource.runLibraryCommand(libraryId, action);
    },
    onSuccess: () => {
      setPendingCommand(null);
      void query.refetch();
    },
  });

  useEffect(() => {
    if (!editingProfile) {
      const next = profileToForm(metadataProfile.profile);
      setProfileDraft((current) => (profileFormEquals(current, next) ? current : next));
    }
  }, [editingProfile, metadataProfile.profile]);

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            activeOptions={{ exact: true }}
            className="routeTextLink routeBackLink"
            to="/libraries"
          >
            <ArrowLeft size={16} />
            {t("libraryDetail.back")}
          </Link>
          <Button
            disabled={query.isFetching}
            onClick={() => void query.refetch()}
            variant="outline"
          >
            <RefreshCw size={16} />
            {t("libraryDetail.refresh")}
          </Button>
        </div>
      }
      description={
        library
          ? t("libraryDetail.description.ready")
          : t("libraryDetail.description.missing")
      }
      kicker={t("libraryDetail.kicker")}
      status={<SourceLabel source={result.source} />}
      title={library?.name ?? t("libraryDetail.fallbackTitle")}
      titleId="library-detail-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("libraryDetail.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("libraryDetail.loading")} /> : null}

      {!query.isLoading && !library ? (
        <EmptyRouteState>
          {t("libraryDetail.empty", { libraryId })}
        </EmptyRouteState>
      ) : null}

      {!query.isLoading && library ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("libraryDetail.facts.description", {
              count: result.value.configuredLibraryCount,
            })}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("libraryDetail.facts.redactionHint")}
              </div>
            }
            title={t("libraryDetail.facts.title")}
          >
            <div className="libraryFactList">
              <FactRow
                badge={library.preset}
                label={t("libraryDetail.facts.preset")}
                tone="info"
                value={library.id}
              />
              <FactRow
                badge={library.backend_kind}
                label={t("libraryDetail.facts.storageBackend")}
                tone={library.backend_kind === "webdav" ? "warning" : "success"}
                value={library.root_scheme}
              />
              <FactRow
                badge={secretReferenceLabel(library, t)}
                label={t("libraryDetail.facts.secretReference")}
                tone={secretReferenceTone(library)}
                value={
                  library.backend_kind === "webdav"
                    ? t("libraryDetail.facts.secretWebdav")
                    : t("libraries.secret.notRequired")
                }
              />
              <FactRow
                badge={
                  library.backend_kind === "webdav"
                    ? t("libraryDetail.facts.runtimeRemote")
                    : t("libraryDetail.facts.runtimeLocal")
                }
                detail={runtimePolicyLabel(library, t)}
                label={t("libraryDetail.facts.runtime")}
                tone="neutral"
                value={t("libraryDetail.facts.runtimeValue")}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("libraryDetail.metadata.description")}
            headerAccessory={
              editingProfile ? (
                <Button
                  onClick={() => {
                    setEditingProfile(false);
                    setProfileDraft(profileToForm(metadataProfile.profile));
                    updateProfileMutation.reset();
                  }}
                  size="sm"
                  variant="ghost"
                >
                  <X size={15} />
                  {t("libraryDetail.metadata.cancelEdit")}
                </Button>
              ) : (
                <Button
                  onClick={() => {
                    setEditingProfile(true);
                    updateProfileMutation.reset();
                  }}
                  size="sm"
                  variant="outline"
                >
                  <Pencil size={15} />
                  {t("libraryDetail.metadata.edit")}
                </Button>
              )
            }
            title={t("libraryDetail.metadata.title")}
          >
            <div className="libraryAuthorityNotice">
              <Badge tone="warning">{t("libraryDetail.metadata.replacementBadge")}</Badge>
              <span>{t("libraryDetail.metadata.replacementCopy")}</span>
            </div>
            {updateProfileMutation.error ? (
              <div className="libraryInlineNotice" role="alert">
                {errorMessage(updateProfileMutation.error, t)}
              </div>
            ) : null}
            {updateProfileMutation.data ? (
              <div className="libraryInlineNotice success" role="status">
                {t("libraryDetail.metadata.updateSuccess")}
              </div>
            ) : null}
            {editingProfile ? (
              <MetadataProfileEditor
                draft={profileDraft}
                isSaving={updateProfileMutation.isPending}
                onChange={setProfileDraft}
                onSave={() => updateProfileMutation.mutate()}
                t={t}
              />
            ) : (
              <div className="libraryFactList">
                <FactRow
                  badge={metadataProfile.profile.refresh_mode}
                  label={t("libraryDetail.metadata.refreshMode")}
                  tone="info"
                  value={formatLocale(
                    metadataProfile.profile.language,
                    metadataProfile.profile.country,
                    t,
                  )}
                />
                <FactRow
                  badge={metadataProfile.profile.local_metadata_policy}
                  label={t("libraryDetail.metadata.localMetadata")}
                  tone="success"
                  value={t("libraryDetail.metadata.localReaders", {
                    count: metadataProfile.profile.local_readers.length,
                  })}
                />
                <FactRow
                  badge={t("libraryDetail.metadata.providersBadge", {
                    count: metadataProfile.profile.metadata_providers.length,
                  })}
                  detail={
                    metadataProfile.profile.metadata_providers.join(", ")
                    || t("libraryDetail.metadata.providerNone")
                  }
                  label={t("libraryDetail.metadata.providerOrder")}
                  tone="neutral"
                  value={
                    metadataProfile.profile.image_providers.join(", ")
                    || t("libraryDetail.metadata.noImageProviders")
                  }
                />
              </div>
            )}
            <div className="libraryPlanGrid">
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.local_nfo_import} label={t("libraryDetail.plan.nfoImport")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.provider_refresh} label={t("libraryDetail.plan.providerRefresh")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.addon_scrape} label={t("libraryDetail.plan.addonScrape")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.addon_writeback} label={t("libraryDetail.plan.addonWriteback")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.embedded_read} label={t("libraryDetail.plan.embeddedRead")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.sidecar_read} label={t("libraryDetail.plan.sidecarRead")} t={t} />
              <PlanBadge enabled={metadataProfile.scan_acquisition_plan.image_discovery} label={t("libraryDetail.plan.imageDiscovery")} t={t} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("libraryDetail.sources.description")}
            headerAccessory={<SourceLabel source={sourceInventory.source} />}
            title={t("libraryDetail.sources.title")}
          >
            {sourceInventory.error ? (
              <div className="libraryInlineNotice" role="status">
                {t("libraryDetail.sources.fallback", { error: sourceInventory.error })}
              </div>
            ) : null}
            <div className="libraryFactList">
              <FactRow
                badge={t("libraryDetail.sources.returned", {
                  count: sourceInventory.returnedSourceCount,
                })}
                label={t("libraryDetail.sources.visible")}
                tone={sourceInventory.source === "live" ? "success" : "warning"}
                value={t("libraryDetail.sources.visibleValue", {
                  count: sourceInventory.sourceCount,
                })}
              />
              <FactRow
                badge={sourceInventory.latestScanJob?.status ?? t("source.planned")}
                label={t("libraryDetail.sources.lastScan")}
                tone={sourceInventory.latestScanJob ? "info" : "neutral"}
                value={
                  sourceInventory.latestScanJob
                    ? formatJobTime(sourceInventory.latestScanJob.queuedAt)
                    : t("libraryDetail.sources.lastScanValue")
                }
              />
              <FactRow
                badge={t("libraryDetail.sources.failureBadge", {
                  count: sourceInventory.failedJobCount,
                })}
                label={t("libraryDetail.sources.failures")}
                tone={sourceInventory.failedJobCount > 0 ? "danger" : "success"}
                value={
                  sourceInventory.latestScanJob?.resourceClass
                  ?? t("libraryDetail.sources.failuresValue")
                }
              />
              <FactRow
                badge={t("libraryDetail.sources.probedBadge", {
                  count: sourceInventory.probedSourceCount,
                })}
                label={t("libraryDetail.sources.technicalState")}
                tone={sourceInventory.probedSourceCount > 0 ? "success" : "neutral"}
                value={formatBytes(sourceInventory.totalSizeBytes, t)}
              />
            </div>
            {sourceInventory.samples.length > 0 ? (
              <div className="librarySourceSamples">
                {sourceInventory.samples.map((sample) => (
                  <div className="librarySourceSample" key={sample.id}>
                    <div>
                      <strong>{sample.fileName}</strong>
                      <span>{sample.itemTitle ?? sample.id}</span>
                    </div>
                    <Badge tone={sample.hasProbe ? "success" : "neutral"}>
                      {sample.hasProbe
                        ? t("libraryDetail.sources.probed")
                        : t("libraryDetail.sources.unprobed")}
                    </Badge>
                  </div>
                ))}
              </div>
            ) : null}
          </DataPanel>

          <DataPanel
            description={t("libraryDetail.operations.description")}
            title={t("libraryDetail.operations.title")}
          >
            {commandMutation.error ? (
              <div className="libraryInlineNotice" role="alert">
                {errorMessage(commandMutation.error, t)}
              </div>
            ) : null}
            {commandMutation.data ? (
              <div className="libraryInlineNotice success" role="status">
                {t("libraryDetail.commands.queued", {
                  jobId: commandMutation.data.job.id,
                  status: commandMutation.data.job.status,
                })}
              </div>
            ) : null}
            <div className="libraryOperationGrid">
              <OperationTile
                label={t("libraryDetail.operations.sourceInventory")}
                status={sourceModeLabel(sourceInventory.source, t)}
                tone={sourceInventory.source === "live" ? "success" : "warning"}
                value={t("libraryDetail.operations.sourceInventoryValue", {
                  count: sourceInventory.sourceCount,
                })}
              />
              <OperationTile
                label={t("libraryDetail.operations.metadataProfileEdit")}
                status={t("libraryDetail.operations.metadataProfileEditStatus")}
                tone="success"
                value={t("libraryDetail.operations.metadataProfileEditValue")}
              />
              <CommandTile
                action="scan"
                icon={<Play size={15} />}
                isPending={commandMutation.isPending}
                label={t("libraryDetail.operations.libraryScan")}
                onCancel={() => setPendingCommand(null)}
                onConfirm={() => commandMutation.mutate("scan")}
                onPrepare={() => {
                  commandMutation.reset();
                  setPendingCommand("scan");
                }}
                pending={pendingCommand === "scan"}
                status={t("libraryDetail.operations.libraryScanStatus")}
                t={t}
                value={t("libraryDetail.operations.libraryScanValue")}
              />
              <CommandTile
                action="nfoImport"
                icon={<Download size={15} />}
                isPending={commandMutation.isPending}
                label={t("libraryDetail.operations.nfoImport")}
                onCancel={() => setPendingCommand(null)}
                onConfirm={() => commandMutation.mutate("nfoImport")}
                onPrepare={() => {
                  commandMutation.reset();
                  setPendingCommand("nfoImport");
                }}
                pending={pendingCommand === "nfoImport"}
                status={t("libraryDetail.operations.nfoImportStatus")}
                t={t}
                value={t("libraryDetail.operations.nfoImportValue")}
              />
              <CommandTile
                action="nfoExport"
                icon={<Upload size={15} />}
                isPending={commandMutation.isPending}
                label={t("libraryDetail.operations.nfoExport")}
                onCancel={() => setPendingCommand(null)}
                onConfirm={() => commandMutation.mutate("nfoExport")}
                onPrepare={() => {
                  commandMutation.reset();
                  setPendingCommand("nfoExport");
                }}
                pending={pendingCommand === "nfoExport"}
                status={t("libraryDetail.operations.nfoExportStatus")}
                t={t}
                value={t("libraryDetail.operations.nfoExportValue")}
              />
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

async function loadLibraryDetail(
  dataSource: AdminDataSource,
  libraryId: string,
  t: Translate,
): Promise<LibraryDetailResult> {
  if (!dataSource.loadLibraryDetail) {
    return {
      value: mockLibraryDetail(libraryId),
      source: "mock",
      error: t("libraryDetail.dataSourceUnavailable"),
    };
  }

  return dataSource.loadLibraryDetail(libraryId);
}

function mockLibraryDetail(libraryId: string): LibraryManagementDetail {
  return {
    configuredLibraryCount: mockSystemConfig.libraries.length,
    library: mockSystemConfig.libraries.find((library) => library.id === libraryId) ?? null,
    metadataProfile: mockLibraryMetadataProfile(libraryId),
    sourceInventory: mockLibrarySourceInventory(),
  };
}

function mockLibrarySourceInventory(): LibrarySourceInventorySummary {
  return {
    source: "mock",
    sourceCount: 0,
    linkedItemCount: 0,
    probedSourceCount: 0,
    returnedSourceCount: 0,
    totalSizeBytes: null,
    latestScanJob: null,
    failedJobCount: 0,
    page: {
      limit: 50,
      offset: 0,
      returned: 0,
    },
    samples: [],
  };
}

function FactRow({
  badge,
  detail,
  label,
  tone,
  value,
}: {
  badge: string;
  detail?: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="libraryFactRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail ? <small>{detail}</small> : null}
      </div>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function PlanBadge({ enabled, label, t }: { enabled: boolean; label: string; t: Translate }) {
  return (
    <div className="libraryPlanBadge">
      <span>{label}</span>
      <Badge tone={enabled ? "success" : "neutral"}>
        {enabled ? t("libraryDetail.plan.enabled") : t("libraryDetail.plan.disabled")}
      </Badge>
    </div>
  );
}

function MetadataProfileEditor({
  draft,
  isSaving,
  onChange,
  onSave,
  t,
}: {
  draft: MetadataProfileForm;
  isSaving: boolean;
  onChange: (draft: MetadataProfileForm) => void;
  onSave: () => void;
  t: Translate;
}) {
  const update = <K extends keyof MetadataProfileForm>(
    key: K,
    value: MetadataProfileForm[K],
  ) => onChange({ ...draft, [key]: value });

  return (
    <div className="metadataProfileEditor">
      <label>
        <span>{t("libraryDetail.metadata.language")}</span>
        <input
          aria-label={t("libraryDetail.metadata.language")}
          onChange={(event) => update("language", event.target.value)}
          placeholder="en-US"
          value={draft.language}
        />
      </label>
      <label>
        <span>{t("libraryDetail.metadata.country")}</span>
        <input
          aria-label={t("libraryDetail.metadata.country")}
          onChange={(event) => update("country", event.target.value)}
          placeholder="US"
          value={draft.country}
        />
      </label>
      <label>
        <span>{t("libraryDetail.metadata.refreshMode")}</span>
        <select
          aria-label={t("libraryDetail.metadata.refreshMode")}
          onChange={(event) =>
            update("refreshMode", event.target.value as MetadataProfileForm["refreshMode"])
          }
          value={draft.refreshMode}
        >
          {["none", "validation_only", "default", "missing_only", "full_refresh"].map((value) => (
            <option key={value} value={value}>
              {value}
            </option>
          ))}
        </select>
      </label>
      <label>
        <span>{t("libraryDetail.metadata.localMetadata")}</span>
        <select
          aria-label={t("libraryDetail.metadata.localMetadata")}
          onChange={(event) =>
            update(
              "localMetadataPolicy",
              event.target.value as MetadataProfileForm["localMetadataPolicy"],
            )
          }
          value={draft.localMetadataPolicy}
        >
          {["disabled", "read_only", "local_first", "remote_first", "write_sidecar"].map((value) => (
            <option key={value} value={value}>
              {value}
            </option>
          ))}
        </select>
      </label>
      <TextAreaField
        label={t("libraryDetail.metadata.itemKinds")}
        onChange={(value) => update("itemKinds", value)}
        value={draft.itemKinds}
      />
      <TextAreaField
        label={t("libraryDetail.metadata.localReadersField")}
        onChange={(value) => update("localReaders", value)}
        value={draft.localReaders}
      />
      <TextAreaField
        label={t("libraryDetail.metadata.metadataProviders")}
        onChange={(value) => update("metadataProviders", value)}
        value={draft.metadataProviders}
      />
      <TextAreaField
        label={t("libraryDetail.metadata.imageProviders")}
        onChange={(value) => update("imageProviders", value)}
        value={draft.imageProviders}
      />
      <div className="metadataProfileToggles">
        <label>
          <input
            checked={draft.scanEnabled}
            onChange={(event) => update("scanEnabled", event.target.checked)}
            type="checkbox"
          />
          <span>{t("libraryDetail.metadata.scanEnabled")}</span>
        </label>
        <label>
          <input
            checked={draft.addonScrape}
            onChange={(event) => update("addonScrape", event.target.checked)}
            type="checkbox"
          />
          <span>{t("libraryDetail.plan.addonScrape")}</span>
        </label>
        <label>
          <input
            checked={draft.addonWriteback}
            onChange={(event) => update("addonWriteback", event.target.checked)}
            type="checkbox"
          />
          <span>{t("libraryDetail.plan.addonWriteback")}</span>
        </label>
      </div>
      <div className="metadataProfileActions">
        <Button disabled={isSaving} onClick={onSave}>
          <Save size={15} />
          {isSaving
            ? t("libraryDetail.metadata.saving")
            : t("libraryDetail.metadata.saveReplacement")}
        </Button>
      </div>
    </div>
  );
}

function TextAreaField({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label>
      <span>{label}</span>
      <textarea
        aria-label={label}
        onChange={(event) => onChange(event.target.value)}
        value={value}
      />
    </label>
  );
}

function OperationTile({
  label,
  status,
  tone,
  value,
}: {
  label: string;
  status: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="libraryOperationTile">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{status}</Badge>
    </div>
  );
}

function CommandTile({
  action,
  icon,
  isPending,
  label,
  onCancel,
  onConfirm,
  onPrepare,
  pending,
  status,
  t,
  value,
}: {
  action: LibraryCommandAction;
  icon: ReactNode;
  isPending: boolean;
  label: string;
  onCancel: () => void;
  onConfirm: () => void;
  onPrepare: () => void;
  pending: boolean;
  status: string;
  t: Translate;
  value: string;
}) {
  return (
    <div className="libraryOperationTile" data-action={action}>
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone="info">{status}</Badge>
      {pending ? (
        <div className="libraryCommandConfirm">
          <small>{t("libraryDetail.commands.confirmation")}</small>
          <div>
            <Button disabled={isPending} onClick={onConfirm} size="sm">
              {icon}
              {isPending ? t("libraryDetail.commands.queueing") : t("libraryDetail.commands.confirm")}
            </Button>
            <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
              {t("libraryDetail.commands.cancel")}
            </Button>
          </div>
        </div>
      ) : (
        <Button disabled={isPending} onClick={onPrepare} size="sm" variant="outline">
          {icon}
          {t("libraryDetail.commands.prepare")}
        </Button>
      )}
    </div>
  );
}

function secretReferenceLabel(library: LibraryConfigDiagnostics, t: Translate) {
  if (library.backend_kind !== "webdav") {
    return t("libraries.secret.notRequired");
  }

  return library.has_webdav_password_env
    ? t("libraries.secret.configured")
    : t("libraries.secret.missing");
}

function secretReferenceTone(library: LibraryConfigDiagnostics): BadgeTone {
  if (library.backend_kind !== "webdav") {
    return "neutral";
  }

  return library.has_webdav_password_env ? "success" : "warning";
}

function runtimePolicyLabel(library: LibraryConfigDiagnostics, t: Translate) {
  if (library.backend_kind !== "webdav") {
    return t("libraries.runtime.localPolicy");
  }

  const timeout = library.webdav_timeout_ms
    ? `${library.webdav_timeout_ms} ms`
    : t("libraries.runtime.defaultTimeout");
  const attempts = library.webdav_max_attempts
    ? `${library.webdav_max_attempts} attempts`
    : t("libraries.runtime.defaultAttempts");
  return `${timeout} / ${attempts}`;
}

function formatLocale(language: string | null, country: string | null, t: Translate) {
  if (language && country) {
    return `${language} / ${country}`;
  }

  return language ?? country ?? t("libraryDetail.metadata.defaultLocale");
}

function profileToForm(profile: AdminMetadataProfile): MetadataProfileForm {
  return {
    itemKinds: profile.item_kinds.join(", "),
    localReaders: profile.local_readers.join(", "),
    metadataProviders: profile.metadata_providers.join(", "),
    imageProviders: profile.image_providers.join(", "),
    language: profile.language ?? "",
    country: profile.country ?? "",
    refreshMode: profile.refresh_mode,
    localMetadataPolicy: profile.local_metadata_policy,
    scanEnabled: profile.scan.enabled,
    addonScrape: profile.scan.addon_scrape,
    addonWriteback: profile.scan.addon_writeback,
  };
}

function formToProfile(
  draft: MetadataProfileForm,
  current: AdminMetadataProfile,
): AdminMetadataProfile {
  return {
    ...current,
    item_kinds: parseList(draft.itemKinds),
    local_readers: parseList(draft.localReaders),
    metadata_providers: parseList(draft.metadataProviders),
    image_providers: parseList(draft.imageProviders),
    language: nullableText(draft.language),
    country: nullableText(draft.country),
    refresh_mode: draft.refreshMode,
    local_metadata_policy: draft.localMetadataPolicy,
    scan: {
      enabled: draft.scanEnabled,
      addon_scrape: draft.addonScrape,
      addon_writeback: draft.addonWriteback,
    },
  };
}

function profileFormEquals(left: MetadataProfileForm, right: MetadataProfileForm) {
  return (Object.keys(left) as Array<keyof MetadataProfileForm>).every(
    (key) => left[key] === right[key],
  );
}

function parseList(value: string) {
  return value
    .split(/[,\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

function nullableText(value: string) {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
}

function formatJobTime(value: string) {
  return value.replace("T", " ").replace("Z", " UTC");
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("libraryDetail.sources.sizeUnknown");
  }

  if (value >= 1_073_741_824) {
    return `${(value / 1_073_741_824).toFixed(1)} GiB`;
  }

  if (value >= 1_048_576) {
    return `${(value / 1_048_576).toFixed(1)} MiB`;
  }

  return `${value} B`;
}

function sourceModeLabel(source: DataSourceMode, t: Translate) {
  if (source === "live") {
    return t("source.live");
  }

  if (source === "hybrid") {
    return t("source.hybrid");
  }

  if (source === "planned") {
    return t("source.planned");
  }

  return t("source.mock");
}

function errorMessage(error: unknown, t: Translate) {
  return error instanceof Error ? error.message : t("libraryDetail.commands.operationFailed");
}
