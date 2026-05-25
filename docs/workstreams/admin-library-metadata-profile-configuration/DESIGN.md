# Admin Library Metadata Profile Configuration

Status: Completed
Last updated: 2026-05-25

## Why This Lane Exists

Nako can already execute scan-time metadata acquisition from a Media Library's
`MetadataProfile`, but operators still need a product API for changing that
profile after startup. TOML-only overrides are useful for bootstrapping, but
they do not form a runtime configuration loop for Admin clients.

## Relevant Authority

- Glossary: `CONTEXT.md`
- Related workstreams:
  - `docs/workstreams/library-metadata-scan-policy`
  - `docs/workstreams/scan-addon-bulk-metadata-scrape`
  - `docs/workstreams/metadata-acquisition-pipeline`
  - `docs/workstreams/addon-protected-writes`
- Existing code:
  - `crates/nako-core/src/media/profile.rs`
  - `crates/nako-core/src/media/library.rs`
  - `crates/nako-server/src/app/library.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-api/src/admin.rs`

## Problem

Scan-time acquisition policy is implemented but not operator-editable through
Admin API. A user can configure `metadata.library_profiles` before startup, and
Public Client library DTOs expose the effective profile, but there is no Admin
mutation path that persists changes to a library's `options_json`.

This leaves the product loop incomplete:

- local NFO import can be enabled/disabled only by config edits or direct DB
  changes;
- Addon Bulk Metadata Scrape and explicit Addon metadata writeback require
  code/config setup rather than an Admin workflow;
- tests prove scan behavior from preloaded profiles, not runtime operator
  updates.

## Target State

An administrator can read and replace a Media Library's effective
`MetadataProfile` through Admin API. The update is persisted through the
existing `LibraryRepository::upsert_library` path and the next scan derives its
`MetadataScanAcquisitionPlan` from the updated profile.

## In Scope

- Admin API response/request DTOs for library metadata profile read/update.
- Admin HTTP routes under `/admin/v1/libraries/{library_id}/metadata-profile`.
- Application service methods that load a library, replace
  `library.options.metadata_profile`, and upsert the library.
- Focused HTTP tests for persistence and scan-policy effect.
- Admin TypeScript contract route/type updates.
- Workstream evidence updates.

## Out Of Scope

- Admin Web UI screens or controls.
- Partial patch/merge semantics for every profile field.
- Schema migrations or a new profile table.
- New provider refresh, embedded metadata, sidecar reader, or artwork runtime
  behavior.
- Addon capability negotiation, grant UI, or health-state UX.
- Restart-proofing against configured-library reconciliation overwrites beyond
  documenting the current behavior.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `LibraryOptions` is already persisted as JSON for SQLite and PostgreSQL. | High | `crates/nako-db/src/sqlite/library.rs`, `crates/nako-db/src/postgres/core_catalog.rs` | A migration or repository API expansion would be needed. |
| Scans fetch the library from the repository at execution time. | High | Existing scan tests mutate stored libraries and scan behavior follows. | The update would need runtime cache invalidation. |
| Full-profile replace is a safer first Admin contract than many field-specific toggles. | Medium | `MetadataProfile` already serializes as one policy object. | We may split a follow-on for typed partial update commands. |
| Configured libraries may overwrite DB library options on restart. | High | startup reconciliation tests cover configured desired state. | Admin-updated profiles may need config persistence or operator docs later. |

## Architecture Direction

Keep ownership simple for the first product slice:

- `nako-core` remains the owner of the `MetadataProfile` domain model and
  `MetadataScanAcquisitionPlan` derivation.
- `nako-db` keeps storing full `LibraryOptions` through existing library
  repository upserts; no new schema table is introduced.
- `nako-server::app::LibraryAppService` owns the workflow of reading a library,
  replacing its profile, and persisting it.
- `nako-server::http::admin` owns the Admin route boundary.
- `nako-api::admin` owns Admin DTOs and the generated Admin TypeScript
  contract.

Full-profile replacement keeps the first API deterministic: the request body is
the complete desired profile, and the response returns the persisted effective
profile. Later UI work can layer safer form-specific patch commands on top.

## Closeout Condition

This lane can close when:

- Admin read/update routes are implemented and tested;
- updating a profile through Admin API persists through repository reload;
- a scan after update observes the new scan acquisition plan;
- Admin TypeScript contract is regenerated and verified;
- evidence gates are recorded; and
- UI, config-file persistence, and richer field-level validation are either
  implemented or explicitly split.

Closeout result: met on 2026-05-25. Follow-ons are split for restart-proof
configuration authority, Admin Web controls/design, field-specific update
commands, and capability-aware Addon scrape/writeback UX.
