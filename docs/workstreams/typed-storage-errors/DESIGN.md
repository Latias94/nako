# Typed Storage Errors Design

Status: Completed
Last updated: 2026-05-17

## Problem

`taru-server` currently maps storage errors to public HTTP status codes and
client error codes by parsing `TaruError::Storage.message`. That is fragile:
WebDAV, local filesystem, staging, future S3/SMB/NAS adapters, and playback
staging can all phrase timeout, auth, rate limit, and validation failures
differently.

The current behavior is test-covered and useful, but the classification belongs
at the error source, not in the HTTP adapter.

## Target State

- `taru-core` owns a typed storage error classification.
- VFS/storage-facing code classifies storage failures when constructing
  `TaruError::Storage`.
- HTTP maps `StorageErrorKind` directly to status, public code, and public
  message.
- Existing public error codes and status codes remain stable.
- Generic storage failures still exist for IO and unknown backend errors.

## In Scope

- `crates/taru-core/src/error.rs`
- `crates/taru-vfs/src`
- storage/staging/playback file IO paths in `taru-server`
- storage-related tests in `taru-server` and `taru-vfs`
- workstream and goal documentation

## Out Of Scope

- No new S3, SMB, NAS, or cloud backend.
- No public API/OpenAPI/SDK/protocol expansion.
- No database schema changes.
- No NFO Round Trip preservation or link-management policy.
- No playback source-selection or transcode-plan redesign.
- No durable retry policy or storage health model redesign.

## Architecture Direction

Use a small enum in `taru-core`:

```text
StorageErrorKind:
  Unknown
  Io
  Network
  Timeout
  Unauthorized
  RateLimited
  HttpStatus
  StagingBudgetExhausted
  StagingValidationMismatch
  ResourceBudgetClosed
  SecurityViolation
```

The first slice should keep `TaruError::Storage` as the canonical storage
error variant and attach a `kind`. Helper constructors can keep call sites
readable. HTTP must stop parsing messages for storage classification.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `TaruError` can evolve because it is an internal server/workflow type, not a public wire DTO. | High | Public error DTOs live in `taru-api`/`taru-client-protocol`. | If external crates depend on the variant shape later, hide constructors behind helpers before publishing. |
| Public HTTP error codes should not change in M45. | High | Client protocol already treats these as stable wire values. | If a code changes, update protocol tests and treat it as a separate API contract goal. |
| WebDAV status and reqwest errors can be classified at construction sites. | High | Current code already centralizes retry and status handling in `webdav.rs`. | If a backend needs richer source errors, add backend-local mapping helpers. |

## Closeout Condition

This lane can close when:

- HTTP storage error mapping has no message-parsing classification helpers;
- current public storage error codes still pass;
- VFS/WebDAV/staging/playback source errors carry useful typed categories;
- focused and workspace gates pass;
- remaining storage health/retry follow-ons are documented.
