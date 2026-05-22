# Typed Storage Errors Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M45 is complete. Storage errors carry `StorageErrorKind`, HTTP maps storage
errors by kind, and VFS/staging/playback sources classify storage errors at
construction time.

## Follow-Ons

- Split `nako-api` by public client, admin, metadata diagnostics, extension,
  and automation DTO modules.
- Design NFO Round Trip preservation before managed sidecar writes and link
  policy expand.
- Extend typed storage diagnostics if future S3/SMB/NAS adapters need
  backend-specific classification detail.

## Cautions

- Public client error codes and HTTP statuses were preserved.
- No new storage backend was added.
- OpenAPI/SDK/protocol contracts were not expanded.
- NFO Round Trip and playback source-selection work remain separate.
