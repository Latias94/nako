# Phase 28.0: Crate Boundary Baseline

## Status

Baseline recorded.

## Objective

Document the crate and module seams that should be deepened in M28 without
changing behavior.

## Audit Summary

- `crates/nako-api/Cargo.toml` depends on `nako-core` and `nako-streaming`,
  so the public client DTO layer is still coupled to server internals.
- `crates/nako-core/src/media.rs` is 1,353 lines and mixes media library,
  item, source, metadata, artwork, and scan-state concepts.
- `crates/nako-core/src/repository.rs` is 784 lines and aggregates many
  repository traits in one seam.
- `crates/nako-library/src/lib.rs` is 2,584 lines and mixes scan, index,
  probe, local inference, summary, and source lifecycle logic.
- `crates/nako-nfo/src/lib.rs` is 1,508 lines and mixes codec, import,
  export, hierarchy confirmation, and workflow logic.
- `crates/nako-streaming/src/lib.rs` is already small and focused on playback
  decision planning.
- `crates/nako-transcode/src/lib.rs` owns FFmpeg/runtime/session logic.
- `crates/nako-server/src/app/playback/*` already has submodules, which makes
  it a good place to sharpen the final orchestration seam.
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  gives the license boundary for future public protocol crates.
- `nako-addon-protocol` is a useful precedent for a permissive protocol crate
  boundary.

## First Proof Candidate

Extract the first public client wire slice into a permissive
`nako-client-protocol` crate, then keep `nako-api` as the AGPL server adapter
layer that maps to it.

## Follow-On Map

- Public client wire types and server adapter DTOs should diverge first.
- `nako-core` should be deepened by module next.
- `nako-library` and `nako-nfo` should be split into workflow modules after
  that.
- Playback ownership should be clarified once the workflow crates are easier
  to read.
