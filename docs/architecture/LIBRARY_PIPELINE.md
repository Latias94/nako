# Library And Asset Pipeline Architecture

Last updated: 2026-06-02

This document maps the media lifecycle after files become visible through VFS.
It covers scan, watcher, probe, metadata, artwork, and addon-assisted intake.

## Target Chain

```text
Storage event or scheduled scan
  -> stable file candidate
  -> source record and tombstone reconciliation
  -> ffprobe facts
  -> local inference
  -> NFO/provider/addon metadata evidence
  -> canonical metadata merge
  -> catalog graph and search projection
  -> artwork derivatives and client delivery
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Durable scan state | Shipped | `docs/adr/0012-durable-scan-state-and-source-tombstones.md` | Watcher/debounce productization. |
| Source tombstones | Shipped foundation | `docs/adr/0012-durable-scan-state-and-source-tombstones.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Watcher/debounce productization and repair workflows. |
| Local inference | Shipped foundation | `CONTEXT.md`; metadata/catalog lanes | Anime/series path heuristics and confidence reporting. |
| Media probe | Shipped foundation | playback/transcode lanes | More HDR/audio/subtitle technical facts. |
| NFO authority | Shipped foundation | `docs/adr/0008-nfo-as-local-metadata-boundary.md` | Round-trip/writeback polish and backup policy. |
| Metadata merge policy | Shipped foundation | `docs/adr/0007-metadata-merge-policy-and-local-authority.md` | Field-level review UX and provider conflict diagnostics. |
| TMDB provider | Shipped movie plus series/season/episode foundation | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/` | Provider depth, identity matching, and conflict precision. |
| Douban provider | Shipped MVP foundation | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/` | Provider depth, identity matching, and conflict precision. |
| Bangumi provider | Shipped MVP foundation | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/` | Anime-first provider depth and identity matching. |
| Addon-assisted metadata | Shipped guarded apply, bulk apply, provider mapping breadth, read-only apply recovery foundation, and Web recovery UI | addon architecture lanes; `docs/workstreams/generated-artifact-metadata-authority-apply/`; `docs/workstreams/generated-artifact-bulk-metadata-apply/`; `docs/workstreams/generated-artifact-provider-mapping-breadth/`; `docs/workstreams/generated-artifact-apply-operations-repair/`; `docs/workstreams/web-admin-generated-artifact-recovery-ui/` | Active bounded repair actions, then deeper provider identity precision. |
| Artwork artifact lifecycle | Shipped selection, lifecycle, variant, and remediation foundation | managed artwork lanes | Delivery cache placeholders and broader derivative policy. |
| Watcher/debounce | Weak | This document | Open `library-watcher-and-media-intake-stability`. |

## Workstream Evidence

Use
`docs/architecture/WORKSTREAM_LINKS.md#library-metadata-nfo-and-artwork` as the
consolidated index for library, metadata, NFO, and artwork workstreams. Keep
this document focused on intake and asset pipeline capability state.

## Next Work Lanes

### source-identity-foundation

Status: The first source identity resilience slice shipped in
`docs/workstreams/storage-vfs-resilience-and-source-identity/`.

Shipped behavior:

- layered source fingerprint evidence without mandatory full-file hashing;
- strong-evidence move/rename reconciliation;
- duplicate-source suggestions instead of automatic weak-evidence merges;
- source-scoped storage failure diagnostics.

### library-watcher-and-media-intake-stability

Goal: Make incremental library intake safe for large files, slow copies, and
remote storage.

Scope:

- filesystem watcher integration;
- debounce and stable-size detection;
- copy-in-progress handling;
- scheduled reconciliation scan;
- per-library intake diagnostics.

Exit criteria:

- a large file copy does not trigger premature probe;
- moved/renamed sources do not lose metadata when evidence is strong;
- scan failures stay source-scoped.

### artwork-delivery-cache-placeholder

Goal: Serve artwork in client-appropriate forms instead of sending raw provider
images everywhere.

Scope:

- derivative generation;
- WebP or other client-appropriate output formats;
- size presets;
- Blurhash or placeholder evidence;
- cache invalidation and selected artwork policy.

### generated-artifact-bulk-metadata-apply

Status: Closed at `docs/workstreams/generated-artifact-bulk-metadata-apply/`.

Goal: Turn one-artifact Metadata Authority apply into guarded bulk planning,
durable execution, partial-failure reporting, and Web Admin operator controls.

Shipped:

- read-only bulk apply-plan contract;
- durable batch confirmation and job-backed execution;
- per-item idempotency and partial-failure reporting;
- Admin status/result routes and Web Admin operator workflow.

Next lanes:

- `docs/workstreams/generated-artifact-provider-mapping-breadth/`;
- `docs/workstreams/generated-artifact-apply-operations-repair/`;
- `docs/workstreams/generated-artifact-apply-repair-actions/`.

### generated-artifact-provider-mapping-breadth

Status: Closed at
`docs/workstreams/generated-artifact-provider-mapping-breadth/`.

Goal: Extend accepted metadata Generated Artifact apply so it can plan and
apply Provider Subject and Provider Mapping proposals without turning review
acceptance into a catalog mutation, then surface those effects through
bulk/Admin/Web results.

Shipped backend slices:

- `GAPM-020` adds redaction-safe read-only Provider Mapping plan entries and
  counters to the existing Generated Artifact metadata apply plan.
- `GAPM-030` adds durable/idempotent Provider Subject and accepted Provider
  Mapping apply through the single-artifact Metadata Authority outcome
  transaction.
- `GAPM-040` adds Provider Mapping apply/skip/noop counters to bulk plan
  summaries, batch snapshots, Admin HTTP responses, generated contracts, and
  Web read-model mapping.

Closeout:

- Web Admin renders Provider Mapping plan/result facts in single and bulk
  Metadata Authority apply workflows without adding backend apply behavior.
- Continue through the closed
  `docs/workstreams/generated-artifact-apply-operations-repair/` evidence and
  `proposed:provider-identity-mapping-breadth`, not by reopening this lane.

Boundaries:

- no provider search/depth or hierarchy repair;
- no raw provider payload exposure;
- no Public Client API changes;
- no operations repair tooling in this lane.

### generated-artifact-apply-operations-repair

Status: Closed at
`docs/workstreams/generated-artifact-apply-operations-repair/`.

Goal: Turn durable Generated Artifact apply outcomes and bulk batch state into
an operator-facing recovery workflow for stale, failed, skipped, and noop
results without weakening Metadata Authority boundaries or redaction.

Shipped:

- one-artifact apply outcomes are queryable through Admin list/detail routes;
- outcome-only records and bulk batch terminal items flow into an Admin
  recovery queue;
- recovery entries classify `needs_repair`, `needs_review`, `replay_only`,
  and `resolved`;
- generated Admin contracts and Web Admin read models carry recovery facts
  without raw internal leakage.

Boundaries:

- no blind retry button without plan/result semantics;
- no provider-depth precision expansion in this lane;
- no Public Client API changes.

Follow-ons:

- `docs/workstreams/web-admin-generated-artifact-recovery-ui/` (closed)
- `docs/workstreams/generated-artifact-apply-repair-actions/` (active)
- `proposed:provider-identity-mapping-breadth`

### generated-artifact-apply-repair-actions

Status: Active at
`docs/workstreams/generated-artifact-apply-repair-actions/`.

Goal: Turn the recovery queue into bounded repair actions while preserving
Metadata Authority apply as the execution kernel.

Initial boundary:

- prove whether existing apply and bulk apply routes already provide enough
  repair semantics;
- add a recovery-context wrapper only if it prevents wrong-artifact, stale-row,
  or replay-only mistakes;
- do not add provider-depth or hierarchy matching work in this lane.

## Risk Register

### Watch Events Are Not Stable Media Events

Create/modify events often fire before a media file is complete.

Mitigation:

- debounce events;
- require size stability or closed-file evidence where available;
- use scheduled scans as correction, not as the only intake path.

### Addons Must Not Own Canonical State

Addon or AI metadata can be powerful, but host policy must own application,
field locks, provenance, and conflict resolution.

Mitigation:

- addons submit evidence or proposed changes;
- Nako applies through metadata application policy;
- user/admin review controls ambiguous changes.

### Artwork Can Dominate Client Perceived Performance

Large raw images can make a good catalog feel slow.

Mitigation:

- generate derivatives;
- cache by selected artwork and size;
- expose placeholder hashes for skeleton/loading states.

## Agent Notes

Do not put provider-specific hierarchy directly into `MediaItem` shape when a
Nako term exists. Use `Provider Subject`, `Provider Mapping`, `Local Inference`,
and `Canonical Metadata` from `CONTEXT.md`.
