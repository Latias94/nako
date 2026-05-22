# Crate Boundary and Public Protocol Hardening

Status: Draft
Last updated: 2026-05-17

## Why This Lane Exists

Nako now has enough product surface that the current crate seams are starting
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
  - `crates/nako-api`
  - `crates/nako-core/src/media.rs`
  - `crates/nako-core/src/repository.rs`
  - `crates/nako-library/src/lib.rs`
  - `crates/nako-nfo/src/lib.rs`
  - `crates/nako-streaming/src/lib.rs`
  - `crates/nako-transcode/src/lib.rs`
  - `crates/nako-server/src/app/playback/*`

## Problem

- `nako-api` is the server HTTP DTO layer, but it currently depends directly
  on `nako-core` and `nako-streaming`, so the client-facing wire contract is
  not isolated.
- `crates/nako-core/src/media.rs` and `crates/nako-core/src/repository.rs`
  aggregate too many concepts into too few files.
- `crates/nako-library/src/lib.rs` and `crates/nako-nfo/src/lib.rs` are large
  workflow modules that mix unrelated responsibilities.
- `crates/nako-server/src/app/playback/*` is already split, but the
  orchestration contract between planning, runtime, and server composition is
  still concentrated in one area.

## Target State

- A permissive `nako-client-protocol` crate owns public client wire types.
- `nako-api` remains an AGPL server adapter that maps HTTP requests and
  responses onto server internals and the public protocol boundary.
- `nako-core` is deeper and easier to navigate because records and repository
  contracts are grouped by concept.
- `nako-library` and `nako-nfo` expose clearer module seams for scan, index,
  probe, codec, import, export, and workflow code.
- Playback responsibilities are explicit across `nako-streaming`,
  `nako-transcode`, and `nako-server`.
- Behavior stays stable while dependency direction and test locality improve.

## In Scope

- Public client protocol boundary design and extraction.
- `nako-core` module deepening without an immediate crate split.
- `nako-library` and `nako-nfo` module decomposition.
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
| Public client wire types should live outside server internals. | High | ADR 0022, `CONTEXT.md` public-client language, existing `nako-addon-protocol` precedent | Client SDKs will keep depending on server internals. |
| `nako-api` should remain the server adapter layer, not the long-term public protocol crate. | High | Current crate role and ADR 0022 | The public contract and server internals will stay coupled. |
| `nako-core` should be deepened by module before a new crate split is attempted. | High | Large `media.rs` and `repository.rs` files | A crate split would happen before the domain is easier to understand. |
| `nako-library`, `nako-nfo`, and playback can be decomposed safely before any major behavior change. | High | Current file sizes and already-separated playback submodules | The codebase will remain hard to localize when future features land. |

## Architecture Direction

Prefer seams that increase locality and leverage:

- Keep the public client wire boundary dependency-light and server-agnostic.
- Keep server adapters close to HTTP and composition concerns.
- Deepen `nako-core` and the workflow crates by moving concepts into smaller
  modules that match the project language in `CONTEXT.md`.
- Use the deletion test for each candidate seam: if deleting a module merely
  moves complexity elsewhere, it is not deep enough yet.

## Closeout Condition

This lane can close when:

- the public client protocol boundary is explicit,
- the core/library/NFO/playback module seams are narrower and documented,
- validation proves behavior stayed stable,
- and follow-on work is either split or explicitly deferred.
