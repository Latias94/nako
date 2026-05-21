import { AdminApiClient, type AdminApiClientOptions } from "./client";
import {
  mockAdminConsoleData,
  mockAcquisitionIntakeCandidates,
  mockCatalogGovernance,
  mockEvents,
  mockJobs,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackSessions,
  mockStorageStaging,
  mockSystemConfig,
} from "./mockData";
import type {
  AdminAcquisitionIntakeCandidateListResponse,
  AdminCatalogGovernanceItemListResponse,
  AdminJobListResponse,
  AdminOutboxEventListResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
} from "./generated/contract";
import type {
  AdminConsoleData,
  AdminErrorMap,
  AdminSectionKey,
  AdminSourceMap,
  CatalogGovernanceSummary,
  DataSourceMode,
  EventSummary,
  IntakeSummary,
  JobRow,
  PlaybackSummary,
  SettingRow,
  StorageSummary,
} from "./types";

export type { AdminConsoleData, AdminSourceMap, DataSourceMode };

export type AdminDataSource = {
  load(): Promise<AdminConsoleData>;
};

type LoadResult<T> = {
  value: T;
  source: DataSourceMode;
  error?: string;
};

export function createAdminDataSource(options: AdminApiClientOptions = {}): AdminDataSource {
  const client = new AdminApiClient(options);

  return {
    async load() {
      const [
        overview,
        catalogGovernance,
        acquisitionIntakeCandidates,
        events,
        jobs,
        playbackSessions,
        playbackRuntime,
        storageStaging,
        systemConfig,
      ] = await Promise.all([
        loadSection(() => client.getOverview(), mockOverview),
        loadSection(() => client.getCatalogGovernanceItems(), mockCatalogGovernance),
        loadSection(() => client.getAcquisitionIntakeCandidates(), mockAcquisitionIntakeCandidates),
        loadSection(() => client.getEvents(), mockEvents),
        loadSection(() => client.getJobs(), mockJobs),
        loadSection(() => client.getPlaybackSessions(), mockPlaybackSessions),
        loadSection(() => client.getPlaybackRuntime(), mockPlaybackRuntime),
        loadSection(() => client.getStorageStaging(), mockStorageStaging),
        loadSection(() => client.getSystemConfig(), mockSystemConfig),
      ]);

      const sources: AdminSourceMap = {
        overview: overview.source,
        catalogGovernance: catalogGovernance.source,
        acquisitionIntake: acquisitionIntakeCandidates.source,
        events: events.source,
        jobs: jobs.source,
        playbackSessions: playbackSessions.source,
        playbackRuntime: playbackRuntime.source,
        storageStaging: storageStaging.source,
        systemConfig: systemConfig.source,
      };
      const errors: AdminErrorMap = {};

      recordError(errors, "overview", overview);
      recordError(errors, "catalogGovernance", catalogGovernance);
      recordError(errors, "acquisitionIntake", acquisitionIntakeCandidates);
      recordError(errors, "events", events);
      recordError(errors, "jobs", jobs);
      recordError(errors, "playbackSessions", playbackSessions);
      recordError(errors, "playbackRuntime", playbackRuntime);
      recordError(errors, "storageStaging", storageStaging);
      recordError(errors, "systemConfig", systemConfig);

      return {
        ...mockAdminConsoleData,
        sources,
        errors,
        overview: overview.value,
        catalog: mapCatalogGovernance(catalogGovernance.value),
        acquisitionIntake: mapAcquisitionIntake(acquisitionIntakeCandidates.value),
        events: mapEvents(events.value),
        jobs: mapJobs(jobs.value),
        playback: mapPlayback(playbackSessions.value, playbackRuntime.value),
        storage: mapStorage(storageStaging.value),
        settings: mapSettings(systemConfig.value),
      };
    },
  };
}

async function loadSection<T>(loader: () => Promise<T>, fallback: T): Promise<LoadResult<T>> {
  try {
    return {
      value: await loader(),
      source: "live",
    };
  } catch (error: unknown) {
    return {
      value: fallback,
      source: "mock",
      error: error instanceof Error ? error.message : "Admin API request failed",
    };
  }
}

function recordError<T>(errors: AdminErrorMap, section: AdminSectionKey, result: LoadResult<T>) {
  if (result.error) {
    errors[section] = result.error;
  }
}

function mapCatalogGovernance(
  response: AdminCatalogGovernanceItemListResponse,
): CatalogGovernanceSummary {
  return {
    items: response.items.map((item) => ({
      id: item.item_id,
      title: item.title,
      kind: item.kind,
      issues: item.issues,
      sourceCount: item.source_count,
      providerMappingCount: item.provider_mapping_count,
    })),
    page: response.page,
  };
}

function mapAcquisitionIntake(
  response: AdminAcquisitionIntakeCandidateListResponse,
): IntakeSummary {
  return {
    candidates: response.candidates.map((candidate) => ({
      id: candidate.id,
      sourceKind: candidate.source_kind,
      sourceScheme: candidate.source_scheme ?? "unknown",
      state: candidate.state,
      sizeBytes: candidate.size_bytes,
      hasDiagnostics: candidate.has_diagnostics,
      linkedArtifactId: candidate.managed_import_artifact_id,
    })),
    page: response.page,
  };
}

function mapEvents(response: AdminOutboxEventListResponse): EventSummary {
  return {
    events: response.events.map((event) => ({
      id: event.id,
      kind: event.kind,
      status: event.status,
      attempts: event.attempts,
      hasError: event.has_error,
    })),
    page: response.page,
  };
}

function mapJobs(response: AdminJobListResponse): JobRow[] {
  return response.jobs.map((job) => ({
    id: job.id,
    kind: job.kind,
    status: job.status,
    resourceClass: job.resource_class,
    hasError: job.has_error,
  }));
}

function mapPlayback(
  sessions: AdminPlaybackSessionListResponse,
  runtime: AdminPlaybackRuntimeDiagnosticsResponse,
): PlaybackSummary {
  return {
    hardwarePolicy: hardwarePolicyLabel(runtime),
    ffmpegStatus: runtime.ffmpeg.probe_status,
    accelerators: runtime.hardware.capabilities.map((capability) => ({
      name: capability.accelerator,
      available: capability.available,
    })),
    sessions: sessions.sessions.map((session) => ({
      id: session.id,
      kind: session.kind,
      sourceTitle: session.source_id,
      state: session.state,
    })),
  };
}

function mapStorage(response: AdminStorageStagingDiagnosticsResponse): StorageSummary {
  return {
    stagingUsedBytes: response.summary.used_manifest_bytes,
    stagingMaxBytes: response.summary.configured_max_bytes,
    vfsObjectCount: response.summary.vfs_cache.object_count,
    records: response.records.map((record) => ({
      id: record.id,
      sourceScheme: record.source_scheme,
      purpose: record.purpose,
      state: record.state,
      sizeBytes: record.size_bytes,
      hasValidationError: record.has_validation_error,
    })),
  };
}

function mapSettings(response: AdminServerConfigDiagnosticsResponse): SettingRow[] {
  return [
    {
      label: "Admin auth",
      value: response.auth.enabled ? "Auth configured" : "Auth disabled",
    },
    {
      label: "FFmpeg",
      value: "Runtime diagnostics enabled",
    },
    {
      label: "Transcode policy",
      value: `${response.transcode.gpu_concurrency} GPU slot`,
    },
    {
      label: "Settings edits",
      value: "Planned Admin API",
    },
  ];
}

function hardwarePolicyLabel(runtime: AdminPlaybackRuntimeDiagnosticsResponse) {
  const requested =
    typeof runtime.hardware.policy.requested === "string"
      ? runtime.hardware.policy.requested.toUpperCase()
      : "configured hardware";

  if (runtime.hardware.selection.fallback_used) {
    return `${requested} requested, fallback active`;
  }

  return `${requested} selected`;
}
