# Post-RPD Product Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The post-RPD product roadmap is open as an umbrella. It chooses
`metadata-provider-breadth` as the first execution lane and records NFO/link,
playback/transcode, managed import, network, AI, and addon distribution as
ordered follow-ons. `metadata-provider-breadth` is complete. The next mainline
lane, `nfo-link-authority`, has completed VFS link dry-run diagnostics,
Source Duplicate Relationship filesystem-link suggestions, NFO authority
preview, and the link apply split decision. The next mainline should be
`managed-import-staging`, which is now open.

## Active Task

- Task ID: PRPH-050
- Owner: planner
- Files: `docs/workstreams/post-rpd-product-hardening`,
  `docs/workstreams/nfo-link-authority`
- Validation: child closeout reviewed; next executable lane recorded
- Status: ACTIVE
- Review: route execution through `managed-import-staging`
- Evidence: `docs/workstreams/managed-import-staging/DESIGN.md`

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
- Keep playback/transcode ops hardening as a parallel sidecar candidate only
  if it stays diagnostic/runtime-focused and avoids NFO/import write scope.

## Blockers

- None for opening the next execution lane.

## Next Recommended Action

- Execute `managed-import-staging` MIS-020.
- Keep `post-rpd-product-hardening` active until the next lane is represented
  by durable docs, or close it if the roadmap no longer reduces coordination
  cost.
