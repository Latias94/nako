# 10-hour Media Server Architecture Campaign

## Goal

Plan a 10-hour Codex goal for Nako's self-hosted media server features: identify
parallel architecture lanes where sub-agents can perform read-only inspection,
then turn the findings into a fearless refactor and development campaign with
clear sequencing, validation gates, and stop conditions.

## What I Already Know

* The user wants to stop considering Extism for now and focus on Nako's core
  self-hosted media-server functionality.
* The user explicitly asked for sub-agents to inspect architecture.
* The output should explain how to plan a hypothetical 10-hour goal, including
  parallel tasks for fearless refactoring and feature development.
* Current repo guidance treats Addons as Sidecars by default, keeps Playback
  Runtime owned by Nako, and uses Trellis tasks/specs for durable planning.
* `main` is clean before this planning task starts.

## Assumptions

* This is a planning and architecture-inspection task, not implementation.
* The 10-hour goal should maximize useful progress while avoiding schema/API
  churn unless evidence justifies it.
* Parallel work should use independent read-only architecture inspections first,
  then converge into a sequenced campaign.

## Requirements

* Use sub-agents for independent architecture inspection.
* Cover the main Nako media-server capability areas rather than only addon
  architecture.
* Produce a practical 10-hour plan with parallel lanes, serial gates,
  validation commands, and decision points.
* Identify which tasks are safe for parallel workers and which must remain
  serial because of shared contracts or high blast radius.
* Preserve Nako domain language from `CONTEXT.md`.

## Acceptance Criteria

* [x] At least four architecture inspection lanes report findings.
* [x] The final plan ranks work by user-visible value, risk reduction, and
      parallelizability.
* [x] The plan includes a 10-hour timeline with worker prompts and gates.
* [x] The plan distinguishes refactor-only work from feature-development work.
* [x] The plan calls out ADR/spec/task updates required before implementation.

## Research Artifacts

* `research/library-metadata-catalog.md`
* `research/playback-transcode-streaming.md`
* `research/storage-vfs-operations.md`
* `research/addon-control-plane.md`
* `campaign-plan.md`

## Definition of Done

* Sub-agent findings are summarized with file/path evidence.
* No code changes are made during inspection.
* Follow-on implementation tasks can be opened from the plan without redoing the
  architecture reconnaissance.
* Trellis context is valid.

## Out of Scope

* Implementing the selected refactors or features in this task.
* Reopening the Extism/plugin-runtime decision.
* Public API, schema, or dependency changes without a separate task/ADR.

## Technical Notes

* Relevant architecture docs likely include `docs/architecture/*.md`,
  `docs/adr/*.md`, `CONTEXT.md`, and `.trellis/spec/`.
* Candidate lanes to inspect:
  * Playback / Transcode / Streaming.
  * Library / Metadata / Catalog / NFO.
  * Storage / VFS / Operations.
  * Addon / Automation / Control Plane.
  * Client/API/Admin surfaces if time allows.
