# Douban TV Episode Endpoint Depth

## Goal

Extend Douban metadata support from movie-only to endpoint-backed TV series
subjects without overclaiming Season/Episode or hierarchy mutation support.

## Requirements

- Add Douban `Series` search/fetch support only when the provider response is a
  TV subject (`subtype: "tv"`).
- Keep Douban `Season` and `Episode` search/fetch unsupported until a dedicated
  endpoint-backed contract is proven.
- Keep Douban hierarchy support disabled in provider capabilities for this
  slice; Series fetch should return a root-only Candidate Graph.
- Preserve existing Douban movie behavior and API key/header redaction.
- Keep raw provider payloads out of public/admin/API surfaces; this task only
  changes `nako-metadata`.

## Acceptance Criteria

- [ ] Douban capabilities include `MediaKind::Series` and
  `ProviderSubjectKind::Series`, but still exclude `Season` and `Episode`.
- [ ] Douban Series search reaches the mocked Douban HTTP runtime and returns
  only TV-subtype candidates with `ProviderSubjectKind::Series`.
- [ ] Douban Series fetch maps a TV-subtype subject to a root-only Series
  candidate graph.
- [ ] Douban Season/Episode search and fetch still fail before HTTP calls.
- [ ] Existing Douban movie tests still pass.

## Definition of Done

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata douban --no-fail-fast`
- `cargo check -p nako-core -p nako-metadata --tests`
- `python ./.trellis/scripts/task.py validate 06-05-douban-tv-episode-endpoint-depth`
- `git diff --check`

## Technical Approach

- Reuse Douban's current `movie/search` and `movie/subject/{id}` endpoint
  adapter, because the documented subject payload distinguishes `subtype:
  "movie"` from `subtype: "tv"`.
- Add `subtype` to `DoubanSubject` and route expected media kind through a
  subtype guard:
  - `Movie` accepts movie or missing subtype for backward-compatible fixtures;
  - `Series` accepts only `tv`;
  - `Unknown` preserves the existing broad behavior.
- Change Douban provider capabilities to honestly advertise subject-level
  Series support, not Season/Episode or hierarchy support.
- Keep candidate graph creation root-only for Series.

## Decision (ADR-lite)

**Context**: The closed Douban subject-kind precision lane intentionally removed
Series/Season/Episode claims while the adapter used movie-oriented endpoints
without subtype validation.

**Decision**: Restore only the endpoint-backed TV subject slice by validating
Douban `subtype: "tv"` and mapping it to Nako `Series`. Defer Season/Episode
and related graph preview.

**Consequences**: Nako can match TV series through Douban without implying
episode-level depth. Future episode work must prove a separate endpoint and
mapping contract.

## Out of Scope

- No Douban Season or Episode fetch/search support.
- No child graph preview, accepted hierarchy application, or Provider Mapping
  child writes.
- No API/Admin/Web/generated contract changes.
- No schema migration or durable review behavior change.

## Technical Notes

- Local evidence:
  - `docs/workstreams/douban-subject-kind-precision/`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `.trellis/tasks/archive/2026-06/06-04-playback-renderer-transport-flow-extraction/research/next-parallel-feature-candidates.md`
- Implementation target:
  - `crates/nako-metadata/src/providers/douban.rs`
  - `crates/nako-metadata/src/mapping/douban.rs`
  - `crates/nako-metadata/src/tests.rs`
- Research detail:
  `research/douban-tv-endpoint-contract.md`.
