# Taru Goal Map

This file is the top-level tracker for current and upcoming engineering goals.
Workstream TODO files track tasks; this file tracks goal boundaries,
non-goals, exit criteria, and evidence.

## Goal Format

Each implementation goal should define:

- Objective: the user-visible or architecture-visible outcome.
- Deliverables: concrete files, APIs, crates, or documents expected to change.
- Non-goals: adjacent work intentionally left out.
- Exit criteria: observable behavior that must be true.
- Evidence: commands, tests, docs, or commit IDs proving completion.

Use one goal per meaningful milestone. A goal should be large enough to produce
a coherent commit, but small enough that validation remains clear.

## Completed Goals

### M0-M2.1: Server Runtime Foundation

Status: completed.

Evidence:

- Rust workspace and crate stubs exist.
- SQLite persistence, server runtime, persisted jobs, pagination, logging, and
  developer docs are implemented.
- Related docs: [server-foundation milestones](workstreams/server-foundation/MILESTONES.md).

### M3.1-M3.6: Metadata, NFO, Profiles, and Catalog Planning

Status: completed for the first movie-focused foundation.

Evidence:

- Metadata merge policy, NFO policy, provider secret policy, library presets,
  catalog graph, scan state, and artwork resource-class ADRs exist.
- TMDB movie refresh, NFO import/export jobs, metadata profile execution, and
  catalog/search planning are implemented or documented.

### M4.0-M4.4: Catalog Ingestion and Playback Foundation

Status: completed through M4.4.

Evidence:

- Catalog ingestion, graph hydration, browse APIs, direct play, FFmpeg command
  planning, and remux process runner guard are implemented.
- Last completed implementation goal: M4.4 remux process runner and runtime
  resource guard.

## Recently Completed Planning Goal

### Planning Docs: Goal Map and Refactoring Policy

Status: completed.

Objective:

- Give the project a single top-level route for roadmap, goal tracking,
  workstream ownership, and fearless refactoring policy.

Deliverables:

- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`
- `docs/development/REFACTORING_POLICY.md`
- server-foundation milestone and TODO updates

Non-goals:

- no runtime code changes;
- no ADR status migration beyond documenting the hygiene rule;
- no workstream directory split yet.

Exit criteria:

- top-level docs link to current focus, roadmap, and active workstream;
- the next recommended implementation goal is explicit;
- refactoring policy documents crate boundaries, dependency direction, and
  validation gates;
- doc consistency checks pass.

Evidence:

- `git diff --check` passed for the docs-only change set.

## Recommended Next Implementation Goal

### M4.5: Remux App Service Integration and Local Staging Policy

Status: proposed.

Objective:

- Move remux orchestration behind an application service that can later serve
  HTTP remux playback without exposing process-runner details to handlers.

Deliverables:

- application-level remux service in `taru-server::app`;
- local staging directory policy for remux outputs;
- deterministic output naming and cleanup rules;
- duplicate request reuse or explicit idempotency behavior;
- error mapping from transcode/remux failures to API-safe errors;
- tests using fake runners or command planning where possible.

Non-goals:

- no public remux playback HTTP route yet;
- no HLS segment serving;
- no hardware acceleration detection;
- no remote-source staging;
- no persisted transcode session table unless needed for clean service shape.

Exit criteria:

- server composition can create and call the remux app service;
- staging paths cannot escape the configured staging root;
- duplicate remux requests have deterministic behavior;
- runner errors, cancellation, timeout, and invalid requests are mapped without
  leaking process internals;
- `cargo fmt --all -- --check`, `cargo check --workspace`,
  `cargo nextest run --workspace`, and `git diff --check` pass.

## Later Goals

### M4.6: Remux Playback Route

Expose remuxed playback through an HTTP route after M4.5 has a clean
application boundary. This should stream a staged remux output or return a
clear pending/error state without blocking request handlers indefinitely.

### M4.7: Transcode Session Persistence

Persist remux/transcode session records so cancellation, retry, observability,
and later multi-client playback can survive process restarts.

### M4.8: HLS Transcode Foundation

Add HLS playlist/segment planning and process lifecycle management. Keep
hardware acceleration optional until the queue and resource model are explicit.

### M5: Extension and Automation Surface

Implement webhook outbox, automation jobs, addon manifest schema, and one
reference addon.

### M6: Remote Storage Preview

Implement one limited remote storage backend and prove scan/probe/playback can
work through `taru-vfs` without local-path assumptions.
