import type { AdminPlaybackSupportEvidenceResponse } from "../../adminApi/types";

export type PlaybackSupportSession = NonNullable<AdminPlaybackSupportEvidenceResponse["session"]>;

export function sessionTone(state: string) {
  if (state === "failed") {
    return "danger" as const;
  }

  if (state === "running") {
    return "info" as const;
  }

  if (state === "starting") {
    return "warning" as const;
  }

  return "neutral" as const;
}

export function runtimeTone(status: string) {
  if (status === "ready") {
    return "success" as const;
  }

  if (status === "degraded") {
    return "warning" as const;
  }

  return "danger" as const;
}

export function boolText(value: boolean | null | undefined) {
  if (value === undefined || value === null) {
    return "n/a";
  }

  return value ? "yes" : "no";
}

export function formatNullableNumber(value: number | null | undefined) {
  return value === null || value === undefined ? "n/a" : String(value);
}

export function formatSessionMetrics(
  metrics: PlaybackSupportSession["runtime_metrics"] | null | undefined,
) {
  if (!metrics) {
    return "n/a";
  }

  return [
    `frames=${formatNullableNumber(metrics.frame_count)}`,
    `fps_millis=${formatNullableNumber(metrics.fps_millis)}`,
    `bitrate_kbps=${formatNullableNumber(metrics.bitrate_kbps)}`,
    `size_bytes=${formatNullableNumber(metrics.total_size_bytes)}`,
    `output_time_ms=${formatNullableNumber(metrics.output_time_ms)}`,
    `dup_frames=${formatNullableNumber(metrics.dup_frames)}`,
    `drop_frames=${formatNullableNumber(metrics.drop_frames)}`,
    `speed_millis=${formatNullableNumber(metrics.speed_millis)}`,
    `progress=${metrics.progress ?? "n/a"}`,
  ].join(" / ");
}

export function formatSessionTiming(session: PlaybackSupportSession | null) {
  if (!session) {
    return "n/a";
  }

  return [
    `created=${session.created_at}`,
    `updated=${session.updated_at}`,
    `started=${session.started_at ?? "n/a"}`,
    `completed=${session.completed_at ?? "n/a"}`,
  ].join(" / ");
}

export function formatReadiness(status: string, reason: string) {
  return `${status} / ${reason}`;
}

export function formatPolicy(policy: AdminPlaybackSupportEvidenceResponse["runtime"]["policy"]) {
  return [
    `user_policy_rows_supported=${boolText(policy.user_policy_rows_supported)}`,
    `role_policy_rows_supported=${boolText(policy.role_policy_rows_supported)}`,
    `effective_resolution_supported=${boolText(policy.effective_resolution_supported)}`,
    `library_access_required=${boolText(policy.library_access_required)}`,
    `user_policy_overrides_role_policy=${boolText(policy.user_policy_overrides_role_policy)}`,
    `role_policy_merge=${policy.role_policy_merge}`,
    `permissions=${policy.permissions.join(", ") || "n/a"}`,
  ].join(" / ");
}

export function formatFfmpeg(ffmpeg: AdminPlaybackSupportEvidenceResponse["runtime"]["ffmpeg"]) {
  return [
    `probe_status=${ffmpeg.probe_status}`,
    `has_probe_error=${boolText(ffmpeg.has_probe_error)}`,
    `hardware_capability_count=${ffmpeg.hardware_capability_count}`,
    `available_gpu_capabilities=${ffmpeg.available_gpu_capabilities}`,
  ].join(" / ");
}

export function formatHardware(hardware: AdminPlaybackSupportEvidenceResponse["runtime"]["hardware"]) {
  return [
    `selected_acceleration=${hardware.selected_acceleration}`,
    `fallback_used=${boolText(hardware.fallback_used)}`,
    `capability_count=${hardware.capability_count}`,
    `unavailable_capabilities=${hardware.unavailable_capabilities.length}`,
  ].join(" / ");
}

export function formatTranscode(transcode: AdminPlaybackSupportEvidenceResponse["runtime"]["transcode"]) {
  return [
    `configured_cpu_slots=${transcode.configured_cpu_slots}`,
    `configured_gpu_slots=${transcode.configured_gpu_slots}`,
    `effective_cpu_slots=${transcode.effective_cpu_slots}`,
    `effective_gpu_slots=${transcode.effective_gpu_slots}`,
    `selected_hls_slots=${transcode.selected_hls_slots}`,
  ].join(" / ");
}

export function formatRemux(remux: AdminPlaybackSupportEvidenceResponse["runtime"]["remux"]) {
  return [
    `max_concurrent_sessions=${remux.max_concurrent_sessions}`,
    `timeout_ms=${remux.timeout_ms}`,
  ].join(" / ");
}

export function formatRemotePlayback(
  remotePlayback: AdminPlaybackSupportEvidenceResponse["runtime"]["remote_playback"],
) {
  return [
    `backend_count=${remotePlayback.backend_count}`,
    `stream_permits=${remotePlayback.stream_permits_available}/${remotePlayback.stream_permits_max}`,
    `stage_permits=${remotePlayback.stage_permits_available}/${remotePlayback.stage_permits_max}`,
    `state_scope=${remotePlayback.state_scope}`,
  ].join(" / ");
}

export function formatStaging(staging: AdminPlaybackSupportEvidenceResponse["runtime"]["staging"]) {
  return [
    `max_bytes=${staging.max_bytes}`,
    `retention_ms=${staging.retention_ms}`,
    `cleanup_on_startup=${boolText(staging.cleanup_on_startup)}`,
    `startup_deleted_records=${staging.startup_deleted_records}`,
    `startup_deleted_files=${staging.startup_deleted_files}`,
  ].join(" / ");
}

export function formatArtifactLifecycle(
  artifactLifecycle: AdminPlaybackSupportEvidenceResponse["runtime"]["artifact_lifecycle"],
) {
  return [
    `transcode_artifact_retention_ms=${artifactLifecycle.transcode_artifact_retention_ms}`,
    `transcode_artifact_cleanup_on_startup=${boolText(
      artifactLifecycle.transcode_artifact_cleanup_on_startup,
    )}`,
    `hls_segment_cleanup_enabled=${boolText(artifactLifecycle.hls_segment_cleanup_enabled)}`,
    `hls_segment_keep_ms=${artifactLifecycle.hls_segment_keep_ms}`,
    `deleted_artifacts=${artifactLifecycle.startup_deleted_artifacts}`,
    `deleted_bytes=${artifactLifecycle.startup_deleted_bytes}`,
    `deleted_directories=${artifactLifecycle.startup_deleted_directories}`,
    `deleted_files=${artifactLifecycle.startup_deleted_files}`,
    `examined=${artifactLifecycle.startup_examined_artifacts}`,
    `skipped_security=${artifactLifecycle.startup_skipped_security}`,
  ].join(" / ");
}

export function formatThrottle(throttle: AdminPlaybackSupportEvidenceResponse["runtime"]["throttle"]) {
  return [`enabled=${boolText(throttle.enabled)}`, `delay_ms=${throttle.delay_ms}`].join(" / ");
}

export function allRedactionPassed(redaction: AdminPlaybackSupportEvidenceResponse["redaction"]) {
  return (
    redaction.paths_redacted &&
    redaction.source_references_redacted &&
    redaction.ffmpeg_commands_redacted &&
    redaction.stderr_redacted &&
    redaction.credentials_redacted
  );
}
