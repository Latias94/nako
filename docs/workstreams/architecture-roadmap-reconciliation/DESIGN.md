# Architecture Roadmap Reconciliation - Design

Status: Active
Last updated: 2026-06-01

## Problem

Nako has closed many architecture and product lanes quickly. The current code
and workstream evidence are ahead of some roadmap and architecture navigation
docs:

- top-level planning docs show no active planner goal after MVP Release Shape;
- some capability maps still list shipped provider, playback policy, storage,
  cache, Web, or addon work as future or partial;
- `docs/workstreams/README.md` still uses "active" wording for completed lanes
  in several high-traffic entries;
- proposed lane names include already-shipped MVP provider slices instead of
  the next useful provider-depth work;
- workstream indexes are missing important evidence links from recent closed
  lanes.

This drift makes parallel planning risky because a lane terminal can pick a
historical gap instead of a real next task.

## Target Outcome

After this reconciliation, the repository should answer three planning
questions from documentation alone:

1. What is active now?
2. Which completed work proves the current architecture state?
3. What focused follow-on should be opened next for each sub-architecture?

## Scope

Update planner and architecture navigation docs:

- `docs/GOALS.md`
- `docs/ROADMAP.md`
- `docs/workstreams/README.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/CONTROL_PLANE.md`
- targeted stale references only when they directly affect planner decisions

Open this workstream as the audit source of truth:

- `docs/workstreams/architecture-roadmap-reconciliation/`

## Non-Goals

- No Rust, Web, schema, API, generated contract, or runtime changes.
- No broad rewrite of every historical handoff.
- No reopening completed workstreams to make their old task wording perfect.
- No implementation of proposed follow-on lanes.
- No release publication or packaging changes.

## Policy

- `WORKSTREAM.json` remains the status authority for individual workstreams.
- Historical handoffs may preserve old execution context, but current queue
  documents must not route new work to closed lanes.
- When a proposed lane names an already-shipped MVP, rename or replace it with
  the next deeper capability.
- When a capability map says "not started" or "partial", link concrete shipped
  evidence or narrow the remaining gap.

## Closeout Criteria

- Program-level docs identify `architecture-roadmap-reconciliation` as the
  active planner goal.
- Architecture lane queue has exactly one active planner lane unless a later
  implementation lane is intentionally opened.
- High-risk drift from the six sub-architecture audits is either corrected or
  split into explicit follow-ons.
- JSON and diff checks pass.
