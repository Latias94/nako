# Addon Resource Search Protocol - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast
```

Before ARSP-020 lands this is expected to run zero tests or fail because the
contract does not exist yet. After ARSP-020, it proves protocol vocabulary,
serde shape, and manifest validation behavior.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast
cargo nextest run -p nako-addon-client resource_search --no-fail-fast
cargo nextest run -p nako-server addon_resource_search --no-fail-fast
cargo nextest run -p nako-server acquisition_intake addon_resource_search --no-fail-fast
```

### Package Gates

```bash
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-server addon --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests
git diff --check
```

Run broader `cargo nextest run --workspace --no-fail-fast` only if the lane
touches shared storage or cross-cutting server behavior beyond addon/resource
contracts.

### Review Gate

Run `review-workstream` before accepting completed implementation slices.
Run `verify-rust-workstream` before marking the lane complete.

## Evidence Anchors

- `docs/workstreams/addon-resource-search-protocol/DESIGN.md`
- `docs/workstreams/addon-resource-search-protocol/TODO.md`
- `docs/workstreams/addon-resource-search-protocol/CLOSEOUT.md`
- `docs/workstreams/addon-resource-search-protocol/MILESTONES.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-core/src/acquisition_intake.rs`
- `crates/nako-core/src/managed_import.rs`
- `crates/nako-server/src/app/acquisition_intake.rs`
- `crates/nako-db/src/contract_tests.rs`
- `crates/nako-server/src/app/addons.rs`
- `F:/SourceCodes/Rust/nako-official-addons/docs/workstreams/official-resource-search-architecture-hardening/PROTOCOL_PROPOSAL.md`

## Evidence Log

### 2026-05-28 - ARSP-010

- Opened workstream from the hardened official resource-search addon proposal.
- Froze read-only search scope, non-goals, and acquisition handoff split.
- No code changes yet.

### 2026-05-28 - ARSP-020

- Added `AddonResource::ResourceSearch`, `AddonScope::AcquisitionSearchRead`,
  resource-search request/response DTOs, link taxonomy, provider execution
  status, and provider finality contracts.
- Hardened `Debug` output for resource links so raw URLs and extraction
  passwords are not emitted through protocol debug formatting.
- Passed `cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast`
  with 3 tests.
- Passed `cargo nextest run -p nako-addon-protocol --no-fail-fast` with 16
  tests.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client --tests`.
- Passed `git diff --check`.

### 2026-05-28 - ARSP-030

- Added typed `call_addon_resource_search` and
  `call_addon_resource_search_with_outcome` helpers in `nako-addon-client`.
- Kept transport, retry, timeout, manifest, envelope, auth, and scope behavior
  on the existing generic resource-call path.
- Enforced `acquisition_search_read` on the resource-search manifest
  declaration and granted scope set before HTTP.
- Enforced request/response payload schema constants and typed response payload
  parsing.
- Passed `cargo nextest run -p nako-addon-client resource_search --no-fail-fast`
  with 6 tests.
- Passed `cargo nextest run -p nako-addon-client --no-fail-fast` with 22 tests.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client --tests`.
- Passed `git diff --check`.

### 2026-05-28 - ARSP-040

- Added admin/API DTOs for resource-search diagnostics with host-owned request
  limit, typed intent, source filters, link-type filters, and safe response
  summaries.
- Added `AddonAppService::diagnose_addon_resource_search` and
  `/admin/v1/addons/{addon_id}/diagnostics/resource-search`.
- Kept diagnostics redaction-safe: responses expose result/link/provider counts
  and provider execution status only, not raw result titles, URLs, passwords,
  request context, or provider messages.
- Refreshed generated Admin API TypeScript contracts for `apps/admin-web` and
  `web`.
- Passed `cargo nextest run -p nako-server addon_resource_search --no-fail-fast`
  with 2 tests.
- Passed `cargo nextest run -p nako-api admin_contract --no-fail-fast` with 5
  tests.
- Passed `cargo check -p nako-api -p nako-server --tests`.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-server --tests`.
- Passed `git diff --check`.

### 2026-05-28 - ARSP-050

- Added `resource_search_selection` as an explicit
  `AcquisitionIntakeSourceKind` and `ManagedImportSourceKind`.
- Added host-owned `AcquisitionIntakeAppService::record_resource_search_selection`
  for explicit selected-result/link conversion.
- Kept selected resource-search conversion out of HTTP routing and out of addon
  runtime candidate-write scopes.
- Recorded selected links as ready intake candidates with stable hashed source
  keys, redacted diagnostics, and no downloader/link-check/cloud-drive effects.
- Preserved later accept flow into managed import as
  `ManagedImportSourceKind::ResourceSearchSelection` without media-source
  creation or promotion apply.
- Passed `cargo nextest run -p nako-server acquisition_intake --no-fail-fast`
  with 8 tests.
- Passed `cargo nextest run -p nako-server acquisition_intake addon_resource_search --no-fail-fast`
  with 10 tests.
- Passed `cargo nextest run -p nako-db sqlite_managed_import_contract_round_trips_artifacts_and_state sqlite_acquisition_intake_contract_round_trips_candidates_and_state --no-fail-fast`
  with 2 tests.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`.
- Passed `git diff --check`.
- Passed `python -m json.tool docs\workstreams\addon-resource-search-protocol\WORKSTREAM.json`.

### 2026-05-28 - ARSP-060

- Reviewed the completed workstream against scope, milestones, and evidence.
- Closed the lane and split official addon migration, admin/UI selection,
  link-checking, downloader execution, cloud-drive save, and password handling
  to follow-ons in `CLOSEOUT.md`.
- Passed `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server -p nako-db resource_search admin_contract acquisition_intake addon_resource_search sqlite_managed_import_contract_round_trips_artifacts_and_state sqlite_acquisition_intake_contract_round_trips_candidates_and_state --no-fail-fast`
  with 27 tests.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`.
- Passed `python -m json.tool docs\workstreams\addon-resource-search-protocol\WORKSTREAM.json`.
- Passed `git diff --check`.

## Notes

Search is not acquisition. The base protocol must not imply candidate writes,
link checks, downloader execution, cloud-drive save, stream URL read, or
playback authority.
