# Phase 28.0: Crate Boundary Baseline

## Status

Baseline recorded.

## Objective

Document the crate and module seams that should be deepened in M28 without
changing behavior.

## Audit Summary

- `crates/taru-api/Cargo.toml` depends on `taru-core` and `taru-streaming`,
  so the public client DTO layer is still coupled to server internals.
- `crates/taru-core/src/media.rs` is 1,353 lines and mixes media library,
  item, source, metadata, artwork, and scan-state concepts.
- `crates/taru-core/src/repository.rs` is 784 lines and aggregates many
  repository traits in one seam.
- `crates/taru-library/src/lib.rs` is 2,584 lines and mixes scan, index,
  probe, local inference, summary, and source lifecycle logic.
- `crates/taru-nfo/src/lib.rs` is 1,508 lines and mixes codec, import,
  export, hierarchy confirmation, and workflow logic.
- `crates/taru-streaming/src/lib.rs` is already small and focused on playback
  decision planning.
- `crates/taru-transcode/src/lib.rs` owns FFmpeg/runtime/session logic.
- `crates/taru-server/src/app/playback/*` already has submodules, which makes
  it a good place to sharpen the final orchestration seam.
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  gives the license boundary for future public protocol crates.
- `taru-addon-protocol` is a useful precedent for a permissive protocol crate
  boundary.

## First Proof Candidate

Extract the first public client wire slice into a permissive
`taru-client-protocol` crate, then keep `taru-api` as the AGPL server adapter
layer that maps to it.

## Follow-On Map

- Public client wire types and server adapter DTOs should diverge first.
- `taru-core` should be deepened by module next.
- `taru-library` and `taru-nfo` should be split into workflow modules after
  that.
- Playback ownership should be clarified once the workflow crates are easier
  to read.
