# Architecture Workstream Links

Last updated: 2026-06-02

This index links architecture capability areas to workstream evidence and
candidate follow-on lanes. The top-level architecture map stays concise; this
file is the cross-reference for agents that need execution history.

Use this index as navigation, not as the source of design truth:

- ADRs own durable decisions.
- `docs/architecture/*.md` owns capability progress and risk maps.
- `docs/architecture/LANES.md` owns terminal/worktree lane routing for
  parallel development.
- `docs/workstreams/*` owns task plans, evidence, gates, and handoff state.

## Linkage Rules

- A shipped or partial capability should link at least one concrete workstream
  when evidence exists.
- A future lane should use a proposed slug until a real workstream directory is
  opened, for example `proposed:hls-seek-restart-lifecycle`.
- New workstreams should add `architecture_refs` and `capability_tags` to
  `WORKSTREAM.json` when they materially change an architecture capability.
- New long-lived terminal lanes should be registered in
  `docs/architecture/LANES.md` before multiple agents rely on the boundary.
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

## Architecture Roadmap Reconciliation

Primary map: `docs/workstreams/architecture-roadmap-reconciliation/README.md`

Evidence workstreams:

- `docs/workstreams/architecture-roadmap-reconciliation/` (closed)

This was the planner/docs lane for reconciling roadmap status,
architecture evidence links, active queue routing, and proposed follow-on
names after the latest sub-architecture audit.

## MVP Release Convergence

Primary map: `docs/workstreams/mvp-release-shape/MVP.md`

Evidence workstreams:

- `docs/workstreams/mvp-release-shape/` (closed)

This was a planner-owned release convergence overlay. It routed release
blockers to existing capability lanes instead of replacing playback, storage,
web, addon, client, or operations ownership. Future release execution should
open focused follow-on workstreams.

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
- `docs/workstreams/playback-subtitle-language-default-policy/`
- `docs/workstreams/hls-alternate-audio-renditions/`
- `docs/workstreams/hls-audio-sidecar-artifacts/`
- `docs/workstreams/hls-selected-main-audio-cleanup/`
- `docs/workstreams/playback-audio-language-default-policy/`
- `docs/workstreams/hls-seek-restart-lifecycle/`
- `docs/workstreams/hls-progressive-runtime-boundary/`
- `docs/workstreams/playback-runtime-resource-scheduler/`
- `docs/workstreams/admin-playback-runtime-diagnostics/`
- `docs/workstreams/admin-playback-session-read-model/`
- `docs/workstreams/audio-compatibility-downmix-normalization/`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/`
- `docs/workstreams/hdr-tone-mapping-pipeline/`
- `docs/workstreams/playback-compatibility-matrix-hardening/`
- `docs/workstreams/transcode-capability-inventory-matrix/`
- `docs/workstreams/hls-runtime-lifecycle-boundary/`
- `docs/workstreams/hls-progressive-readiness-test-stability/`
- `docs/workstreams/playback-transcode-jellyfin-class-hardening/` (closed)

Proposed lanes:

- `proposed:ll-hls-cmaf-runtime`
- `proposed:dash-cmaf-playback-packaging`
- `proposed:hls-key-delivery-drm-boundary`
- `proposed:remote-transcode-worker-runtime`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:playback-os-resource-isolation`
- `proposed:playback-device-capacity-tuning`
- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:player-hls-session-controls-and-recovery`
- `proposed:playback-release-hardware-matrix`
- `proposed:hardware-tone-map-execution`
- `proposed:subtitle-burn-in-and-client-subtitle-policy`

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
- `docs/workstreams/remote-storage-health-and-circuit-breaker/`

Proposed lanes:

- `proposed:vfs-cache-repair-diagnostics`
- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:source-fingerprint-escalation-policy`
- `proposed:library-watcher-and-media-intake-stability`
- `proposed:storage-vfs-postgresql-runtime-harness`

## Library, Metadata, NFO, And Artwork

Primary map: `docs/architecture/LIBRARY_PIPELINE.md`

Evidence workstreams:

- `docs/workstreams/core-architecture-deepening/`
- `docs/workstreams/library-metadata-scan-policy/`
- `docs/workstreams/metadata-catalog/`
- `docs/workstreams/metadata-provider-breadth/`
- `docs/workstreams/metadata-provider-depth-and-precision/` (closed)
- `docs/workstreams/tmdb-season-episode-graph-depth/` (closed)
- `docs/workstreams/bangumi-relations-and-episode-depth/` (closed)
- `docs/workstreams/douban-subject-kind-precision/` (closed)
- `docs/workstreams/metadata-candidate-durable-review/` (closed)
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed)
- `docs/workstreams/admin-web-provider-depth-governance/` (closed)
- `docs/workstreams/admin-candidate-review-list-navigation/` (closed)
- `docs/workstreams/provider-review-global-queue-search/` (closed)
- `docs/workstreams/provider-governance-bulk-review/` (closed)
- `docs/workstreams/provider-governance-durable-batch-execution/` (closed)
- `docs/workstreams/metadata-acquisition-pipeline/`
- `docs/workstreams/metadata-profile-configuration-authority/`
- `docs/workstreams/metadata-application-policy-seam/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/` (closed)
- `docs/workstreams/generated-artifact-provider-mapping-breadth/` (closed)
- `docs/workstreams/generated-artifact-apply-operations-repair/` (closed)
- `docs/workstreams/generated-artifact-apply-repair-actions/` (closed)
- `docs/workstreams/metadata-provider-attempt-runtime/`
- `docs/workstreams/metadata-operations/`
- `docs/workstreams/scan-addon-bulk-metadata-scrape/`
- `docs/workstreams/scan-addon-bulk-continuation/`
- `docs/workstreams/nfo-round-trip-preservation/`
- `docs/workstreams/nfo-sidecar-backup-policy/`
- `docs/workstreams/nfo-backup-retention-diagnostics/`
- `docs/workstreams/nfo-sidecar-cancellation-checkpoints/`
- `docs/workstreams/managed-artwork-fetch-artifact-storage/`
- `docs/workstreams/managed-artwork-ingest-selection/`
- `docs/workstreams/managed-artwork-public-serving-selection/`
- `docs/workstreams/managed-artwork-artifact-store-drift-inventory/`
- `docs/workstreams/managed-artwork-remediation-policy/`
- `docs/workstreams/managed-artwork-gallery-candidate-management/`
- `docs/workstreams/managed-artwork-thumbnail-variants/`
- `docs/workstreams/selected-artwork-unpublish-delete-policy/`
- `docs/workstreams/managed-artwork-artifact-lifecycle-cleanup/`
- `docs/workstreams/managed-artwork-postgresql-parity/`
- `docs/workstreams/admin-web-v2-item-artwork-selection/`
- `docs/workstreams/metadata-merge-policy-unification/`
- `docs/workstreams/metadata-application-cross-path-audit/`
- `docs/workstreams/metadata-refresh-seam/`

Proposed lanes:

- `proposed:library-watcher-and-media-intake-stability`
- `proposed:artwork-delivery-cache-placeholder`
- `proposed:provider-review-related-hierarchy-application`
- `proposed:douban-tv-episode-endpoint-depth`
- `proposed:provider-identity-mapping-breadth`
- `proposed:provider-review-public-client-governance`
- `proposed:provider-governance-audit-and-undo`

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
- `docs/workstreams/public-client-browser-playback-session-identity/`
- `docs/workstreams/user-playlists-contract-and-web-slice/`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/admin-api-typescript-contract/`
- `docs/workstreams/managed-artwork-postgresql-parity/`

Proposed lanes:

- `proposed:playback-db-write-pressure-and-wal-policy`
- `proposed:playback-access-policy-and-session-limits`
- `proposed:api-scale-and-cache-contracts`
- `proposed:postgresql-contract-ci-hardening`
- `proposed:fts-filter-scale-up`

## Web Product

Primary map: `docs/architecture/LANES.md#web-product`

Evidence workstreams:

- `docs/workstreams/web-media-live-public-client-parity/`
- `docs/workstreams/admin-media-management-context-links/`
- `docs/workstreams/web-playlist-management-ui-mutations/`
- `docs/workstreams/web-admin-generated-artifact-review-mutations/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/web-admin-generated-artifact-recovery-ui/` (closed)
- `docs/workstreams/provider-governance-bulk-review/` (closed)
- `docs/workstreams/web-mvp-live-smoke/` (closed)

Proposed lanes:

- `proposed:admin-settings-api-backed-restoration`
- `proposed:web-public-client-library-scoped-item-browse`
- `proposed:web-public-client-release-smoke-script`
- `proposed:web-player-error-recovery-ux`
- `proposed:desktop-tauri-native-playback-spike`

## Realtime, Events, And Sync

Primary map: `docs/architecture/REALTIME_SYNC.md`

Evidence workstreams:

- `docs/workstreams/addons-automation/`
- `docs/workstreams/addon-task-runtime-contract/`
- `docs/workstreams/addon-notification-bridge/`
- `docs/workstreams/addon-notification-provider-adapters/`
- `docs/workstreams/addon-notification-platform-adapters/`
- `docs/workstreams/addon-notification-template-controls/`
- `docs/workstreams/addon-notification-provider-attempt-history/`
- `docs/workstreams/addon-notification-provider-live-smoke/`
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
- `docs/workstreams/provider-governance-durable-batch-execution/` (closed)
- `docs/workstreams/admin-operations-read-models/`
- `docs/workstreams/admin-web-console/`
- `docs/workstreams/web-admin-generated-artifacts-automation/`
- `docs/workstreams/web-admin-generated-artifact-review-mutations/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/` (closed)
- `docs/workstreams/generated-artifact-apply-operations-repair/` (closed)
- `docs/workstreams/generated-artifact-apply-repair-actions/` (closed)
- `docs/workstreams/addon-architecture-deepening/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/addon-install-guide-generation/`
- `docs/workstreams/addon-source-catalog-marketplace/`
- `docs/workstreams/addon-outbound-task-dispatch-credentials/`
- `docs/workstreams/official-addon-e2e-alpha2/`
- `docs/workstreams/addon-notification-template-controls/`
- `docs/workstreams/addon-notification-provider-attempt-history/`
- `docs/workstreams/addon-notification-provider-live-smoke/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/public-client-library-browse-query-contract/`

Proposed lanes:

- `proposed:durable-job-priority-policy-and-scheduler-migration`
- `proposed:control-plane-observability-and-trace-context`
- `proposed:addon-manager-trust-update-lifecycle`
- `proposed:api-scale-and-cache-contracts`
- `proposed:self-hosted-remote-access-and-endpoint-discovery`

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
- `proposed:self-hosted-remote-access-and-endpoint-discovery`
- `proposed:admin-settings-api-backed-restoration`
- `proposed:backup-classification-for-generated-artifacts`
- `proposed:config-hot-apply-and-restart-required-model`
