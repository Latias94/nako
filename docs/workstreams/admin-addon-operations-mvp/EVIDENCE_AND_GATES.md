# Admin Addon Operations MVP — Evidence And Gates

Status: Completed
Last updated: 2026-05-21

## Preferred Gates

```text
cargo fmt --all -- --check
cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p taru-core -p taru-db -p taru-server --tests
cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast
cargo nextest run -p taru-db addon --no-fail-fast
cargo nextest run -p taru-server addons --no-fail-fast
git diff --check
```

When `TARU_TEST_POSTGRES_URL` is available:

```text
cargo nextest run -p taru-db addon --run-ignored ignored-only --no-fail-fast
```

Closeout should add workspace gates when practical:

```text
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
```

## Evidence Ledger

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | AAO-000 workstream open | Created `docs/workstreams/admin-addon-operations-mvp/` after release packaging and Addon architecture deepening completed. | Pass |
| 2026-05-21 | AAO-010 contract baseline | Froze the Admin Addon Operations MVP route contract and lifecycle policy. Chose terminal `unregistered` state over physical deletion, with active token revocation, accepted grant clearing, preserved audit history, and no `DELETE /admin/v1/addons/{addon_id}` route. Updated HTTP API planning notes and Admin Web Console matrix. Ran `git diff --check`. | Pass |
| 2026-05-21 | AAO-020 lifecycle mutation | Added explicit `PATCH /admin/v1/addons/{addon_id}/status` for `enabled` / `disabled`, repository status mutation for SQLite/PostgreSQL, stored-manifest enable validation, and runtime disabled rejection before token usage refresh. Ran `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAO-030 unregister semantics | Added terminal `unregistered` lifecycle status, `POST /admin/v1/addons/{addon_id}/unregister`, atomic registration status transition with active token revocation and grant clearing, active-manifest uniqueness for re-registration, and HTTP/DB coverage for audit preservation and runtime failure. Ran `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. PostgreSQL opt-in contract was skipped because `TARU_TEST_POSTGRES_URL` was not available. | Pass |
| 2026-05-21 | AAO-040 health checks | Added Addon Health Check protocol envelopes, `taru-addon-client::check_addon_health`, reference-addon `/health`, Admin health DTOs, and `POST /admin/v1/addons/{addon_id}/health-check`. The route returns safe reachability/status, latency, protocol/manifest facts, and safe error codes without sending admin bearer tokens, Addon Tokens, resolved Secret References, or resource payloads to the Addon Sidecar. Ran `cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAO-050 hosted surface read models | Added `GET /admin/v1/addons/{addon_id}/surfaces` with Admin DTOs for Entry Points, Hosted Pages, Configuration Schema metadata, Secret Reference field declarations, Addon Tasks, and Addon Event Subscriptions. Hosted Page URLs are derived from manifest base URL and declared paths without admin bearer tokens, Addon Tokens, launch secrets, or resolved Secret Reference values. Ran `cargo check -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAO-060 resource-call diagnostics | Added `POST /admin/v1/addons/{addon_id}/diagnostics/resource-call`, diagnostic DTOs, and Addon client outcome metadata for safe HTTP status / attempt reporting. Diagnostics classify success, missing resource, missing grant, authorization gap, unreachable transport, protocol mismatch, retryable/non-retryable HTTP failure, and unsafe response cases without echoing diagnostic payloads, response payloads, raw response bodies, token material, Source Locators, storage paths, provider secrets, or raw network errors. Ran `cargo check -p taru-addon-client -p taru-api -p taru-server --tests`; `cargo nextest run -p taru-addon-client --no-fail-fast`; `cargo nextest run -p taru-server addons --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-21 | AAO-070 closeout | Closed Admin Addon Operations MVP after confirming all target operations shipped or remained explicit non-goals. Ran `cargo fmt --all -- --check`; `cargo check -p taru-addon-protocol -p taru-addon-client -p taru-api -p taru-core -p taru-db -p taru-server --tests`; `cargo nextest run -p taru-addon-protocol -p taru-addon-client --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast` with 12 tests passed and 105 skipped; `cargo nextest run -p taru-server addons --no-fail-fast` with 39 tests passed and 156 skipped; `cargo check --workspace --tests`; `cargo nextest run --workspace --no-fail-fast` with 532 tests passed and 25 skipped; `git diff --check`. PostgreSQL opt-in contracts were skipped because `TARU_TEST_POSTGRES_URL` was not set. | Pass |

## Redaction Gates

Admin Addon operations must not expose:

- raw Addon Token values or token hashes;
- administrator bearer tokens;
- provider credentials or resolved Secret Reference values;
- raw diagnostic request/response bodies;
- raw Addon Side Effect payload/provenance in Admin summaries;
- Source Locators;
- storage URIs;
- local filesystem paths;
- backup URIs;
- raw remote handles;
- raw network errors that include credentials.
