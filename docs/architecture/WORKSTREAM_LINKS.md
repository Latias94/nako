# Architecture Workstream Links

Last updated: 2026-05-29

This index links architecture capability areas to workstream evidence and
candidate follow-on lanes. The top-level architecture map stays concise; this
file is the cross-reference for agents that need execution history.

Use this index as navigation, not as the source of design truth:

- ADRs own durable decisions.
- `docs/architecture/*.md` owns capability progress and risk maps.
- `docs/workstreams/*` owns task plans, evidence, gates, and handoff state.

## Linkage Rules

- A shipped or partial capability should link at least one concrete workstream
  when evidence exists.
- A future lane should use a proposed slug until a real workstream directory is
  opened, for example `proposed:hls-seek-restart-lifecycle`.
- New workstreams should add `architecture_refs` and `capability_tags` to
  `WORKSTREAM.json` when they materially change an architecture capability.
- Do not copy detailed workstream evidence into architecture docs. Link the
  workstream instead.

Recommended `WORKSTREAM.json` fields:

```json
{
  "architecture_refs": [
    {
      "path": "docs/architecture/PLAYBACK.md",
      "capability": "HLS seek/restart"
    }
  ],
  "capability_tags": ["playback.hls.seek"]
}
```

## Playback And Transcode

Primary map: `docs/architecture/PLAYBACK.md`

Evidence workstreams:

- `docs/workstreams/playback-streaming/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/playback-capability-profile-and-rendition-planning/`
- `docs/workstreams/playback-api-transcode-boundary-cleanup/`
- `docs/workstreams/playback-planner-transcode-value-vocabulary/`
- `docs/workstreams/playback-planner-transcode-seam-deepening/`
- `docs/workstreams/playback-runtime-boundary-deepening/`
- `docs/workstreams/playback-transcode-policy-deepening/`
- `docs/workstreams/playback-transcode-ops-hardening/`
- `docs/workstreams/source-aware-transcode-runtime/`
- `docs/workstreams/transcode-runtime/`
- `docs/workstreams/transcode-output-shape-hls-manifest-ladder/`
- `docs/workstreams/executable-hls-fmp4-runtime-boundary/`
- `docs/workstreams/adaptive-hls-source-aware-ladder/`
- `docs/workstreams/hls-master-renditions-authoring/`
- `docs/workstreams/hls-media-renditions-runtime/`
- `docs/workstreams/hls-alternate-audio-renditions/`
- `docs/workstreams/hls-audio-sidecar-artifacts/`
- `docs/workstreams/hls-selected-main-audio-cleanup/`
- `docs/workstreams/playback-audio-language-default-policy/`
- `docs/workstreams/hls-seek-restart-lifecycle/`
- `docs/workstreams/hls-progressive-runtime-boundary/`
- `docs/workstreams/playback-runtime-resource-scheduler/`
- `docs/workstreams/admin-playback-runtime-diagnostics/`
- `docs/workstreams/admin-playback-session-read-model/`

Proposed lanes:

- `proposed:hdr-tone-mapping-pipeline`
- `proposed:audio-compatibility-downmix-normalization`
- `proposed:ll-hls-cmaf-runtime`
- `proposed:dash-cmaf-playback-packaging`
- `proposed:hls-key-delivery-drm-boundary`
- `proposed:remote-transcode-worker-runtime`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:playback-os-resource-isolation`
- `proposed:playback-device-capacity-tuning`
- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:playback-release-hardware-matrix`

## Storage And VFS

Primary map: `docs/architecture/STORAGE_VFS.md`

Evidence workstreams:

- `docs/workstreams/storage-vfs/`
- `docs/workstreams/typed-storage-errors/`
- `docs/workstreams/managed-import-staging/`
- `docs/workstreams/link-apply-and-import-promotion/`
- `docs/workstreams/nfo-storage-write-policy/`
- `docs/workstreams/nfo-link-authority/`
- `docs/workstreams/addon-library-file-write-policy/`
- `docs/workstreams/nfo-sidecar-promotion-apply/`
- `docs/workstreams/admin-web-v2-storage-staging-route/`
- `docs/workstreams/storage-vfs-resilience-and-source-identity/`

Proposed lanes:

- `proposed:vfs-cache-repair-diagnostics`
- `proposed:remote-storage-health-and-circuit-breaker`

## Library, Metadata, NFO, And Artwork

Primary map: `docs/architecture/LIBRARY_PIPELINE.md`

Evidence workstreams:

- `docs/workstreams/core-architecture-deepening/`
- `docs/workstreams/library-metadata-scan-policy/`
- `docs/workstreams/metadata-catalog/`
- `docs/workstreams/metadata-provider-breadth/`
- `docs/workstreams/metadata-acquisition-pipeline/`
- `docs/workstreams/metadata-profile-configuration-authority/`
- `docs/workstreams/metadata-application-policy-seam/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/metadata-provider-attempt-runtime/`
- `docs/workstreams/metadata-operations/`
- `docs/workstreams/scan-addon-bulk-metadata-scrape/`
- `docs/workstreams/scan-addon-bulk-continuation/`
- `docs/workstreams/nfo-round-trip-preservation/`
- `docs/workstreams/nfo-sidecar-backup-policy/`
- `docs/workstreams/nfo-backup-retention-diagnostics/`
- `docs/workstreams/managed-artwork-fetch-artifact-storage/`
- `docs/workstreams/managed-artwork-gallery-candidate-management/`
- `docs/workstreams/managed-artwork-thumbnail-variants/`
- `docs/workstreams/managed-artwork-artifact-lifecycle-cleanup/`
- `docs/workstreams/managed-artwork-postgresql-parity/`

Proposed lanes:

- `proposed:library-watcher-and-media-intake-stability`
- `proposed:artwork-delivery-pipeline`
- `proposed:tmdb-series-season-episode-depth`
- `proposed:douban-provider-mvp`
- `proposed:bangumi-provider-mvp`

## State, Database, Identity, And Access

Primary map: `docs/architecture/STATE_ACCESS.md`

Evidence workstreams:

- `docs/workstreams/public-api-contract/`
- `docs/workstreams/access-boundary-auth/`
- `docs/workstreams/credential-session-auth/`
- `docs/workstreams/identity-and-library-access-contract/`
- `docs/workstreams/user-playback-state-contract/`
- `docs/workstreams/postgresql-production-readiness/`
- `docs/workstreams/repository-seam-deepening/`
- `docs/workstreams/durable-job-runtime-admin-read-model/`
- `docs/workstreams/admin-catalog-governance-read-model/`
- `docs/workstreams/catalog-hydration-lookup-deepening/`
- `docs/workstreams/public-client-library-browse-query-contract/`

Proposed lanes:

- `proposed:playback-db-write-pressure-and-wal-policy`
- `proposed:playback-access-policy-and-session-limits`
- `proposed:fts-filter-scale-up`

## Realtime, Events, And Sync

Primary map: `docs/architecture/REALTIME_SYNC.md`

Evidence workstreams:

- `docs/workstreams/addons-automation/`
- `docs/workstreams/addon-task-runtime-contract/`
- `docs/workstreams/addon-notification-bridge/`
- `docs/workstreams/addon-notification-provider-adapters/`
- `docs/workstreams/addon-notification-platform-adapters/`
- `docs/workstreams/user-playback-state-contract/`
- `docs/workstreams/public-client-browser-playback-session-identity/`
- `docs/workstreams/admin-playback-session-read-model/`
- `docs/workstreams/casting-renderer-runtime/`

Proposed lanes:

- `proposed:client-realtime-event-gateway`
- `proposed:offline-sync-and-download-artifacts`
- `proposed:multi-device-playback-conflict-policy`

## Control Plane

Primary map: `docs/architecture/CONTROL_PLANE.md`

Evidence workstreams:

- `docs/workstreams/server-runtime-deepening/`
- `docs/workstreams/runtime-foundation/`
- `docs/workstreams/durable-job-recovery/`
- `docs/workstreams/durable-job-queue-and-resource-classes/`
- `docs/workstreams/durable-job-ownership-leases/`
- `docs/workstreams/job-runtime-worker-control-plane/`
- `docs/workstreams/worker-job-cancellation-checkpoints/`
- `docs/workstreams/admin-operations-read-models/`
- `docs/workstreams/admin-web-console/`
- `docs/workstreams/web-admin-generated-artifacts-automation/`
- `docs/workstreams/web-admin-generated-artifact-review-mutations/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/addon-architecture-deepening/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/addon-install-guide-generation/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/public-client-library-browse-query-contract/`

Proposed lanes:

- `proposed:durable-job-priority-policy-and-scheduler-migration`
- `proposed:control-plane-observability-and-trace-context`
- `proposed:api-scale-and-cache-contracts`
- `proposed:remote-access-and-endpoint-discovery`

## Operations And Release

Primary map: `docs/architecture/OPERATIONS_RELEASE.md`

Evidence workstreams:

- `docs/workstreams/self-hosted-release-readiness/`
- `docs/workstreams/release-packaging-and-distribution/`
- `docs/workstreams/postgresql-production-readiness/`
- `docs/workstreams/admin-settings-configuration-authority/`
- `docs/workstreams/admin-web-v2-system-settings-route/`
- `docs/workstreams/admin-web-v2-settings-mutation-authority/`
- `docs/workstreams/admin-operations-read-models/`

Proposed lanes:

- `proposed:self-hosted-remote-access-cookbook`
- `proposed:backup-classification-for-generated-artifacts`
- `proposed:config-hot-apply-and-restart-required-model`
