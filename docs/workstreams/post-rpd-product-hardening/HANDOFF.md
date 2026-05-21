# Post-RPD Product Hardening — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

The post-RPD product roadmap is open as an umbrella. It chooses
`metadata-provider-breadth` as the first execution lane and records NFO/link,
playback/transcode, managed import, network, AI, and addon distribution as
ordered follow-ons. `metadata-provider-breadth` is complete. `nfo-link-authority`
is complete with VFS link dry-run diagnostics, Source Duplicate Relationship
filesystem-link suggestions, NFO authority preview, and the link apply split
decision. `managed-import-staging` is complete as a non-mutating staging and
promotion-preview lane. `link-apply-and-import-promotion` is complete after
implementing accepted promotion apply, VFS-mediated target creation, catalog
commit ordering, duplicate evidence, and cleanup-complete/cleanup-pending
audit. LAIP-070 split NFO sidecar import/export mutation to
`nfo-sidecar-promotion-apply`. PRPH-080 selected
`nfo-sidecar-promotion-apply` as the next mainline execution lane. NFO Sidecar
Promotion Apply is now complete with accepted import/export apply, VFS
write/restore, canonical metadata/local authority, hierarchy confirmation,
rollback/repair, idempotent replay, and redacted diagnostics. PRPH-090 selects
Playback/Transcode Ops Hardening as the next mainline lane, and PRPH-100 opens
`playback-transcode-ops-hardening`.

## Active Task

- Task ID: PTOH-020
- Owner: unassigned
- Files: `docs/workstreams/playback-transcode-ops-hardening`,
  `crates/taru-transcode/src/hardware.rs`,
  `crates/taru-server/src/app/playback`,
  `crates/taru-server/src/http/admin.rs`,
  `crates/taru-api/src/admin.rs`
- Validation: implement and verify an Admin-only playback runtime readiness
  contract with diagnostics/runtime gates.
- Status: READY
- Review: keep scope runtime/diagnostic-focused; do not mix downloader,
  metadata authority, sidecar apply, or library file mutation into this lane.
- Evidence: PRPH-090 lane scoring in `DESIGN.md`, PRPH-100 lane open docs, and
  NSPA closeout evidence.

## Decisions Since Last Update

- Do not implement a generic downloads lane first.
- Treat downloads as `managed-import-staging` after metadata and local file
  authority are stronger.
- Start Wave 1 with metadata provider capability, matching, and conflict
  explanation rather than UI or AI breadth.
- Close Wave 1 before downloads/import because provider identity and ambiguity
  must be explicit first.
- Choose `nfo-link-authority` as the next mainline lane because it is the
  remaining high-risk local data-loss boundary.
- Open `nfo-link-authority` with VFS link dry-run diagnostics as the first
  non-destructive slice.
- Record filesystem-link evidence as suggested Source Duplicate Relationships
  without merging Media Sources or Media Items.
- Add non-mutating NFO authority preview before sidecar writes.
- Defer actual hardlink/symlink apply to a follow-on after managed import
  staging defines promotion, rollback, cleanup, audit, and source duplicate
  confirmation semantics.
- Managed Import Staging completed durable artifact records, redacted
  diagnostics, and non-mutating promotion preview.
- Actual promotion apply is split to `link-apply-and-import-promotion` because
  it requires operator confirmation, plan revalidation, durable audit,
  rollback/cleanup, VFS-only mutation, and catalog consistency.
- `link-apply-and-import-promotion` is complete with fresh closeout evidence for
  accepted promotion apply, VFS-mediated target creation, catalog commit
  ordering, duplicate evidence, and cleanup-complete/cleanup-pending audit.
- NFO sidecar import/export mutation is split to
  `nfo-sidecar-promotion-apply` because it is a separate accepted **Library
  File Write** and metadata-authority workflow with backup, retention,
  field-lock, hierarchy-confirmation, rollback/repair, idempotency, and
  redacted audit requirements.
- Keep playback/transcode ops hardening as a parallel sidecar candidate only
  if it stays diagnostic/runtime-focused and avoids NFO/import write scope.
- PRPH-080 selects `nfo-sidecar-promotion-apply` as the next mainline lane.
  Playback/transcode ops remains the safest parallel sidecar. Downloads and
  watch-folder acquisition remain downstream of accepted promotion and NFO
  sidecar apply boundaries.
- NFO Sidecar Promotion Apply is complete. PRPH-090 now selects
  Playback/Transcode Ops Hardening as the next mainline lane. Downloads/watch
  folder may be re-scored afterward, but must still enter through staged
  artifacts, promotion apply, and sidecar apply boundaries.
- `playback-transcode-ops-hardening` is opened. The first executable task is
  PTOH-020, which should harden runtime readiness diagnostics without changing
  Public Client API behavior.

## Blockers

- None for PTOH-020.

## Next Recommended Action

- Execute PTOH-020 in `playback-transcode-ops-hardening`.
- Keep downloads/watch-folder, network, AI, and addon runtime downstream or
  parallel only if they consume accepted Taru-owned boundaries and do not
  introduce new direct mutation paths.
