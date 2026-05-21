# Metadata Provider Attempt Runtime Design

Status: Completed
Last updated: 2026-05-17

## Problem

M40 introduced `MetadataRefreshPort` and `MetadataAttemptPort`, which moved
persistence details behind workflow-shaped Interfaces. The main refresh Module
is still shallow in a different way: `MetadataStrategyExecutor` owns provider
selection, provider search/fetch, attempt classification, raw response creation,
refresh commit, and catalog hydration orchestration in one file.

That makes later TMDB/Douban/Bangumi differences, provider auth, rate-limit
diagnostics, raw response diagnostics, and future **Metadata Scrape** work
harder to reason about. The interface is smaller than before, but the
implementation locality is still weak.

## Target State

- Provider attempt execution is an internal `taru-metadata` Module.
- `MetadataStrategyExecutor::refresh_item` keeps the existing public workflow
  shape and delegates provider-attempt runtime details.
- Attempt classification, skipped-provider attempts, provider search/fetch,
  raw response construction, and success/failure summaries live together.
- Refresh commit and catalog hydration remain explicit at the strategy level
  unless the implementation proves they belong behind the attempt runtime.
- Current provider behavior, attempt records, raw response caching, and
  provider mappings remain compatible.

## In Scope

- `crates/taru-metadata/src/strategy.rs` provider attempt logic.
- New internal `taru-metadata` Module files if they improve locality.
- Focused tests for provider attempt runtime behavior through existing public
  refresh Interfaces.
- Workstream and goal documentation.

## Out Of Scope

- No new TMDB, Douban, Bangumi, or addon provider breadth.
- No public HTTP API, OpenAPI, SDK, CLI, or protocol changes.
- No repository trait churn unless a real use case proves a narrower Interface.
- No database schema changes.
- No NFO Round Trip work.
- No playback/client-profile work.
- No `taru-api` module split.

## Architecture Direction

The first slice should create an internal provider-attempt runtime, not a new
public crate boundary. The strategy should become easier to read:

```text
load refresh snapshot
for each configured provider:
  run provider attempt runtime
  record attempt
  if accepted:
    commit refresh
    hydrate catalog
    return summary
return no-match/failure
```

The attempt runtime should own:

- registered-provider state handling;
- provider lookup/fetch;
- match kind classification;
- provider error classification;
- attempt DTO construction;
- raw response construction for successful results.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The first cut can keep `MetadataStrategyExecutor::refresh_item` externally compatible. | High | Current callers construct the executor and call `refresh_item`; tests focus on summaries and attempts. | If helper visibility changes leak out, restore the public shape and keep extraction internal. |
| Provider attempt runtime should stay inside `taru-metadata`. | High | It depends on metadata provider traits, merge policy, and provider-specific classifications. | If another crate needs it later, split a permissive protocol only after a real second consumer exists. |
| Repository port splits are not needed for this goal. | Medium | M40 already added `MetadataRefreshPort` and `MetadataAttemptPort`. | If commit logic still dominates, split a follow-on workflow port instead of broadening M44. |

## Closeout Condition

This lane can close when:

- provider attempt execution lives behind an internal Module with clear types;
- `MetadataStrategyExecutor` is thinner and delegates attempt execution;
- existing provider behavior and attempt diagnostics are covered by focused
  tests;
- no public API/SDK/protocol/database behavior changes are introduced;
- closeout gates pass and follow-ons are recorded.
