# Metadata Application Cross-Path Audit - TODO

Status: Complete
Last updated: 2026-05-26

## Tasks

- [x] MACPA-010 - Audit provider refresh application path.
- [x] MACPA-020 - Audit hierarchy confirmation application path.
- [x] MACPA-030 - Compare both paths with server `MetadataApplication`.
- [x] MACPA-040 - Record dependency-boundary decision.

## Decision

No code extraction in this lane. The correct shared boundary is
`nako-core::MetadataMergePolicy`; the server `MetadataApplication` remains
server-owned.
