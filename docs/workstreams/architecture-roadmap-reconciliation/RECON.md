# Architecture Roadmap Reconciliation - Audit Notes

Status: Active
Last updated: 2026-06-01

These notes summarize the read-only sub-architecture audit used to open this
workstream. They are planning input, not closeout evidence by themselves.

## Playback And Transcode

State: implementation and roadmap mostly match. No active lane should be
running by default.

High-risk drift:

- `playback-transcode-jellyfin-class-hardening` contains a stale ADR 0053 path.
- `web-media-live-public-client-parity` handoff says browser heartbeat lacks a
  playback session id, but a later browser playback session identity lane
  shipped that contract.

Recommended follow-ons:

- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:playback-release-hardware-matrix`
- `proposed:hardware-tone-map-execution`
- `proposed:subtitle-burn-in-and-client-subtitle-policy`

## Storage And VFS

State: storage lane is idle after remote storage health and circuit-breaker
closeout.

High-risk drift:

- Some historical TODO/HANDOFF notes still show old active states.
- The workstream navigation should not describe durable backend circuit
  breakers as only a future follow-on.

Recommended follow-ons:

- `proposed:vfs-cache-repair-diagnostics`
- `proposed:source-fingerprint-escalation-policy`
- `proposed:library-watcher-and-media-intake-stability`
- `proposed:storage-vfs-postgresql-runtime-harness`
- `proposed:hls-artifact-io-pressure-enforcement`

## Library, Metadata, NFO, And Artwork

State: Generated Artifact Metadata Authority apply is closed; provider and
artwork foundations are deeper than some maps state.

High-risk drift:

- `LIBRARY_PIPELINE.md` still describes Douban and Bangumi as not started.
- Proposed provider lanes still name MVPs that have already landed as
  foundation slices.
- Artwork and metadata evidence links are missing from
  `WORKSTREAM_LINKS.md`.

Recommended follow-ons:

- `proposed:generated-artifact-bulk-metadata-apply`
- `proposed:generated-artifact-provider-mapping-breadth`
- `proposed:metadata-provider-depth-and-precision`
- `proposed:library-watcher-and-media-intake-stability`
- `proposed:artwork-delivery-cache-placeholder`

## Web Product

State: current product Web lanes are closed or deferred. The Web lane should
not show historical copy/refactor work as active.

High-risk drift:

- `docs/workstreams/README.md` still says several completed Web lanes are
  active.
- `WORKSTREAM_LINKS.md` needs clearer Web evidence and proposed follow-ons
  after MVP Gate 3 and GAMA closeout.

Recommended follow-ons:

- `proposed:admin-settings-api-backed-restoration`
- `proposed:web-public-client-library-scoped-item-browse`
- `proposed:generated-artifact-apply-operations-repair`
- `proposed:web-player-error-recovery-ux`
- `proposed:desktop-tauri-native-playback-spike`

## State, Database, Identity, And Access

State: shared state/access work is mostly foundational and should be treated as
cross-lane coordination.

High-risk drift:

- `STATE_ACCESS.md` still calls Playback Policy partial even though persisted
  user/role playback policy and planner enforcement shipped.
- State/access evidence links miss browser playback session identity,
  playlists, playback policy, Admin TypeScript contract, and managed artwork
  PostgreSQL parity.

Recommended follow-ons:

- `proposed:playback-db-write-pressure-and-wal-policy`
- `proposed:playback-access-policy-and-session-limits`
- `proposed:api-scale-and-cache-contracts`
- `proposed:postgresql-contract-ci-hardening`

## Control Plane, Addons, Ops, And Realtime

State: control-plane foundation is strong, but priority scheduling,
observability, addon manager trust/update lifecycle, realtime client events,
and remote endpoint discovery remain future lanes.

High-risk drift:

- `CONTROL_PLANE.md` says HTTP cache/ETag contracts are not started, while
  selected artwork image cache validators already shipped.
- `WORKSTREAM_LINKS.md` misses completed addon notification, attempt history,
  official addon, outbound credential, and source catalog evidence links.

Recommended follow-ons:

- `proposed:durable-job-priority-policy-and-scheduler-migration`
- `proposed:control-plane-observability-and-trace-context`
- `proposed:addon-manager-trust-update-lifecycle`
- `proposed:client-realtime-event-gateway`
- `proposed:self-hosted-remote-access-and-endpoint-discovery`
