# Douban Series Follow-On Map Reconciliation

## Goal

Update architecture and roadmap maps after the Douban TV subject slice shipped,
so future planning no longer routes work to the completed Series-level
capability while still preserving Season/Episode endpoint depth as a focused
follow-on.

## Requirements

- Mark Douban subject-level Series support as shipped in architecture and
  roadmap docs.
- Remove or narrow stale `proposed:douban-tv-episode-endpoint-depth`
  references where they now imply the completed Series-level task is still
  available.
- Preserve the explicit boundary that Douban Season/Episode support,
  child graph preview, hierarchy mutation, Admin/Web governance, and Public
  Client API exposure remain out of scope unless opened as separate tasks.
- Link the archived Trellis task
  `.trellis/tasks/archive/2026-06/06-05-douban-tv-episode-endpoint-depth/`
  as shipped evidence.
- Do not change Rust code, generated API contracts, schema, or provider
  capability behavior in this task.

## Acceptance Criteria

- [x] `docs/architecture/LIBRARY_PIPELINE.md` describes Douban Series
  subject-level support as shipped and names the remaining Season/Episode
  endpoint depth follow-on accurately.
- [x] `docs/architecture/LANES.md`, `docs/architecture/WORKSTREAM_LINKS.md`,
  `docs/ROADMAP.md`, and `docs/GOALS.md` no longer route planners to the
  completed broad `douban-tv-episode-endpoint-depth` task.
- [x] Remaining proposed labels distinguish unfinished Douban Season/Episode
  graph depth from the completed Series subject slice.
- [x] No new implementation or public/API contract behavior is introduced.

## Definition of Done

- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-05-douban-series-follow-on-map-reconciliation`
- Focused grep proves stale broad proposed labels were removed or narrowed.
- Documentation diff reviewed for no accidental Season/Episode overclaim.

## Technical Approach

Use the archived task PRD and current metadata tests as the source of truth:
Douban now supports Movie plus TV Series subject-level search/fetch through the
existing subtype-aware endpoint, but Season/Episode direct support and related
graph depth are still unsupported. Update only planning docs that still list the
old broad proposal as available.

## Decision (ADR-lite)

**Context**: The completed Douban task intentionally shipped a Series-only
subject slice. Several durable architecture maps still list
`proposed:douban-tv-episode-endpoint-depth`, which can now mislead future
campaign planning into reopening completed work.

**Decision**: Reconcile the maps by recording Series support as shipped and
renaming remaining future work to Douban Season/Episode graph depth where that
capability is still unsupported.

**Consequences**: Planning remains honest: future agents can still open a
Douban child-depth task, but they should not duplicate the completed
Series-level endpoint work.

## Out of Scope

- No Douban Season or Episode provider implementation.
- No hierarchy preview or accepted hierarchy application.
- No Admin API, Public Client API, Web Admin, generated contract, or schema
  changes.
- No changes to `crates/nako-metadata`.

## Research References

- [`research/douban-series-completion-evidence.md`](research/douban-series-completion-evidence.md)
  - local evidence for completed Series support and remaining gaps.

## Technical Notes

- Relevant shipped evidence:
  `.trellis/tasks/archive/2026-06/06-05-douban-tv-episode-endpoint-depth/`.
- Metadata spec authority:
  `.trellis/spec/nako-metadata/backend/quality-guidelines.md`, especially the
  provider capability endpoint precision scenario.
