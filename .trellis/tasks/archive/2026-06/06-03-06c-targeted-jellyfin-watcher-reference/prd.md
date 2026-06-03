# Targeted Jellyfin Watcher Reference

## Goal

Produce a narrow, behavior-level Jellyfin watcher reference note that answers
only the decisions needed to de-risk Nako's watcher runtime productization lane.

## Context

The user selected Option A for the follow-on wave. This third lane is research
support for 06a, not implementation work and not a broad Jellyfin comparison.
Jellyfin reference material is GPL; use it only to understand behavior and
architecture pressure. Do not copy, translate, or port implementation code,
comments, tests, schemas, or generated artifacts.

## Scope

Review only watcher/event/debounce material that can inform Nako decisions:

* library monitor lifecycle and start/stop boundaries;
* event kinds and event coalescing behavior;
* debounce or delay semantics around file changes;
* suppression of watcher events during planned host writes;
* fallback scan/reconciliation behavior after unreliable watcher events;
* operator-facing configuration or capability flags that affect watcher
  product expectations.

Expected output is a concise `research.md` or `evidence.md` in this task
directory with:

* source paths and short behavior summaries;
* Nako decision implications for 06a;
* explicit "do not copy" licensing note;
* unresolved questions that 06a should answer in Nako-native terms.

## Non-Goals

* No Nako implementation code changes.
* No `docs/architecture` edits.
* No broad Jellyfin audit.
* No Jellyfin playback, metadata provider, plugin, database, API, web UI, or
  deployment comparison.
* No source-code port, line-by-line translation, copied tests, copied comments,
  copied schemas, or copied generated artifacts.
* No recommendation that overrides Nako's ADR 0053 control-plane boundary.

## Acceptance Criteria

* [ ] The research output covers only watcher/event/debounce/reference files.
* [ ] Every observation is behavior-level and cites source paths without copying
      implementation code.
* [ ] The output answers at least these 06a decision questions:
      watcher lifecycle, debounce delay, event coalescing, planned-write
      suppression, and fallback reconciliation.
* [ ] The output clearly states what Nako should decide independently instead
      of following Jellyfin.
* [ ] The lane does not modify Nako code or architecture docs.

## Suggested Gates

* `git diff --check`
* Manual licensing review against `docs/legal/LICENSING.md`
* Confirm changed files are limited to this task directory unless the parent
  planner explicitly approves otherwise

## Coordination Notes

* Feed concise findings to 06a, but do not block 06a on a full reference pass.
* Stop if the research starts expanding beyond watcher/event/debounce semantics.
