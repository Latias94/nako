# Crate Boundary and Public Protocol Hardening

Status: Draft
Last updated: 2026-05-17

## Why This Lane Exists

Taru now has enough product surface that the current crate seams are starting
to hide meaning rather than explain it. The public client contract should be
reusable without dragging server internals with it, and the core, library,
NFO, and playback seams should become shallower so future refactors stay
local.

## Relevant Authority

- ADRs:
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/GOALS.md`
  - `docs/ROADMAP.md`
  - `docs/README.md`
  - `docs/workstreams/metadata-catalog/README.md`
  - `docs/workstreams/server-architecture-hardening/README.md`
  - `docs/workstreams/playback-streaming/README.md`
  - `docs/workstreams/transcode-runtime/README.md`
- Related crates and modules:
  - `crates/taru-api`
  - `crates/taru-core/src/media.rs`
  - `crates/taru-core/src/repository.rs`
  - `crates/taru-library/src/lib.rs`
  - `crates/taru-nfo/src/lib.rs`
  - `crates/taru-streaming/src/lib.rs`
  - `crates/taru-transcode/src/lib.rs`
  - `crates/taru-server/src/app/playback/*`

## Problem

- `taru-api` is the server HTTP DTO layer, but it currently depends directly
  on `taru-core` and `taru-streaming`, so the client-facing wire contract is
  not isolated.
- `crates/taru-core/src/media.rs` and `crates/taru-core/src/repository.rs`
  aggregate too many concepts into too few files.
- `crates/taru-library/src/lib.rs` and `crates/taru-nfo/src/lib.rs` are large
  workflow modules that mix unrelated responsibilities.
- `crates/taru-server/src/app/playback/*` is already split, but the
  orchestration contract between planning, runtime, and server composition is
  still concentrated in one area.

## Target State

- A permissive `taru-client-protocol` crate owns public client wire types.
- `taru-api` remains an AGPL server adapter that maps HTTP requests and
  responses onto server internals and the public protocol boundary.
- `taru-core` is deeper and easier to navigate because records and repository
  contracts are grouped by concept.
- `taru-library` and `taru-nfo` expose clearer module seams for scan, index,
  probe, codec, import, export, and workflow code.
- Playback responsibilities are explicit across `taru-streaming`,
  `taru-transcode`, and `taru-server`.
- Behavior stays stable while dependency direction and test locality improve.

## In Scope

- Public client protocol boundary design and extraction.
- `taru-core` module deepening without an immediate crate split.
- `taru-library` and `taru-nfo` module decomposition.
- Playback seam clarification across planning, runtime, and server
  orchestration.
- Doc and test updates that prove the new seams without changing product
  behavior.

## Out Of Scope

- Flutter, web, or CLI client implementation.
- Public API behavior changes unrelated to mechanical wire-type movement.
- Metadata-catalog semantic redesign.
- HLS ladder or optimized-version work.
- Immediate workspace-wide crate splitting.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public client wire types should live outside server internals. | High | ADR 0022, `CONTEXT.md` public-client language, existing `taru-addon-protocol` precedent | Client SDKs will keep depending on server internals. |
| `taru-api` should remain the server adapter layer, not the long-term public protocol crate. | High | Current crate role and ADR 0022 | The public contract and server internals will stay coupled. |
| `taru-core` should be deepened by module before a new crate split is attempted. | High | Large `media.rs` and `repository.rs` files | A crate split would happen before the domain is easier to understand. |
| `taru-library`, `taru-nfo`, and playback can be decomposed safely before any major behavior change. | High | Current file sizes and already-separated playback submodules | The codebase will remain hard to localize when future features land. |

## Architecture Direction

Prefer seams that increase locality and leverage:

- Keep the public client wire boundary dependency-light and server-agnostic.
- Keep server adapters close to HTTP and composition concerns.
- Deepen `taru-core` and the workflow crates by moving concepts into smaller
  modules that match the project language in `CONTEXT.md`.
- Use the deletion test for each candidate seam: if deleting a module merely
  moves complexity elsewhere, it is not deep enough yet.

## Closeout Condition

This lane can close when:

- the public client protocol boundary is explicit,
- the core/library/NFO/playback module seams are narrower and documented,
- validation proves behavior stayed stable,
- and follow-on work is either split or explicitly deferred.
