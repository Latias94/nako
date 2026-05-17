# Metadata Provider Attempt Runtime Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M44 is complete. `taru-metadata` now has an internal provider attempt runtime
Module, and `MetadataStrategyExecutor` delegates provider attempt execution and
classification while preserving existing refresh behavior.

## Follow-Ons

- Split `taru-api` by public client, admin, metadata diagnostics, and extension
  DTO modules without changing `taru-client-protocol`.
- Add typed VFS/storage error classification before S3/SMB/NAS adapters grow
  string-based HTTP mapping further.
- Design NFO Round Trip preservation before managed file write/link behavior.

## Cautions

- M44 intentionally did not add provider breadth.
- M44 did not change public HTTP/OpenAPI/SDK/protocol contracts.
- M44 did not change repository traits, NFO, playback, or database schema.
