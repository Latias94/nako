# Metadata Refresh Workflow Port And Provider Runtime Seam

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M39 proved that catalog hydration becomes easier to reason about when callers
depend on a workflow-shaped port instead of a broad repository trait
combination. The next high-friction boundary is metadata refresh:
`nako-metadata` currently coordinates item loading, provider mapping, raw
response cache, attempt records, field locks, refresh application, hierarchy
confirmation, and catalog hydration from one strategy surface.

The problem is not that refresh has many steps. The problem is that callers and
tests must understand too many persistence and provider-runtime details just to
exercise one refresh workflow.

## Target State

- Metadata refresh has a use-case-shaped port that hides low-level persistence
  details behind workflow operations.
- The first slice keeps provider behavior unchanged and focuses on the refresh
  strategy seam.
- `nako-metadata` can test core refresh behavior with a small fake adapter,
  while SQLite behavior remains covered by existing integration tests.
- Catalog hydration continues to flow through M39's `CatalogHydrationPort`.
- Provider breadth, public APIs, SDKs, NFO Round Trip, and playback stay out of
  this lane.

## In Scope

- Open a durable M40 workstream with design, task, milestone, evidence, and
  handoff docs.
- Audit `strategy.rs`, `confirmation.rs`, provider runtime, raw cache/attempt
  writes, and metadata repository trait usage.
- Extract one vertical metadata refresh workflow port when it hides meaningful
  behavior rather than merely renaming existing repository methods.
- Add focused fake-port behavior tests for the new seam.
- Preserve existing SQLite-backed behavior tests.
- Update GOALS, ROADMAP, and the workstream index.

## Out Of Scope

- TMDB, Douban, Bangumi feature breadth or new metadata fields.
- Public Client API, OpenAPI, Rust SDK, TypeScript SDK, CLI, or license
  boundary changes.
- NFO Round Trip preservation, unknown XML retention, partial NFO updates, or
  soft/hard-link policy.
- Playback source selection, transcode planning, client profiles, or adaptive
  HLS.
- Database schema changes unless the first slice proves an existing behavior
  cannot be represented.
- Mechanical splitting of every `MetadataRepository` method.

## Architecture Direction

Use ports only when the interface names a real workflow capability. For the
first slice, the likely shape is a `MetadataRefreshPort` or equivalent that can
load the refresh subject, record provider attempts/raw responses, apply the
accepted metadata refresh, and expose the data needed by mapping/confirmation
without making the strategy depend directly on every repository detail.

Do not hide provider clients behind the same port. Provider lookup/fetch is a
separate runtime concern and should only be adjusted if the first refresh seam
needs a smaller `ProviderResolutionPort`.

## First Slice Decision

M40.1 chooses `MetadataRefreshPort` plus `MetadataAttemptPort` inside
`nako-metadata`.

`MetadataRefreshPort` owns the refresh persistence workflow:

- load a `MetadataRefreshSnapshot` with the **Media Item** and field locks;
- commit a `MetadataRefreshCommit` containing the updated **Media Item** and
  raw provider response;
- keep provider subject, provider mapping, and library-item confirmation
  persistence behind the commit operation.

`MetadataAttemptPort` records provider-attempt diagnostics without requiring
the strategy to depend directly on `MetadataRepository`.

Provider lookup/fetch stays outside the port because provider clients are a
runtime seam, not repository persistence. Catalog graph/search updates stay
behind M39's `CatalogHydrationPort`.

## Closeout Condition

M40 can close when:

- the workstream is documented and indexed;
- one metadata refresh workflow seam is deeper than before;
- focused fake-port tests prove the new port contract;
- existing metadata/provider/NFO behavior still passes;
- workspace validation gates pass;
- follow-on seams such as provider runtime, metadata maintenance, or library
  scan/probe are split clearly.
