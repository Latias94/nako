# Crate Boundary and Public Protocol Hardening Workstream

## Purpose

This workstream deepens Taru's crate and module seams after the M27 media
catalog expansion. It introduces a permissive public client protocol boundary
for future Flutter, web, and CLI consumers while keeping server internals
AGPL, and it makes the large workflow crates easier to navigate without
changing behavior.

## Status

In progress.

## Top-Level Tracking

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0022: Keep Public Protocol Crates Permissive While Server Crates Remain AGPL](../../adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Design](DESIGN.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Phase 28.0 crate boundary baseline](PHASE28_0_CRATE_BOUNDARY_BASELINE.md)

## Goals

- Separate public client wire types from server adapter DTOs.
- Keep `taru-api` as an AGPL server adapter layer, not the long-term public
  protocol crate.
- Introduce a permissive `taru-client-protocol` boundary when the public wire
  surface is stable enough.
- Deepen `taru-core` by module instead of immediately splitting it into more
  crates.
- Split `taru-library` and `taru-nfo` into focused internal modules.
- Clarify `taru-streaming`, `taru-transcode`, and `taru-server` playback
  ownership.
- Preserve behavior while tightening dependency direction and test locality.

## Non-Goals

- No Flutter, web, or CLI client implementation.
- No public API behavior rewrite unless a wire-type move requires a mechanical
  mapping.
- No metadata-catalog semantic redesign.
- No HLS ladder or optimized-version work.
- No immediate full workspace crate split.

## Boundary Rules

- `taru-client-protocol` (future crate) holds permissive, dependency-light
  public client wire types.
- `taru-api` stays the AGPL server adapter layer and may map to or re-export
  public protocol types, but it should not become the long-term public wire
  contract crate.
- `taru-core` owns durable domain types and repository contracts; internal
  module decomposition comes before any new crate split.
- `taru-library` owns scan, index, probe, and local-inference orchestration.
- `taru-nfo` owns NFO codec and workflow boundaries.
- `taru-streaming` owns playback decision planning.
- `taru-transcode` owns FFmpeg/runtime/session orchestration.
- `taru-server::app` owns composition and HTTP translation.
- Public protocol crates must remain dependency-light and must not depend on
  AGPL server crates or internal server domain models.

## Related Workstreams

- [metadata-catalog](../metadata-catalog/README.md): completed M27
  media-library domain expansion.
- [server-architecture-hardening](../server-architecture-hardening/README.md):
  completed server composition cleanup.
- [playback-streaming](../playback-streaming/README.md): completed remote
  streaming boundary work.
- [transcode-runtime](../transcode-runtime/README.md): completed playback
  runtime productization.
