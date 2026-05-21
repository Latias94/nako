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
promotion-preview lane. The next mainline lane is
`link-apply-and-import-promotion`.

## Active Task

- Task ID: PRPH-070
- Owner: planner
- Files: `docs/workstreams/post-rpd-product-hardening`,
  `docs/workstreams/managed-import-staging`,
  `docs/workstreams/link-apply-and-import-promotion`
- Validation: child closeout reviewed; next executable lane recorded
- Status: ACTIVE
- Review: route execution through `link-apply-and-import-promotion`
- Evidence: `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`

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
- Keep playback/transcode ops hardening as a parallel sidecar candidate only
  if it stays diagnostic/runtime-focused and avoids NFO/import write scope.

## Blockers

- None for LAIP-030.

## Next Recommended Action

- Execute `link-apply-and-import-promotion` LAIP-030.
- Keep `post-rpd-product-hardening` active while it continues ordering the next
  product-hardening lanes. Re-score playback/transcode ops, network, AI, and
  addon runtime after the apply boundary is proven or split.
