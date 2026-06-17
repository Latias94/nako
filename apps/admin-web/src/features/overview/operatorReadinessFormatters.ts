import type {
  AdminOperatorReadinessCheck,
  AdminOperatorReadinessResponse,
  AdminOperatorReadinessStatus,
} from "../../adminApi/types";
import type { MessageId } from "../../i18n/messages";

export type OperatorReadinessTranslate = (
  id: MessageId,
  values?: Record<string, number | string>,
) => string;

export type OperatorReadinessBadgeTone = "success" | "warning" | "danger";

type OperatorReadinessIntakeComponent =
  AdminOperatorReadinessResponse["details"]["media_library_scan"]["intake_action_plan"]["components"][number]["component"];

export const OPERATOR_READINESS_AREA_LABELS: Record<
  AdminOperatorReadinessCheck["area"],
  MessageId
> = {
  setup: "overview.operatorReadiness.area.setup",
  media_library_scan: "overview.operatorReadiness.area.mediaLibraryScan",
  playback: "overview.operatorReadiness.area.playback",
  durable_jobs: "overview.operatorReadiness.area.durableJobs",
  storage: "overview.operatorReadiness.area.storage",
  network: "overview.operatorReadiness.area.network",
  backup: "overview.operatorReadiness.area.backup",
};

const OPERATOR_READINESS_STATUS_LABELS: Record<
  AdminOperatorReadinessStatus,
  MessageId
> = {
  ready: "overview.operatorReadiness.status.ready",
  degraded: "overview.operatorReadiness.status.degraded",
  unavailable: "overview.operatorReadiness.status.unavailable",
};

const OPERATOR_READINESS_REASON_LABELS: Record<
  AdminOperatorReadinessCheck["reason"],
  MessageId
> = {
  auth_configured: "overview.operatorReadiness.reason.authConfigured",
  auth_token_reference_missing:
    "overview.operatorReadiness.reason.authTokenReferenceMissing",
  auth_disabled_local_only:
    "overview.operatorReadiness.reason.authDisabledLocalOnly",
  auth_disabled_remote_exposure:
    "overview.operatorReadiness.reason.authDisabledRemoteExposure",
  media_library_configured:
    "overview.operatorReadiness.reason.mediaLibraryConfigured",
  no_media_library_configured:
    "overview.operatorReadiness.reason.noMediaLibraryConfigured",
  scan_work_pending: "overview.operatorReadiness.reason.scanWorkPending",
  scan_repair_pressure: "overview.operatorReadiness.reason.scanRepairPressure",
  watch_folder_runtime_coverage_gap:
    "overview.operatorReadiness.reason.watchFolderRuntimeCoverageGap",
  playback_ready: "overview.operatorReadiness.reason.playbackReady",
  playback_degraded: "overview.operatorReadiness.reason.playbackDegraded",
  playback_unavailable: "overview.operatorReadiness.reason.playbackUnavailable",
  durable_jobs_ready: "overview.operatorReadiness.reason.durableJobsReady",
  durable_jobs_pressure:
    "overview.operatorReadiness.reason.durableJobsPressure",
  storage_ready: "overview.operatorReadiness.reason.storageReady",
  storage_degraded: "overview.operatorReadiness.reason.storageDegraded",
  storage_unavailable: "overview.operatorReadiness.reason.storageUnavailable",
  vfs_cache_repair_pressure:
    "overview.operatorReadiness.reason.vfsCacheRepairPressure",
  network_ready: "overview.operatorReadiness.reason.networkReady",
  network_degraded: "overview.operatorReadiness.reason.networkDegraded",
  network_unavailable: "overview.operatorReadiness.reason.networkUnavailable",
  backup_runbook_available:
    "overview.operatorReadiness.reason.backupRunbookAvailable",
  backup_needs_durable_database:
    "overview.operatorReadiness.reason.backupNeedsDurableDatabase",
};

const OPERATOR_READINESS_ACTION_LABELS: Partial<Record<string, MessageId>> = {
  jobs: "overview.operatorReadiness.action.jobs",
  playbackRuntime: "overview.operatorReadiness.action.playbackRuntime",
  storageVfsCacheRepairTargets:
    "overview.operatorReadiness.action.storageRepair",
  systemConfig: "overview.operatorReadiness.action.systemConfig",
};

const OPERATOR_READINESS_INTAKE_COMPONENT_LABELS: Record<
  OperatorReadinessIntakeComponent,
  MessageId
> = {
  library_scan: "operatorReadiness.scan.intakeComponent.libraryScan",
  source_fingerprint_hash:
    "operatorReadiness.scan.intakeComponent.sourceFingerprintHash",
  watch_folder: "operatorReadiness.scan.intakeComponent.watchFolder",
};

export function operatorReadinessAreaLabel(
  check: Pick<AdminOperatorReadinessCheck, "area">,
  t: OperatorReadinessTranslate,
) {
  return t(OPERATOR_READINESS_AREA_LABELS[check.area]);
}

export function operatorReadinessStatusLabel(
  status: AdminOperatorReadinessStatus,
  t: OperatorReadinessTranslate,
) {
  return t(OPERATOR_READINESS_STATUS_LABELS[status]);
}

export function operatorReadinessReasonLabel(
  check: Pick<AdminOperatorReadinessCheck, "attention_count" | "reason">,
  t: OperatorReadinessTranslate,
) {
  return t(OPERATOR_READINESS_REASON_LABELS[check.reason], {
    count: check.attention_count,
  });
}

export function operatorReadinessIntakeComponentLabel(
  component: OperatorReadinessIntakeComponent,
  t: OperatorReadinessTranslate,
) {
  return t(OPERATOR_READINESS_INTAKE_COMPONENT_LABELS[component]);
}

export function operatorReadinessActionLabel(
  routeKey: string,
  t: OperatorReadinessTranslate,
) {
  const label = OPERATOR_READINESS_ACTION_LABELS[routeKey];
  return label ? t(label) : routeKey;
}

export function operatorReadinessTone(
  status: AdminOperatorReadinessStatus,
): OperatorReadinessBadgeTone {
  if (status === "ready") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

export function safeOperatorReadinessSourceReason(
  value: string,
  t: OperatorReadinessTranslate,
) {
  if (/^[a-z0-9_.-]+$/.test(value)) {
    return value;
  }

  return t("overview.operatorReadiness.sourceReason.redacted");
}

export function safeOperatorReadinessDisplayValue(
  value: string,
  t: OperatorReadinessTranslate,
) {
  if (isUnsafeOperatorReadinessDisplayValue(value)) {
    return t("operatorReadiness.redacted");
  }

  return value;
}

function isUnsafeOperatorReadinessDisplayValue(value: string) {
  const normalized = value.trim();

  return (
    /^[a-z][a-z0-9+.-]*:\/\//iu.test(normalized) ||
    /[a-z]:[\\/]/iu.test(normalized) ||
    /(^|[\s"'])\/(users|home|var|tmp|mnt|media|srv|opt|etc)\//iu.test(
      normalized,
    ) ||
    /\?.*(token|secret|password|credential)=/iu.test(normalized) ||
    /(bearer|token|secret|password|credential|fingerprint|etag|input_json|summary_json)/iu.test(
      normalized,
    )
  );
}
