# Android Generated Public Client SDK

Status: Completed
Last updated: 2026-05-21

This completed workstream moved Android away from handwritten Public Client API
DTO and route mirrors by introducing a generated Kotlin/JVM SDK from Taru's Public
OpenAPI v1 contract.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [CLOSEOUT.md](CLOSEOUT.md)

## Decision Spine

- ADR 0025 makes the generated Public OpenAPI v1 contract the SDK authority.
- ADR 0026 keeps Android playback native while allowing shared client logic.
- ADR 0031 sequences generated Kotlin SDK adoption before mobile Rust/UniFFI.

## Delivered Shape

The lane delivered a checked-in `sdk/kotlin` package whose generated source is
produced by `taru-api`, compile-checked by Gradle, validated against the same
Public Client API leakage rules as the OpenAPI and TypeScript SDK lanes, and
consumed by Android connection, browse, media probe, playback, artwork, and
user-playback client seams through generated request descriptors and DTO
adapters.
