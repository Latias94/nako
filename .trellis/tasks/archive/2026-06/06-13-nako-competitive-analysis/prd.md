# research: nako vs jellyfin plex competitive analysis

## Goal

Research Nako's product position against Jellyfin and Plex, with special focus
on self-hosted media-library workflows, addon/extension strategy, metadata,
playback, remote access, and operator experience. The output should help decide
where Nako should copy established expectations, where it should deliberately
diverge, and which near-term product slices create meaningful differentiation.

## What I Already Know

* User wants to reference Jellyfin and Plex while building Nako as a new
  self-hosted multimedia film/video library.
* Jellyfin GPL source is available under `repo-ref/jellyfin` for behavior and
  architecture research only.
* Official addon work lives in `../nako-official-addons`.
* Nako is currently `0.1.0-alpha.2`, a technical preview rather than a stable
  Jellyfin/Plex replacement.
* Nako's vocabulary intentionally uses Addon, Addon Sidecar, Addon Protocol,
  Addon Token, Addon Task, and Addon Permission instead of in-process plugin
  terminology.
* The current Nako addon boundary is out-of-process HTTP sidecars with scoped
  tokens, grants, health checks, install guides, hosted diagnostics, tasks,
  events, and resource-call diagnostics.

## Research Questions

* What do Jellyfin and Plex establish as baseline user expectations for a
  self-hosted media server?
* Which Jellyfin/Plex capabilities should Nako treat as table stakes for an M1
  video-first operator release?
* Where does Nako already have architectural/product differentiation?
* Where is Nako weaker because the product surface is not mature yet?
* How should official Addons be positioned relative to Jellyfin plugins and
  Plex's current extension story?
* Which product slices are worth prioritizing after current M1 release
  convergence?

## Requirements

* Use local repo docs, `repo-ref/jellyfin`, and `../nako-official-addons` as
  primary local sources.
* Use official Jellyfin and Plex public documentation for external claims when
  current information may have changed.
* Keep reference-code usage at the behavior/architecture level only. Do not
  copy source text, fixtures, schemas, or implementation details.
* Produce research artifacts under `research/`.
* Produce a concise competitive analysis suitable for product planning.

## Acceptance Criteria

* [x] Research artifacts exist under this task's `research/` directory.
* [x] Analysis compares Nako, Jellyfin, and Plex across product promise,
      library intake, metadata, playback, remote access, clients, and
      extension model.
* [x] Analysis identifies Nako's strongest differentiators and largest gaps.
* [x] Analysis recommends near-term product slices without pretending Nako is
      already feature-complete.
* [x] Sources are traceable to local files or official public documentation.

## Definition of Done

* Research files are written and linked from the final response.
* No code changes are made.
* Any Trellis task context added is limited to spec/research files.

## Out of Scope

* Implementing competitor-driven features.
* Jellyfin plugin compatibility.
* Plex-compatible APIs or clients.
* Copying Jellyfin source, tests, schemas, fixtures, docs, or generated files.
* Full market sizing, pricing analysis, or community sentiment analysis.

## Technical Notes

* Key local docs inspected:
  * `README.md`
  * `CONTEXT.md`
  * `docs/ARCHITECTURE.md`
  * `docs/ROADMAP.md`
  * `docs/GOALS.md`
  * `docs/architecture/LANES.md`
  * `docs/architecture/CONTROL_PLANE.md`
  * `docs/architecture/LIBRARY_PIPELINE.md`
  * `docs/addons/OFFICIAL_ADDON_CATALOG.md`
  * `../nako-official-addons/README.md`
  * `../nako-official-addons/addons/*/README.md`
  * `repo-ref/jellyfin/README.md`
* Research artifacts:
  * `research/nako-current-positioning.md`
  * `research/jellyfin-plex-competitive-landscape.md`
  * `research/competitive-analysis-summary.md`
