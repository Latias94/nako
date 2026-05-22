# Addon Architecture Deepening — Evidence And Gates

Status: Completed
Last updated: 2026-05-21

## Gate Policy

Use narrow gates first, then broaden only as risk requires.

Preferred commands:

```text
cargo fmt --all -- --check
cargo check -p nako-addon-protocol --tests
cargo check -p nako-api --tests
cargo check -p nako-core --tests
cargo check -p nako-db --tests
cargo check -p nako-server --tests
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-db addon --no-fail-fast
cargo nextest run -p nako-server addons --no-fail-fast
cargo nextest run -p nako-server addon_side_effect --no-fail-fast
git diff --check
```

Final closeout should run:

```text
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

When `NAKO_TEST_POSTGRES_URL` is available, run touched PostgreSQL opt-in
contract families, especially Addon repository and side-effect contracts.

## Evidence Ledger

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | AAD-000 workstream open | Created `docs/workstreams/addon-architecture-deepening/` after Addon architecture review. | Pass |
| 2026-05-21 | AAD-010 authority freeze | Updated ADR statuses/notes, workstream index, goal map, and TODO ledger. Ran `git diff --check`. | Pass |
| 2026-05-21 | AAD-020 side-effect runtime depth | Added `AddonSideEffectRuntime`; made `intake.rs` a thin submit adapter; shared Addon principal resolution/authorization helpers; changed metadata/artwork/library-file Adapters to return `AddonSideEffectApplyCommand`; centralized validation rejection, apply outcome persistence, and apply failure taxonomy in the apply router. Ran `cargo check -p nako-core -p nako-server --tests`; `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-030 request fingerprint idempotency | Added deterministic `AddonSideEffectRequestFingerprint`, persisted Addon Side Effect request fingerprints, and added runtime conflict detection for same-key different-request reuse. Ran `cargo check -p nako-core -p nako-db -p nako-server --tests`; `cargo nextest run -p nako-db addon --no-fail-fast`; `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. `NAKO_TEST_POSTGRES_URL` was not set, so PostgreSQL opt-in contracts were skipped. AAD-090 later folded the fingerprint into clean base schemas and removed the temporary compatibility migration/fallback. | Pass |
| 2026-05-21 | AAD-040 Protected Write payload contracts | Added `AddonMetadataPatch`, `AddonArtworkWritePayload`, `AddonLibraryFileWritePayload`, and supporting enums to `nako-addon-protocol`; updated server Adapters to parse protocol DTOs; added protocol/reference Addon tests and author docs. Ran `cargo check -p nako-addon-protocol -p nako-api -p nako-reference-addon -p nako-server --tests`; `cargo nextest run -p nako-addon-protocol --no-fail-fast`; `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-050 Addon Manifest depth | Added first-class manifest declarations for Addon Entry Points, Addon Hosted Pages, Addon Configuration Schema, Secret Reference fields, Addon Event Subscriptions, and Addon Tasks; deepened validation; updated reference Addon and author/API docs. Ran `cargo check -p nako-addon-protocol -p nako-reference-addon -p nako-server --tests`; `cargo nextest run -p nako-addon-protocol --no-fail-fast`; `cargo nextest run -p nako-server register_addon --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-060 Library File Write runtime | Added `AddonLibraryFileWriteRuntime` with command/target/outcome seams for the NFO Export file role; centralized target resolution, file-role dispatch, writable-backend validation, permit acquisition, first-party NFO/VFS delegation, and redacted report shaping. Ran `cargo check -p nako-core -p nako-nfo -p nako-vfs -p nako-server --tests`; `cargo nextest run -p nako-server library_file_write --no-fail-fast`; `cargo nextest run -p nako-nfo nfo_service_export --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-070 Admin Addon API and DTO shielding | Migrated Addon registration/list/detail to `/admin/v1/addons`, removed the old root `/addons` management routes and legacy persistence-record response DTOs, added Admin summary/detail DTOs with parsed manifest snapshots, updated route tests to assert root `/addons` returns `404` and `manifest_json` is not exposed, and refreshed Admin API docs/matrix. Ran `cargo check -p nako-api -p nako-server --tests`; `cargo nextest run -p nako-server addons --no-fail-fast`; `cargo nextest run -p nako-api --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-080 Protocol/client crate boundary | Split HTTP caller helpers from `nako-addon-protocol` into new permissive `nako-addon-client`; removed `reqwest` and `async-trait` from the protocol crate; moved `AddonTransport`, `ReqwestAddonTransport`, `AddonClientError`, and `call_addon_resource` plus caller tests to the client crate; updated server reference-addon tests and docs/ADR notes. Ran `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server --tests`; `cargo nextest run -p nako-addon-protocol -p nako-addon-client --no-fail-fast`; `cargo nextest run -p nako-server reference_addon_registers_queries_and_handles_resource_call --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAD-090 Addon persistence parity | Verified the touched Addon persistence slice after the no-compatibility cleanup: SQLite migration `0022_addon_side_effects.sql` and PostgreSQL proof schema `0001_contract_jobs.sql` define `addon_side_effects.request_fingerprint` as `NOT NULL`; the temporary `0031` migration and nullable row-decoding fallback were removed; SQLite tests assert the migrated column is non-null and that stored side effects expose the deterministic fingerprint. Ran `cargo check -p nako-db --tests`; `cargo nextest run -p nako-db addon --no-fail-fast`; `rg -n "0031\|addon_side_effect_request_fingerprint\|request_fingerprint_idx\|Option<String>.*request_fingerprint\|request_fingerprint.*Option\|ADD COLUMN request_fingerprint" crates/nako-db`; `cargo fmt --all -- --check`; `git diff --check`. `NAKO_TEST_POSTGRES_URL` was not set, so PostgreSQL opt-in contracts were skipped. | Pass |
| 2026-05-21 | AAD-100 closeout | Reviewed the lane against Design target state, TODO completion, ADR constraints, redaction gates, and no-compatibility direction. Ran `cargo fmt --all -- --check`; `cargo check --workspace --tests`; `cargo nextest run -p nako-addon-protocol -p nako-addon-client --no-fail-fast` (12 passed); `cargo nextest run -p nako-db addon --no-fail-fast` (11 passed); `cargo nextest run -p nako-server addons --no-fail-fast` (32 passed); `cargo nextest run -p nako-server addon_side_effect --no-fail-fast` (10 passed); `cargo nextest run -p nako-server library_file_write --no-fail-fast` (3 passed); `cargo nextest run -p nako-api --no-fail-fast` (42 passed); `cargo nextest run --workspace --no-fail-fast` (521 passed, 25 skipped); `git diff --check` (passed with Git CRLF normalization warnings only). `NAKO_TEST_POSTGRES_URL` was not set, so PostgreSQL opt-in contracts were skipped. | Pass |

## Redaction Gates

Any changed Addon route, DTO, report, or diagnostic must not expose:

- raw Addon Token values or token hashes;
- administrator bearer tokens;
- provider credentials or Secret Reference resolved values;
- raw Addon Side Effect payload/provenance in public/admin summaries;
- Source Locators;
- storage URIs;
- local filesystem paths;
- backup URIs;
- raw remote handles;
- raw source URLs as public client artwork;
- cache URIs or content hashes unless a route explicitly owns safe operator
  diagnostics.

## Review Gates

- Use `review-workstream` before accepting large implementation slices.
- Use `verify-rust-workstream` before marking tasks or this lane complete.
- Update `HANDOFF.md` after each completed task.
- Add a `JOURNAL/YYYY-MM-DD-<task>.md` note for non-trivial implementation
  slices.
