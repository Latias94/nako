# Post-RPD Product Hardening — Handoff

Status: Active
Last updated: 2026-05-21

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
`nfo-sidecar-promotion-apply` as the next mainline execution lane.

## Active Task

- Task ID: NSPA-020
- Owner: planner
- Files: `docs/workstreams/nfo-sidecar-promotion-apply`, `crates/taru-core`,
  `crates/taru-db`
- Validation: focused DB contract tests for durable sidecar apply records,
  idempotency-key lookup, state transitions, and redacted audit snapshots.
- Status: READY
- Review: execute through `nfo-sidecar-promotion-apply`.
- Evidence: PRPH-080 lane scoring in `DESIGN.md` and NSPA-010 planning docs.

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

## Blockers

- None for NSPA-020.

## Next Recommended Action

- Execute `nfo-sidecar-promotion-apply` NSPA-020 with TDD.
- Keep `post-rpd-product-hardening` active while it continues ordering the next
  product-hardening lanes. After NSPA has durable acceptance/audit, re-check
  whether playback/transcode ops should run as a parallel sidecar or whether
  NFO export apply should continue as the mainline.
