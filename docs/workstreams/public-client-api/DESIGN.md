# Public Client API Contract

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M28 created `taru-client-protocol`, but the useful client-facing library,
catalog, search, and playback DTOs still live in `taru-api` and directly embed
server/internal domain types. Future Flutter, web, and CLI clients need a
stable permissive wire contract without depending on AGPL server internals.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/GOALS.md`
  - `docs/ROADMAP.md`
  - `docs/workstreams/crate-boundary-hardening/`
  - `docs/workstreams/metadata-catalog/`
  - `docs/workstreams/transcode-runtime/`
- Related crates and modules:
  - `crates/taru-client-protocol`
  - `crates/taru-api`
  - `crates/taru-server/src/app/catalog.rs`
  - `crates/taru-server/src/app/library.rs`
  - `crates/taru-server/src/app/playback/*`
  - `crates/taru-server/src/http/catalog.rs`
  - `crates/taru-server/src/http/library.rs`
  - `crates/taru-server/src/http/playback.rs`

## Problem

- `taru-api` owns DTOs that are useful to clients, but those DTOs use
  `taru-core` IDs, enums, and records directly.
- `PlaybackDecisionResponse` exposes `taru_streaming::PlaybackDecision`, which
  is a server playback planning type, not a public protocol type.
- Public browse/search/list/detail routes exist, but their contract is not
  separated from server-admin diagnostics and internal job/provider state.
- Moving DTO structs alone is not enough because Rust orphan rules prevent
  `taru-api` from implementing `From<taru_core::...>` for protocol crate
  types. The adapter must use explicit mapping functions.

## Target State

- `taru-client-protocol` owns the first stable public client library, catalog,
  search, and playback wire types.
- Protocol IDs are wire strings, not `taru-core` ID newtypes.
- Public protocol enums are duplicated deliberately when they are part of the
  client contract.
- `taru-api` remains the AGPL adapter layer that maps server/internal records
  into public protocol DTOs.
- Server-admin/internal DTOs, diagnostics, job internals, provider runtime
  state, webhook, automation, and addon administration remain in `taru-api`.
- `cargo tree -p taru-client-protocol` proves the protocol crate does not
  depend on `taru-core`, `taru-streaming`, `taru-transcode`, or `taru-server`.

## In Scope

- Migrate the first stable `Library`, `MediaItem`, `MediaSource`,
  `MediaProbe`, search, list, detail, and playback decision response DTOs into
  `taru-client-protocol`.
- Define public browse/search/list/detail response envelopes.
- Add explicit adapter functions in `taru-api` for core/streaming/transcode
  records.
- Keep existing route behavior stable while changing the ownership of DTO
  definitions.
- Update docs and tests proving the public protocol boundary.

## Out Of Scope

- Flutter, web, or CLI client implementation.
- Authentication/authorization redesign.
- Server-admin/provider diagnostics migration.
- Job internals, webhook, automation, addon, ingestion failure, and metadata
  maintenance DTO migration.
- Adaptive bitrate HLS ladder or optimized-version work.
- Breaking existing route paths unless a separate API-versioning decision is
  accepted.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public DTOs should not expose `taru-core` ID newtypes. | High | ADR 0022 and M28 protocol boundary | Client crates would still depend on server internals. |
| `taru-api` should use explicit conversion functions instead of `From` impls for protocol DTOs. | High | Rust orphan rules for foreign trait + foreign DTO/source type pairs | DTO migration would fail to compile or require wrapper types. |
| Browse/search/list/detail is the first useful client surface. | High | Existing server routes and M27 catalog domain work | Protocol migration would produce little value for clients. |
| Admin diagnostics should stay out of the first protocol crate expansion. | High | User scope and ADR 0022 | Public clients would inherit unstable operational internals. |

## Architecture Direction

Keep protocol types simple, serializable, and server-agnostic. Use `String`
for public IDs and duplicated protocol enums for stable client vocabulary.
Keep all mappings from `taru-core`, `taru-streaming`, and `taru-transcode` in
`taru-api`, so external clients depend only on `taru-client-protocol`.

The first implementation should prefer a small useful slice over a complete
API migration: library listing, source listing, item list/detail, search, probe
summary, and playback decision response. Server-only diagnostics remain in
place until a later migration has a concrete client need.

## Closeout Condition

This lane can close when:

- the selected public browse/list/detail/search/playback DTOs live in
  `taru-client-protocol`,
- `taru-api` maps internal records into those DTOs without leaking internal
  crate dependencies into the protocol crate,
- server route tests still validate the shipped JSON contracts,
- `cargo tree -p taru-client-protocol` confirms dependency direction,
- full validation gates pass,
- and follow-ons are explicitly recorded.

## Closeout Summary

M29 closed after moving the selected public browse/list/detail/search/probe
and playback decision DTOs into `taru-client-protocol`. `taru-api` now owns
explicit mapping functions from `taru-core`, `taru-streaming`, and
`taru-transcode` records into protocol wire DTOs. Server route tests preserve
the catalog, library, system, and playback JSON behavior while
`cargo tree -p taru-client-protocol` proves the protocol crate remains
dependency-light.
