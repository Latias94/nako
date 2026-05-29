# Subtitle Import Apply Milestones

## M1 - Contract

- Admin API exposes a typed import-apply request and response.
- TypeScript generated contract includes the route and DTOs.

## M2 - Apply Boundary

- Import apply recomputes a ready plan before mutation.
- Stale idempotency keys are rejected.
- Inline and download-url delivery can be written safely.
- Artifact-ref delivery remains explicitly unsupported.

## M3 - File Write Safety

- Sidecar target is derived from the media source locator.
- `create_missing` does not overwrite different existing content.
- `replace_existing` uses storage atomic replace where supported.
- Backup reporting is redacted.

## M4 - Closeout

- Focused API and server tests pass.
- Workstream evidence is recorded.
- Remaining subtitle fact refresh work is split.
