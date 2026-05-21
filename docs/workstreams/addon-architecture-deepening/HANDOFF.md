# Addon Architecture Deepening — Handoff

Status: Completed
Last updated: 2026-05-21

## Current State

The workstream has been opened after the 2026-05-21 Addon architecture review.
AAD-010 is complete: shipped Addon ADRs are marked Accepted with notes,
`docs/workstreams/README.md` links this lane, and `docs/GOALS.md` records
Addon Architecture Deepening as the current active goal.

AAD-020 is complete. Addon Side Effect submission now goes through
`AddonSideEffectRuntime`, while `intake.rs` is only a thin App Service adapter.
The runtime owns replay lookup, authority, target validation, journaling, and
validation rejection. `AddonSideEffectApplyRouter` now owns apply dispatch,
apply outcome persistence, metadata write commit execution, and apply failure
taxonomy. Metadata, Artwork, and Library File Write Adapters return the common
`AddonSideEffectApplyCommand` shape instead of directly deciding journal
outcomes.

AAD-030 is complete. Addon Side Effects now carry a deterministic
`AddonSideEffectRequestFingerprint` derived from permission, library, target,
provenance JSON, and payload JSON. The runtime now returns normal idempotent
replay only when the same Addon plus same idempotency key has the same
fingerprint; different requests return `409 conflict` with redacted public
errors. AAD-090 later folded the fingerprint into the clean SQLite/PostgreSQL
base schemas and removed the temporary compatibility migration/fallback because
Taru has no deployed users.

AAD-040 is complete. Shipped Protected Write payload contracts now live in
`taru-addon-protocol` as Addon-facing DTOs:
`AddonMetadataPatch`, `AddonArtworkWritePayload`, and
`AddonLibraryFileWritePayload` with supporting enums. Server Adapters parse
those DTOs but retain Taru-owned validation/application/storage/report logic.
`taru-reference-addon` exposes small demo builders for the new payload
contracts, and the API/author docs now point Addon authors at these explicit
Interfaces.

AAD-050 is complete. `taru-addon-protocol` now has first-class Addon Manifest
declarations for Addon Entry Points, Addon Hosted Pages, Addon Configuration
Schema, Secret Reference fields, Addon Event Subscriptions, and Addon Tasks.
The additions are compatible and keep the current Addon Protocol Version.
Manifest validation now rejects duplicate declaration IDs, relative declaration
paths, undeclared declaration scopes, entry points that reference missing
hosted pages, non-object configuration schemas, and invalid Addon Task
timeout/retry bounds. The reference Addon manifest now exercises entry point,
hosted page, and configuration schema declarations, and author/API docs explain
the new manifest contract.

AAD-060 is complete. `AddonLibraryFileWriteAdapter` now delegates to
`AddonLibraryFileWriteRuntime`, which owns the file-write command, target, and
outcome seams for the shipped MediaSource-targeted NFO Export path. The runtime
normalizes the typed payload into a Taru command, dispatches by file role,
resolves the MediaSource target, validates the library, acquires the file-write
permit, checks writable storage, delegates to first-party NFO/VFS export, and
returns a redacted apply report with safe IDs, write-mode, backup-policy, and
aggregate counters only. Subtitle, arbitrary sidecar asset, broader NFO, and
queued Library File Write execution remain deferred.

AAD-070 is complete. Addon registration/list/detail management has been
migrated to `/admin/v1/addons`. The old root `/addons` management routes are
intentionally not mounted. `taru-api` now exposes Admin Addon summary/detail
response DTOs instead of the legacy persistence-record responses;
create/detail responses include a summary plus parsed manifest, while list
responses include summaries only. Route tests cover admin
registration/list/detail, root `/addons` `404`, and absence of `manifest_json`
in Admin responses. HTTP API and Admin Web Console matrix docs have been
updated.

AAD-080 is complete. The deletion test found real Interface cost in keeping
HTTP caller helpers inside `taru-addon-protocol`: Addon wire-contract
consumers inherited `reqwest` and `async-trait` even when they only needed DTOs
and validation. The HTTP caller helper has been split into a new permissive
`taru-addon-client` crate. `taru-addon-protocol` now owns manifests, envelopes,
scopes, Protected Write payload DTOs, and validation only; `taru-addon-client`
owns `AddonTransport`, `ReqwestAddonTransport`, `AddonClientError`, and
`call_addon_resource`.

AAD-090 is complete. Addon persistence parity for the touched fingerprinted
side-effect slice now uses a clean schema boundary: SQLite migration
`0022_addon_side_effects.sql` and PostgreSQL proof schema
`migrations/postgres/0001_contract_jobs.sql` define
`addon_side_effects.request_fingerprint` as `NOT NULL`; the temporary `0031`
fingerprint migration and nullable row-decoding fallback are gone. SQLite
Addon tests assert the migrated column is non-null and that persisted side
effects round-trip the deterministic request fingerprint. PostgreSQL opt-in
contracts were skipped because `TARU_TEST_POSTGRES_URL` was not set.

## Closeout

AAD-100 is complete. The lane closed on 2026-05-21 after review and fresh
verification. No remaining tail was hidden inside the lane: Addon Manager, full
Addon Task runtime, Addon Event Subscription delivery, subtitle breadth, Native
Plugin ABI, and Jellyfin Plugin Compatibility remain explicit non-goals rather
than vague follow-up buckets.

## Follow-ons

- No new follow-on workstream is required for this closeout.
- If Addon Manager, hosted Addon UI execution, Addon Task runtime, Event
  Subscription delivery, or subtitle/sidecar file breadth becomes product
  priority, open a new named workstream with its own design, gates, and
  authority updates.

## Important Constraints

- Use Addon terminology from `CONTEXT.md`; do not drift back to plugin-centric
  language in docs or code discussions.
- Do not introduce Native Plugin or Jellyfin Plugin Compatibility.
- Keep `taru-addon-protocol` permissive and free of AGPL server internals.
- Addon runtime routes must use Addon Token authority, not administrator bearer
  token authority.
- Addon Side Effect responses must stay redacted.

## Validation Notes

- AAD-010 validation: `git diff --check` passed.
- AAD-020 validation:
  - `cargo check -p taru-core -p taru-server --tests`
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-030 validation:
  - `cargo check -p taru-core -p taru-db -p taru-server --tests`
  - `cargo nextest run -p taru-db addon --no-fail-fast`
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - PostgreSQL opt-in contracts skipped: `TARU_TEST_POSTGRES_URL` was not set.
- AAD-040 validation:
  - `cargo check -p taru-addon-protocol -p taru-api -p taru-reference-addon -p taru-server --tests`
  - `cargo nextest run -p taru-addon-protocol --no-fail-fast`
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-050 validation:
  - `cargo check -p taru-addon-protocol -p taru-reference-addon -p taru-server --tests`
  - `cargo nextest run -p taru-addon-protocol --no-fail-fast`
  - `cargo nextest run -p taru-server register_addon --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-060 validation:
  - `cargo check -p taru-core -p taru-nfo -p taru-vfs -p taru-server --tests`
  - `cargo nextest run -p taru-server library_file_write --no-fail-fast`
  - `cargo nextest run -p taru-nfo nfo_service_export --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-070 validation:
  - `cargo check -p taru-api -p taru-server --tests`
  - `cargo nextest run -p taru-server addons --no-fail-fast`
  - `cargo nextest run -p taru-api --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-080 validation:
  - `cargo check -p taru-addon-protocol -p taru-addon-client -p taru-reference-addon -p taru-server --tests`
  - `cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast`
  - `cargo nextest run -p taru-server reference_addon_registers_queries_and_handles_resource_call --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- AAD-090 validation:
  - `cargo check -p taru-db --tests`
  - `cargo nextest run -p taru-db addon --no-fail-fast`
  - searched `crates/taru-db` for obsolete `0031`, Addon fingerprint migration,
    fingerprint index, nullable fingerprint fallback, and add-column patterns
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - PostgreSQL opt-in contracts skipped: `TARU_TEST_POSTGRES_URL` was not set.
- AAD-100 validation:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --tests`
  - `cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast`
  - `cargo nextest run -p taru-db addon --no-fail-fast`
  - `cargo nextest run -p taru-server addons --no-fail-fast`
  - `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
  - `cargo nextest run -p taru-server library_file_write --no-fail-fast`
  - `cargo nextest run -p taru-api --no-fail-fast`
  - `cargo nextest run --workspace --no-fail-fast`
  - `git diff --check`
  - PostgreSQL opt-in contracts skipped: `TARU_TEST_POSTGRES_URL` was not set.
