# nako-api Module Split Design

Status: Completed
Last updated: 2026-05-17

## Problem

`crates/nako-api/src/lib.rs` mixes several different surfaces:

- stable Public Client API protocol re-exports and server-to-client DTO
  adapters;
- server admin/internal response DTOs such as jobs, ingestion failures, and
  storage diagnostics;
- metadata provider diagnostics and maintenance request/response DTOs;
- extension surfaces for webhooks, automation providers, and addons.

`nako-client-protocol` already owns the permissive Public Client API wire types.
Keeping every adapter and internal DTO in the `nako-api` crate root makes it
harder to see which symbols are stable public client contract and which symbols
are server/admin integration details.

## Target State

`nako-api` stays the AGPL server adapter/schema aggregation crate, but its crate
root becomes a compatibility facade:

- `public_client`: Public Client protocol re-exports and server model mapping
  functions.
- `admin`: server admin/internal DTOs that are not part of the stable Public
  Client API.
- `metadata_diagnostics`: metadata maintenance and provider diagnostic DTOs.
- `extension`: webhook, automation, and addon request/response DTOs.
- `openapi` and `sdk` continue to generate Public Client API artifacts.

Root-level `pub use` compatibility is preserved in this slice to avoid forcing
server call-site churn at the same time as the module split.

## Scope

- Split `crates/nako-api/src/lib.rs` into explicit modules.
- Preserve root-level re-exports for existing callers.
- Preserve OpenAPI JSON and TypeScript SDK output.
- Preserve Public Client API JSON shapes and existing HTTP behavior.
- Move `nako-api` unit tests out of the crate root if useful.
- Update `docs/GOALS.md` and workstream evidence.

## Non-goals

- No DTO ownership migration into `nako-client-protocol`.
- No new HTTP routes, route renames, or JSON shape changes.
- No OpenAPI contract expansion.
- No SDK generation behavior change.
- No playback, storage, NFO, metadata provider breadth, database schema, or
  server runtime behavior changes.
- No server call-site import cleanup unless needed for compilation.

## Architecture Direction

The module split is a locality refactor, not a contract redesign. The deeper
contract boundary remains:

- permissive `nako-client-protocol`: stable Public Client wire types and route
  inventory;
- AGPL `nako-api`: server model adapters, admin/internal DTOs, OpenAPI and SDK
  generation;
- AGPL `nako-server`: route wiring, app orchestration, and behavior.

Follow-on work may update server call sites to import from specific modules
once the compatibility facade is proven.

## Completion

M46 completed as a behavior-preserving locality refactor:

- `public_client` owns Public Client protocol re-exports and model-to-DTO
  adapters.
- `admin` owns job, ingestion failure, and storage backend diagnostic DTOs.
- `metadata_diagnostics` owns metadata maintenance, provider attempt, provider
  runtime, raw response, and cleanup DTOs.
- `extension` owns webhook, automation, and addon DTOs.
- `lib.rs` is a thin compatibility facade that keeps existing root-level
  imports working.
