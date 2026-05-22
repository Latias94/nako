# Public Client API Contract Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M29 is closed. The first stable public browse/search/list/detail/probe and
playback decision DTO slice now lives in `nako-client-protocol`, while
`nako-api` remains the AGPL server adapter over internal records.

## Active Task

- None. The workstream is completed.

## Decisions Since Last Update

- Protocol IDs should be wire strings, not `nako-core` ID newtypes.
- Stable public enums can be duplicated in `nako-client-protocol` to keep the
  crate dependency-light.
- `nako-api` must use explicit mapping functions for protocol DTOs because
  `From<nako_core::...>` impls for protocol types would violate Rust orphan
  rules.
- Diagnostics, job internals, provider runtime details, webhook, automation,
  addon administration, ingestion failures, and metadata maintenance remain in
  `nako-api` for now.

## Blockers

- None.

## Next Recommended Action

- Choose a new goal. Good follow-ons are API versioning/error envelope
  hardening, client SDK generation, or continuing crate/module decomposition
  where a concrete client or server use case demands it.
