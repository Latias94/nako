import { RefreshCw, Save, ShieldCheck, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminMetadataRawCacheSettingsResponse,
  AdminPlaybackRuntimeSettingsPayload,
  AdminPlaybackRuntimeSettingsResponse,
  AdminServerConfigDiagnosticsResponse,
} from "../../adminApi/types";
import {
  mockMetadataRawCacheSettings,
  mockPlaybackRuntimeSettings,
  mockSystemConfig,
} from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type SettingsPageProps = {
  dataSource: AdminDataSource;
};

type SettingsResult = {
  value: AdminServerConfigDiagnosticsResponse;
  source: DataSourceMode;
  error?: string;
};
type MetadataRawCacheResult = {
  value: AdminMetadataRawCacheSettingsResponse;
  source: DataSourceMode;
  error?: string;
};
type PlaybackRuntimeSettingsResult = {
  value: AdminPlaybackRuntimeSettingsResponse;
  source: DataSourceMode;
  error?: string;
};
type MetadataRawCacheForm = {
  retentionMs: string;
  cleanupOnStartup: boolean;
};
type PlaybackRuntimeSettingsForm = {
  cpuConcurrency: string;
  gpuConcurrency: string;
  remuxConcurrency: string;
  remuxTimeoutMs: string;
  remoteStreamConcurrency: string;
  remoteStageConcurrency: string;
  stagingMaxBytes: string;
  stagingRetentionMs: string;
  stagingCleanupOnStartup: boolean;
  transcodeArtifactRetentionMs: string;
  transcodeArtifactCleanupOnStartup: boolean;
  hlsSegmentCleanupEnabled: boolean;
  hlsSegmentKeepMs: string;
  transcodeThrottleEnabled: boolean;
  transcodeThrottleDelayMs: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type MetadataProvider = AdminServerConfigDiagnosticsResponse["metadata"]["providers"][number];

export function SettingsPage({ dataSource }: SettingsPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-settings", locale],
    queryFn: () => loadSettings(dataSource, t("settings.dataSourceUnavailable")),
  });
  const metadataRawCacheQuery = useQuery({
    queryKey: ["admin-settings", "metadata-raw-cache", locale],
    queryFn: () =>
      loadMetadataRawCacheSettings(dataSource, t("settings.rawCache.dataSourceUnavailable")),
  });
  const playbackRuntimeSettingsQuery = useQuery({
    queryKey: ["admin-settings", "playback-runtime-settings", locale],
    queryFn: () =>
      loadPlaybackRuntimeSettings(
        dataSource,
        t("settings.playbackRuntime.dataSourceUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: mockSystemConfig,
    source: "mock" as const,
  };
  const metadataRawCacheResult = metadataRawCacheQuery.data ?? {
    value: mockMetadataRawCacheSettings,
    source: "mock" as const,
  };
  const playbackRuntimeSettingsResult = playbackRuntimeSettingsQuery.data ?? {
    value: mockPlaybackRuntimeSettings,
    source: "mock" as const,
  };
  const settings = result.value;
  const rawCacheSettings = metadataRawCacheResult.value;
  const playbackRuntimeSettings = playbackRuntimeSettingsResult.value;
  const enabledProviders = settings.metadata.providers.filter((provider) => provider.enabled).length;
  const [rawCacheDraft, setRawCacheDraft] = useState(() =>
    metadataRawCacheToForm(rawCacheSettings),
  );
  const [rawCacheEditing, setRawCacheEditing] = useState(false);
  const [rawCacheConfirming, setRawCacheConfirming] = useState(false);
  const [playbackRuntimeDraft, setPlaybackRuntimeDraft] = useState(() =>
    playbackRuntimeSettingsToForm(playbackRuntimeSettings.settings),
  );
  const [playbackRuntimeEditing, setPlaybackRuntimeEditing] = useState(false);
  const [playbackRuntimeConfirming, setPlaybackRuntimeConfirming] = useState(false);
  const rawCacheMutation = useMutation({
    mutationFn: async () => {
      if (metadataRawCacheResult.source !== "live") {
        throw new Error(t("settings.rawCache.notLiveError"));
      }
      if (!dataSource.updateMetadataRawCacheSettings) {
        throw new Error(t("settings.rawCache.mutationUnavailable"));
      }

      return dataSource.updateMetadataRawCacheSettings({
        retention_ms: parseRetentionMs(
          rawCacheDraft.retentionMs,
          t("settings.rawCache.invalidRetention"),
        ),
        cleanup_on_startup: rawCacheDraft.cleanupOnStartup,
      });
    },
    onSuccess: () => {
      setRawCacheEditing(false);
      setRawCacheConfirming(false);
      void metadataRawCacheQuery.refetch();
      void query.refetch();
    },
  });
  const playbackRuntimeMutation = useMutation({
    mutationFn: async () => {
      if (playbackRuntimeSettingsResult.source !== "live") {
        throw new Error(t("settings.playbackRuntime.notLiveError"));
      }
      if (!dataSource.updatePlaybackRuntimeSettings) {
        throw new Error(t("settings.playbackRuntime.mutationUnavailable"));
      }

      return dataSource.updatePlaybackRuntimeSettings({
        settings: playbackRuntimeFormToPayload(
          playbackRuntimeDraft,
          playbackRuntimeSettings.settings,
          t("settings.playbackRuntime.invalidPositiveInteger"),
        ),
      });
    },
    onSuccess: () => {
      setPlaybackRuntimeEditing(false);
      setPlaybackRuntimeConfirming(false);
      void playbackRuntimeSettingsQuery.refetch();
      void query.refetch();
    },
  });

  useEffect(() => {
    if (!rawCacheEditing) {
      const next = metadataRawCacheToForm(rawCacheSettings);
      setRawCacheDraft((current) => (rawCacheFormEquals(current, next) ? current : next));
    }
  }, [rawCacheEditing, rawCacheSettings]);

  useEffect(() => {
    if (!playbackRuntimeEditing) {
      const next = playbackRuntimeSettingsToForm(playbackRuntimeSettings.settings);
      setPlaybackRuntimeDraft((current) =>
        playbackRuntimeFormEquals(current, next) ? current : next,
      );
    }
  }, [playbackRuntimeEditing, playbackRuntimeSettings]);

  const rawCacheCanSave = rawCacheEditing && rawCacheFormIsValid(rawCacheDraft) && metadataRawCacheResult.source === "live";
  const playbackRuntimeCanSave =
    playbackRuntimeEditing &&
    playbackRuntimeFormIsValid(playbackRuntimeDraft) &&
    playbackRuntimeSettingsResult.source === "live";

  return (
    <RoutePage
      actions={
        <Button
          disabled={
            query.isFetching ||
            metadataRawCacheQuery.isFetching ||
            playbackRuntimeSettingsQuery.isFetching
          }
          onClick={() => {
            void query.refetch();
            void metadataRawCacheQuery.refetch();
            void playbackRuntimeSettingsQuery.refetch();
          }}
          variant="outline"
        >
          <RefreshCw size={16} />
          {t("settings.refresh")}
        </Button>
      }
      description={t("settings.description")}
      kicker={t("settings.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("settings.title")}
      titleId="settings-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("settings.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("settings.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="settingsSummaryGrid">
            <SummaryCard
              badge={
                settings.auth.enabled
                  ? t("settings.summary.auth.configured")
                  : t("settings.summary.auth.open")
              }
              label={t("settings.summary.auth.label")}
              tone={settings.auth.enabled ? "success" : "warning"}
              value={
                settings.auth.enabled
                  ? t("settings.summary.auth.enabled")
                  : t("settings.summary.auth.disabled")
              }
            />
            <SummaryCard
              badge={settings.network.readiness.reason}
              label={t("settings.summary.network.label")}
              tone={readinessTone(settings.network.readiness.status)}
              value={settings.network.readiness.status}
            />
            <SummaryCard
              badge={
                settings.database.migrated_on_startup
                  ? t("settings.summary.database.migrated")
                  : t("settings.summary.database.pending")
              }
              label={t("settings.summary.database.label")}
              tone={settings.database.runtime_supported ? "success" : "danger"}
              value={settings.database.active_backend_kind}
            />
            <SummaryCard
              badge={t("settings.summary.metadata.enabled", { count: enabledProviders })}
              label={t("settings.summary.metadata.label")}
              tone={enabledProviders > 0 ? "success" : "warning"}
              value={`${enabledProviders}/${settings.metadata.providers.length}`}
            />
            <SummaryCard
              badge={
                settings.transcode.gpu_concurrency > 0
                  ? t("settings.summary.transcode.gpuSlots")
                  : t("settings.summary.transcode.cpuOnly")
              }
              label={t("settings.summary.transcode.label")}
              tone={settings.transcode.gpu_concurrency > 0 ? "info" : "neutral"}
              value={t("settings.summary.transcode.value", {
                cpu: settings.transcode.cpu_concurrency,
                gpu: settings.transcode.gpu_concurrency,
              })}
            />
            <SummaryCard
              badge={
                settings.staging.cleanup_on_startup
                  ? t("settings.summary.staging.startupCleanup")
                  : t("settings.summary.staging.manualCleanup")
              }
              label={t("settings.summary.staging.label")}
              tone="neutral"
              value={formatBytes(settings.staging.max_bytes)}
            />
          </div>

          <div className="settingsPanelGrid">
            <DataPanel
              description={t("settings.network.description")}
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  {t("settings.network.redacted")}
                </div>
              }
              title={t("settings.network.title")}
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={settings.network.readiness.status}
                  label={t("settings.network.exposureMode")}
                  tone={readinessTone(settings.network.readiness.status)}
                  value={settings.network.exposure_mode}
                />
                <DiagnosticRow
                  badge={
                    settings.network.external_endpoint.configured
                      ? t("settings.network.configured")
                      : t("settings.network.notConfigured")
                  }
                  detail={settings.network.external_endpoint.scheme ?? t("settings.network.noScheme")}
                  label={t("settings.network.externalEndpoint")}
                  tone={settings.network.external_endpoint.configured ? "success" : "neutral"}
                  value={
                    settings.network.external_endpoint.configured
                      ? t("settings.network.endpointConfigured")
                      : t("settings.network.noEndpoint")
                  }
                />
                <DiagnosticRow
                  badge={
                    settings.network.trusted_proxy.headers_enabled
                      ? t("settings.network.trusted")
                      : t("settings.network.defaultDeny")
                  }
                  detail={t("settings.network.proxySources", {
                    count: settings.network.trusted_proxy.source_count,
                  })}
                  label={t("settings.network.trustedProxy")}
                  tone={settings.network.trusted_proxy.headers_enabled ? "info" : "neutral"}
                  value={t("settings.network.forwardedHeaders")}
                />
                <DiagnosticRow
                  badge={t("settings.network.tunnelProviders", {
                    count: settings.network.tunnel_providers.length,
                  })}
                  detail={t("settings.network.browserOrigins", {
                    count: settings.network.origins.allowed_origin_count,
                  })}
                  label={t("settings.network.browserAccess")}
                  tone={settings.network.origins.configured ? "success" : "warning"}
                  value={
                    settings.network.origins.configured
                      ? t("settings.network.originConfigured")
                      : t("settings.network.originMissing")
                  }
                />
              </div>
            </DataPanel>

            <DataPanel
              description={t("settings.database.description")}
              title={t("settings.database.title")}
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={
                    settings.database.runtime_supported
                      ? t("settings.database.supported")
                      : t("settings.database.unsupported")
                  }
                  label={t("settings.database.activeBackend")}
                  tone={settings.database.runtime_supported ? "success" : "danger"}
                  value={settings.database.active_backend_kind}
                />
                <DiagnosticRow
                  badge={
                    settings.database.migrated_on_startup
                      ? t("settings.database.migrated")
                      : t("settings.database.notMigrated")
                  }
                  detail={t("settings.database.configuredAs", {
                    backend: settings.database.configured_backend_kind,
                  })}
                  label={t("settings.database.startupMigration")}
                  tone={settings.database.migrated_on_startup ? "success" : "warning"}
                  value={settings.database.url_scheme}
                />
                <DiagnosticRow
                  badge={t("settings.database.enabled", {
                    count: enabledCapabilityCount(settings),
                  })}
                  detail={t("settings.database.capabilityDetail")}
                  label={t("settings.database.capabilities")}
                  tone="info"
                  value={`${enabledCapabilityCount(settings)}/${Object.keys(settings.database.capabilities).length}`}
                />
              </div>
            </DataPanel>

            <DataPanel
              description={t("settings.metadata.description")}
              title={t("settings.metadata.title")}
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={
                    settings.metadata.raw_cache_cleanup_on_startup
                      ? t("settings.metadata.startupCleanup")
                      : t("settings.metadata.scheduledCleanup")
                  }
                  detail={t("settings.metadata.cleanupEvery", {
                    duration: formatDuration(settings.metadata.raw_cache_cleanup_interval_ms),
                  })}
                  label={t("settings.metadata.rawCache")}
                  tone="neutral"
                  value={formatDuration(settings.metadata.raw_cache_retention_ms)}
                />
                <DiagnosticRow
                  badge={t("settings.metadata.policies", {
                    count: settings.metadata.maintenance_policies,
                  })}
                  detail={t("settings.metadata.concurrentRequests", {
                    count: settings.metadata.runtime.concurrency,
                  })}
                  label={t("settings.metadata.runtime")}
                  tone="info"
                  value={t("settings.metadata.timeout", {
                    duration: formatDuration(settings.metadata.runtime.timeout_ms),
                  })}
                />
              </div>
              <MetadataRawCacheEditor
                canSave={rawCacheCanSave}
                draft={rawCacheDraft}
                isConfirming={rawCacheConfirming}
                isEditing={rawCacheEditing}
                isPending={rawCacheMutation.isPending}
                result={metadataRawCacheResult}
                settings={rawCacheSettings}
                t={t}
                onCancel={() => {
                  setRawCacheDraft(metadataRawCacheToForm(rawCacheSettings));
                  setRawCacheEditing(false);
                  setRawCacheConfirming(false);
                  rawCacheMutation.reset();
                }}
                onConfirm={() => rawCacheMutation.mutate()}
                onDraftChange={setRawCacheDraft}
                onEdit={() => {
                  setRawCacheEditing(true);
                  setRawCacheConfirming(false);
                  rawCacheMutation.reset();
                }}
                onPrepare={() => setRawCacheConfirming(true)}
              />
              {metadataRawCacheResult.error ? (
                <div className="settingsInlineNotice">
                  {t("settings.fallback", { error: metadataRawCacheResult.error })}
                </div>
              ) : null}
              {rawCacheMutation.error ? (
                <div className="settingsInlineNotice danger">
                  {(rawCacheMutation.error as Error).message}
                </div>
              ) : null}
              {rawCacheMutation.data ? (
                <div className="settingsInlineNotice success">
                  {t("settings.metadata.rawCacheSaved", {
                    effect: rawCacheMutation.data.effect,
                  })}
                </div>
              ) : null}
              <div className="settingsProviderList">
                {settings.metadata.providers.map((provider) => (
                  <ProviderRow key={provider.provider} provider={provider} t={t} />
                ))}
              </div>
            </DataPanel>

            <DataPanel
              description={t("settings.runtime.description")}
              title={t("settings.runtime.title")}
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={t("settings.runtime.scanWorkers", {
                    count: settings.runtime.scan_concurrency,
                  })}
                  detail={t("settings.runtime.probeMetadata", {
                    probe: settings.runtime.probe_concurrency,
                    metadata: settings.runtime.metadata_concurrency,
                  })}
                  label={t("settings.runtime.libraryWorkers")}
                  tone="info"
                  value={t("settings.runtime.webhookWorkers", {
                    count: settings.runtime.webhook_concurrency,
                  })}
                />
                <DiagnosticRow
                  badge={t("settings.runtime.streams", {
                    count: settings.playback.remote_stream_concurrency,
                  })}
                  detail={t("settings.runtime.remoteStageWorkers", {
                    count: settings.playback.remote_stage_concurrency,
                  })}
                  label={t("settings.runtime.playback")}
                  tone="neutral"
                  value={t("settings.runtime.remuxWorker", {
                    count: settings.runtime.remux_concurrency,
                  })}
                />
                <DiagnosticRow
                  badge={
                    settings.staging.cleanup_on_startup
                      ? t("settings.runtime.cleanupEnabled")
                      : t("settings.runtime.cleanupDisabled")
                  }
                  detail={t("settings.runtime.retention", {
                    duration: formatDuration(settings.staging.retention_ms),
                  })}
                  label={t("settings.runtime.staging")}
                  tone={settings.staging.cleanup_on_startup ? "success" : "warning"}
                  value={formatBytes(settings.staging.max_bytes)}
                />
                <DiagnosticRow
                  badge={
                    settings.artwork.ingest_worker_enabled
                      ? t("settings.runtime.workerEnabled")
                      : t("settings.runtime.workerDisabled")
                  }
                  detail={t("settings.runtime.fetchWorkers", {
                    count: settings.artwork.fetch_concurrency,
                  })}
                  label={t("settings.runtime.artwork")}
                  tone={settings.artwork.ingest_worker_enabled ? "success" : "warning"}
                  value={`${settings.artwork.max_width} x ${settings.artwork.max_height}`}
                />
              </div>
              <PlaybackRuntimeSettingsEditor
                canSave={playbackRuntimeCanSave}
                draft={playbackRuntimeDraft}
                isConfirming={playbackRuntimeConfirming}
                isEditing={playbackRuntimeEditing}
                isPending={playbackRuntimeMutation.isPending}
                result={playbackRuntimeSettingsResult}
                settings={playbackRuntimeSettings}
                t={t}
                onCancel={() => {
                  setPlaybackRuntimeDraft(
                    playbackRuntimeSettingsToForm(playbackRuntimeSettings.settings),
                  );
                  setPlaybackRuntimeEditing(false);
                  setPlaybackRuntimeConfirming(false);
                  playbackRuntimeMutation.reset();
                }}
                onConfirm={() => playbackRuntimeMutation.mutate()}
                onDraftChange={setPlaybackRuntimeDraft}
                onEdit={() => {
                  setPlaybackRuntimeEditing(true);
                  setPlaybackRuntimeConfirming(false);
                  playbackRuntimeMutation.reset();
                }}
                onPrepare={() => setPlaybackRuntimeConfirming(true)}
              />
              {playbackRuntimeSettingsResult.error ? (
                <div className="settingsInlineNotice">
                  {t("settings.fallback", { error: playbackRuntimeSettingsResult.error })}
                </div>
              ) : null}
              {playbackRuntimeMutation.error ? (
                <div className="settingsInlineNotice danger">
                  {(playbackRuntimeMutation.error as Error).message}
                </div>
              ) : null}
              {playbackRuntimeMutation.data ? (
                <div className="settingsInlineNotice success">
                  {t("settings.playbackRuntime.saved", {
                    effect: playbackRuntimeMutation.data.effect,
                  })}
                </div>
              ) : null}
            </DataPanel>
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadSettings(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<SettingsResult> {
  if (!dataSource.loadSettings) {
    return {
      value: mockSystemConfig,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadSettings();
}

async function loadMetadataRawCacheSettings(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<MetadataRawCacheResult> {
  if (!dataSource.loadMetadataRawCacheSettings) {
    return {
      value: mockMetadataRawCacheSettings,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadMetadataRawCacheSettings();
}

async function loadPlaybackRuntimeSettings(
  dataSource: AdminDataSource,
  missingDataSourceMessage: string,
): Promise<PlaybackRuntimeSettingsResult> {
  if (!dataSource.loadPlaybackRuntimeSettings) {
    return {
      value: mockPlaybackRuntimeSettings,
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadPlaybackRuntimeSettings();
}

function MetadataRawCacheEditor({
  canSave,
  draft,
  isConfirming,
  isEditing,
  isPending,
  result,
  settings,
  t,
  onCancel,
  onConfirm,
  onDraftChange,
  onEdit,
  onPrepare,
}: {
  canSave: boolean;
  draft: MetadataRawCacheForm;
  isConfirming: boolean;
  isEditing: boolean;
  isPending: boolean;
  result: MetadataRawCacheResult;
  settings: AdminMetadataRawCacheSettingsResponse;
  t: Translate;
  onCancel: () => void;
  onConfirm: () => void;
  onDraftChange: (draft: MetadataRawCacheForm) => void;
  onEdit: () => void;
  onPrepare: () => void;
}) {
  const sourceTone = settings.source === "admin" ? "info" : "neutral";
  const effectTone = settings.effect === "requires_restart" ? "warning" : "success";

  return (
    <div className="settingsMutationPanel">
      <div className="settingsMutationHeader">
        <div>
          <strong>{t("settings.rawCache.title")}</strong>
          <span>{t("settings.rawCache.description")}</span>
        </div>
        <div className="settingsMutationBadges">
          <Badge tone={sourceTone}>{settings.source}</Badge>
          <Badge tone={effectTone}>{settings.effect}</Badge>
        </div>
      </div>

      <div className="settingsMutationFields">
        <label>
          {t("settings.rawCache.retentionMs")}
          <input
            disabled={!isEditing || isPending}
            inputMode="numeric"
            min={1}
            onChange={(event) =>
              onDraftChange({ ...draft, retentionMs: event.currentTarget.value })
            }
            type="number"
            value={draft.retentionMs}
          />
        </label>
        <label className="settingsToggleField">
          <input
            checked={draft.cleanupOnStartup}
            disabled={!isEditing || isPending}
            onChange={(event) =>
              onDraftChange({ ...draft, cleanupOnStartup: event.currentTarget.checked })
            }
            type="checkbox"
          />
          {t("settings.rawCache.cleanupOnStartup")}
        </label>
      </div>

      <div className="settingsMutationFacts">
        <span>
          {t("settings.rawCache.activeValue", {
            duration: formatDuration(settings.retention_ms),
          })}
        </span>
        <span>
          {settings.cleanup_on_startup
            ? t("settings.rawCache.cleanupEnabled")
            : t("settings.rawCache.cleanupDisabled")}
        </span>
        <span>
          {settings.updated_at_ms
            ? t("settings.rawCache.updated", { updatedAt: settings.updated_at_ms })
            : t("settings.rawCache.noAdminUpdate")}
        </span>
      </div>

      <div className="settingsMutationActions">
        {!isEditing ? (
          <Button
            disabled={result.source !== "live" || isPending}
            onClick={onEdit}
            size="sm"
            variant="outline"
          >
            <Save size={14} />
            {t("settings.rawCache.editOverride")}
          </Button>
        ) : null}
        {isEditing && !isConfirming ? (
          <>
            <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
              <X size={14} />
              {t("settings.rawCache.cancel")}
            </Button>
            <Button disabled={!canSave || isPending} onClick={onPrepare} size="sm">
              <Save size={14} />
              {t("settings.rawCache.prepareSave")}
            </Button>
          </>
        ) : null}
        {isEditing && isConfirming ? (
          <>
            <span>{t("settings.rawCache.saveReplacement")}</span>
            <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
              {t("settings.rawCache.cancel")}
            </Button>
            <Button disabled={!canSave || isPending} onClick={onConfirm} size="sm">
              {isPending ? t("settings.rawCache.saving") : t("settings.rawCache.confirmSave")}
            </Button>
          </>
        ) : null}
      </div>
      {result.source !== "live" ? (
        <div className="settingsInlineNotice">
          {t("settings.rawCache.saveDisabled")}
        </div>
      ) : null}
    </div>
  );
}

function PlaybackRuntimeSettingsEditor({
  canSave,
  draft,
  isConfirming,
  isEditing,
  isPending,
  result,
  settings,
  t,
  onCancel,
  onConfirm,
  onDraftChange,
  onEdit,
  onPrepare,
}: {
  canSave: boolean;
  draft: PlaybackRuntimeSettingsForm;
  isConfirming: boolean;
  isEditing: boolean;
  isPending: boolean;
  result: PlaybackRuntimeSettingsResult;
  settings: AdminPlaybackRuntimeSettingsResponse;
  t: Translate;
  onCancel: () => void;
  onConfirm: () => void;
  onDraftChange: (draft: PlaybackRuntimeSettingsForm) => void;
  onEdit: () => void;
  onPrepare: () => void;
}) {
  const payload = settings.settings;
  const sourceTone = settings.source === "admin" ? "info" : "neutral";
  const effectTone = settings.effect === "requires_restart" ? "warning" : "success";
  const disabled = !isEditing || isPending;

  return (
    <div className="settingsMutationPanel">
      <div className="settingsMutationHeader">
        <div>
          <strong>{t("settings.playbackRuntime.title")}</strong>
          <span>{t("settings.playbackRuntime.description")}</span>
        </div>
        <div className="settingsMutationBadges">
          <Badge tone={sourceTone}>{settings.source}</Badge>
          <Badge tone={effectTone}>{settings.effect}</Badge>
        </div>
      </div>

      <div className="settingsMutationFacts">
        <span>
          {t("settings.playbackRuntime.hardwarePolicy", {
            requested: safePlaybackRuntimePolicyValue(payload.hardware_acceleration),
            fallback: safePlaybackRuntimePolicyValue(payload.hardware_fallback),
          })}
        </span>
        <span>
          {t("settings.playbackRuntime.activeConcurrency", {
            cpu: payload.cpu_concurrency,
            gpu: payload.gpu_concurrency,
            remux: payload.remux_concurrency,
          })}
        </span>
        <span>
          {t("settings.playbackRuntime.activeStaging", {
            size: formatBytes(payload.staging_max_bytes),
            retention: formatDuration(payload.staging_retention_ms),
          })}
        </span>
        <span>
          {settings.updated_at_ms
            ? t("settings.playbackRuntime.updated", { updatedAt: settings.updated_at_ms })
            : t("settings.playbackRuntime.noAdminUpdate")}
        </span>
      </div>

      <div className="settingsMutationFields">
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.cpuConcurrency")}
          value={draft.cpuConcurrency}
          onChange={(value) => onDraftChange({ ...draft, cpuConcurrency: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.gpuConcurrency")}
          value={draft.gpuConcurrency}
          onChange={(value) => onDraftChange({ ...draft, gpuConcurrency: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.remuxConcurrency")}
          value={draft.remuxConcurrency}
          onChange={(value) => onDraftChange({ ...draft, remuxConcurrency: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.remuxTimeoutMs")}
          value={draft.remuxTimeoutMs}
          onChange={(value) => onDraftChange({ ...draft, remuxTimeoutMs: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.remoteStreamConcurrency")}
          value={draft.remoteStreamConcurrency}
          onChange={(value) => onDraftChange({ ...draft, remoteStreamConcurrency: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.remoteStageConcurrency")}
          value={draft.remoteStageConcurrency}
          onChange={(value) => onDraftChange({ ...draft, remoteStageConcurrency: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.stagingMaxBytes")}
          value={draft.stagingMaxBytes}
          onChange={(value) => onDraftChange({ ...draft, stagingMaxBytes: value })}
        />
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.stagingRetentionMs")}
          value={draft.stagingRetentionMs}
          onChange={(value) => onDraftChange({ ...draft, stagingRetentionMs: value })}
        />
        <label className="settingsToggleField">
          <input
            checked={draft.stagingCleanupOnStartup}
            disabled={disabled}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                stagingCleanupOnStartup: event.currentTarget.checked,
              })
            }
            type="checkbox"
          />
          {t("settings.playbackRuntime.stagingCleanupOnStartup")}
        </label>
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.transcodeArtifactRetentionMs")}
          value={draft.transcodeArtifactRetentionMs}
          onChange={(value) =>
            onDraftChange({ ...draft, transcodeArtifactRetentionMs: value })
          }
        />
        <label className="settingsToggleField">
          <input
            checked={draft.transcodeArtifactCleanupOnStartup}
            disabled={disabled}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                transcodeArtifactCleanupOnStartup: event.currentTarget.checked,
              })
            }
            type="checkbox"
          />
          {t("settings.playbackRuntime.transcodeArtifactCleanupOnStartup")}
        </label>
        <label className="settingsToggleField">
          <input
            checked={draft.hlsSegmentCleanupEnabled}
            disabled={disabled}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                hlsSegmentCleanupEnabled: event.currentTarget.checked,
              })
            }
            type="checkbox"
          />
          {t("settings.playbackRuntime.hlsSegmentCleanupEnabled")}
        </label>
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.hlsSegmentKeepMs")}
          value={draft.hlsSegmentKeepMs}
          onChange={(value) => onDraftChange({ ...draft, hlsSegmentKeepMs: value })}
        />
        <label className="settingsToggleField">
          <input
            checked={draft.transcodeThrottleEnabled}
            disabled={disabled}
            onChange={(event) =>
              onDraftChange({
                ...draft,
                transcodeThrottleEnabled: event.currentTarget.checked,
              })
            }
            type="checkbox"
          />
          {t("settings.playbackRuntime.transcodeThrottleEnabled")}
        </label>
        <PlaybackRuntimeNumberField
          disabled={disabled}
          label={t("settings.playbackRuntime.transcodeThrottleDelayMs")}
          value={draft.transcodeThrottleDelayMs}
          onChange={(value) =>
            onDraftChange({ ...draft, transcodeThrottleDelayMs: value })
          }
        />
      </div>

      <div className="settingsMutationActions">
        {!isEditing ? (
          <Button
            disabled={result.source !== "live" || isPending}
            onClick={onEdit}
            size="sm"
            variant="outline"
          >
            <Save size={14} />
            {t("settings.playbackRuntime.editRuntime")}
          </Button>
        ) : null}
        {isEditing && !isConfirming ? (
          <>
            <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
              <X size={14} />
              {t("settings.playbackRuntime.cancel")}
            </Button>
            <Button disabled={!canSave || isPending} onClick={onPrepare} size="sm">
              <Save size={14} />
              {t("settings.playbackRuntime.prepareSave")}
            </Button>
          </>
        ) : null}
        {isEditing && isConfirming ? (
          <>
            <span>{t("settings.playbackRuntime.saveReplacement")}</span>
            <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
              {t("settings.playbackRuntime.cancel")}
            </Button>
            <Button disabled={!canSave || isPending} onClick={onConfirm} size="sm">
              {isPending
                ? t("settings.playbackRuntime.saving")
                : t("settings.playbackRuntime.confirmSave")}
            </Button>
          </>
        ) : null}
      </div>
      {result.source !== "live" ? (
        <div className="settingsInlineNotice">
          {t("settings.playbackRuntime.saveDisabled")}
        </div>
      ) : null}
    </div>
  );
}

function PlaybackRuntimeNumberField({
  disabled,
  label,
  value,
  onChange,
}: {
  disabled: boolean;
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <label>
      {label}
      <input
        disabled={disabled}
        inputMode="numeric"
        min={1}
        onChange={(event) => onChange(event.currentTarget.value)}
        type="number"
        value={value}
      />
    </label>
  );
}

function SummaryCard({
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
    <div className="settingsSummaryCard">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function DiagnosticRow({
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
    <div className="settingsDiagnosticRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail ? <small>{detail}</small> : null}
      </div>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function ProviderRow({ provider, t }: { provider: MetadataProvider; t: Translate }) {
  return (
    <div className="settingsProviderRow">
      <div>
        <strong>{provider.provider.toUpperCase()}</strong>
        <span>{provider.language ?? t("settings.provider.defaultLanguage")}</span>
      </div>
      <Badge tone={provider.enabled ? "success" : "warning"}>
        {provider.enabled ? t("settings.provider.enabled") : t("settings.provider.disabled")}
      </Badge>
      <span>
        {provider.has_api_base_url
          ? t("settings.provider.apiBaseConfigured")
          : t("settings.provider.defaultApiBase")}
      </span>
      <span>
        {provider.has_image_base_url
          ? t("settings.provider.imageBaseConfigured")
          : t("settings.provider.defaultImageBase")}
      </span>
      <span>{t("settings.provider.secretHeaders", { count: provider.secret_header_count })}</span>
    </div>
  );
}

function metadataRawCacheToForm(
  settings: AdminMetadataRawCacheSettingsResponse,
): MetadataRawCacheForm {
  return {
    retentionMs: String(settings.retention_ms),
    cleanupOnStartup: settings.cleanup_on_startup,
  };
}

function rawCacheFormEquals(left: MetadataRawCacheForm, right: MetadataRawCacheForm) {
  return (
    left.retentionMs === right.retentionMs &&
    left.cleanupOnStartup === right.cleanupOnStartup
  );
}

function rawCacheFormIsValid(form: MetadataRawCacheForm) {
  return Number.isInteger(Number(form.retentionMs)) && Number(form.retentionMs) > 0;
}

function parseRetentionMs(value: string, invalidMessage: string) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(invalidMessage);
  }

  return parsed;
}

function playbackRuntimeSettingsToForm(
  settings: AdminPlaybackRuntimeSettingsPayload,
): PlaybackRuntimeSettingsForm {
  return {
    cpuConcurrency: String(settings.cpu_concurrency),
    gpuConcurrency: String(settings.gpu_concurrency),
    remuxConcurrency: String(settings.remux_concurrency),
    remuxTimeoutMs: String(settings.remux_timeout_ms),
    remoteStreamConcurrency: String(settings.remote_stream_concurrency),
    remoteStageConcurrency: String(settings.remote_stage_concurrency),
    stagingMaxBytes: String(settings.staging_max_bytes),
    stagingRetentionMs: String(settings.staging_retention_ms),
    stagingCleanupOnStartup: settings.staging_cleanup_on_startup,
    transcodeArtifactRetentionMs: String(settings.transcode_artifact_retention_ms),
    transcodeArtifactCleanupOnStartup: settings.transcode_artifact_cleanup_on_startup,
    hlsSegmentCleanupEnabled: settings.hls_segment_cleanup_enabled,
    hlsSegmentKeepMs: String(settings.hls_segment_keep_ms),
    transcodeThrottleEnabled: settings.transcode_throttle_enabled,
    transcodeThrottleDelayMs: String(settings.transcode_throttle_delay_ms),
  };
}

function playbackRuntimeFormToPayload(
  form: PlaybackRuntimeSettingsForm,
  current: AdminPlaybackRuntimeSettingsPayload,
  invalidMessage: string,
): AdminPlaybackRuntimeSettingsPayload {
  return {
    ...current,
    cpu_concurrency: parsePositiveInteger(form.cpuConcurrency, invalidMessage),
    gpu_concurrency: parsePositiveInteger(form.gpuConcurrency, invalidMessage),
    remux_concurrency: parsePositiveInteger(form.remuxConcurrency, invalidMessage),
    remux_timeout_ms: parsePositiveInteger(form.remuxTimeoutMs, invalidMessage),
    remote_stream_concurrency: parsePositiveInteger(
      form.remoteStreamConcurrency,
      invalidMessage,
    ),
    remote_stage_concurrency: parsePositiveInteger(form.remoteStageConcurrency, invalidMessage),
    staging_max_bytes: parsePositiveInteger(form.stagingMaxBytes, invalidMessage),
    staging_retention_ms: parsePositiveInteger(form.stagingRetentionMs, invalidMessage),
    staging_cleanup_on_startup: form.stagingCleanupOnStartup,
    transcode_artifact_retention_ms: parsePositiveInteger(
      form.transcodeArtifactRetentionMs,
      invalidMessage,
    ),
    transcode_artifact_cleanup_on_startup: form.transcodeArtifactCleanupOnStartup,
    hls_segment_cleanup_enabled: form.hlsSegmentCleanupEnabled,
    hls_segment_keep_ms: parsePositiveInteger(form.hlsSegmentKeepMs, invalidMessage),
    transcode_throttle_enabled: form.transcodeThrottleEnabled,
    transcode_throttle_delay_ms: parsePositiveInteger(
      form.transcodeThrottleDelayMs,
      invalidMessage,
    ),
  };
}

function playbackRuntimeFormEquals(
  left: PlaybackRuntimeSettingsForm,
  right: PlaybackRuntimeSettingsForm,
) {
  return (
    left.cpuConcurrency === right.cpuConcurrency &&
    left.gpuConcurrency === right.gpuConcurrency &&
    left.remuxConcurrency === right.remuxConcurrency &&
    left.remuxTimeoutMs === right.remuxTimeoutMs &&
    left.remoteStreamConcurrency === right.remoteStreamConcurrency &&
    left.remoteStageConcurrency === right.remoteStageConcurrency &&
    left.stagingMaxBytes === right.stagingMaxBytes &&
    left.stagingRetentionMs === right.stagingRetentionMs &&
    left.stagingCleanupOnStartup === right.stagingCleanupOnStartup &&
    left.transcodeArtifactRetentionMs === right.transcodeArtifactRetentionMs &&
    left.transcodeArtifactCleanupOnStartup === right.transcodeArtifactCleanupOnStartup &&
    left.hlsSegmentCleanupEnabled === right.hlsSegmentCleanupEnabled &&
    left.hlsSegmentKeepMs === right.hlsSegmentKeepMs &&
    left.transcodeThrottleEnabled === right.transcodeThrottleEnabled &&
    left.transcodeThrottleDelayMs === right.transcodeThrottleDelayMs
  );
}

function playbackRuntimeFormIsValid(form: PlaybackRuntimeSettingsForm) {
  return [
    form.cpuConcurrency,
    form.gpuConcurrency,
    form.remuxConcurrency,
    form.remuxTimeoutMs,
    form.remoteStreamConcurrency,
    form.remoteStageConcurrency,
    form.stagingMaxBytes,
    form.stagingRetentionMs,
    form.transcodeArtifactRetentionMs,
    form.hlsSegmentKeepMs,
    form.transcodeThrottleDelayMs,
  ].every(isPositiveIntegerString);
}

function parsePositiveInteger(value: string, invalidMessage: string) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(invalidMessage);
  }

  return parsed;
}

function isPositiveIntegerString(value: string) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0;
}

function safePlaybackRuntimePolicyValue(value: string) {
  return [
    "none",
    "cpu",
    "nvenc",
    "vaapi",
    "quick_sync",
    "video_toolbox",
    "auto",
  ].includes(value)
    ? value
    : "unknown";
}

function readinessTone(status: string): BadgeTone {
  if (status === "ready") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

function enabledCapabilityCount(settings: AdminServerConfigDiagnosticsResponse) {
  return Object.values(settings.database.capabilities).filter(Boolean).length;
}

function formatBytes(bytes: number) {
  if (bytes >= 1024 ** 3) {
    return `${Math.round(bytes / 1024 ** 3)} GiB`;
  }

  if (bytes >= 1024 ** 2) {
    return `${Math.round(bytes / 1024 ** 2)} MiB`;
  }

  if (bytes >= 1024) {
    return `${Math.round(bytes / 1024)} KiB`;
  }

  return `${bytes} B`;
}

function formatDuration(ms: number) {
  if (ms >= 24 * 60 * 60 * 1000) {
    return `${Math.round(ms / (24 * 60 * 60 * 1000))} d`;
  }

  if (ms >= 60 * 60 * 1000) {
    return `${Math.round(ms / (60 * 60 * 1000))} h`;
  }

  if (ms >= 1000) {
    return `${Math.round(ms / 1000)} s`;
  }

  return `${ms} ms`;
}

type Translate = (id: MessageId, values?: Record<string, boolean | number | string>) => string;
