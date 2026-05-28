# Addon Resource Search Product Flow - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast
```

Before RSPF-030 lands this may run zero tests. After RSPF-030/RSPF-040, it
should prove safe product search and explicit selection behavior.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast
cargo nextest run -p nako-server acquisition_intake --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests
git diff --check
```

## Evidence Anchors

- `docs/workstreams/addon-resource-search-product-flow/DESIGN.md`
- `docs/workstreams/addon-resource-search-product-flow/TODO.md`
- `docs/workstreams/addon-resource-search-product-flow/EVIDENCE_AND_GATES.md`
- `docs/workstreams/addon-resource-search-protocol/CLOSEOUT.md`
- `crates/nako-api/src/extension.rs`
- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/app/acquisition_intake.rs`

## Evidence Log

### 2026-05-28 - RSPF-010

- Opened Nako-only product-flow lane after closing
  `addon-resource-search-protocol`.
- Froze non-goals: no official addon migration, UI, downloader, link-check,
  cloud-drive save, or password persistence in this lane.

### 2026-05-28 - RSPF-020

- Added Admin API DTOs for product resource search, safe result summaries,
  redacted link summaries, and explicit selection responses.
- Added route constants for product search and selected-link intake candidate
  creation.
- Refreshed generated Admin TypeScript contracts for `apps/admin-web` and
  `web`.
- Added API serialization coverage proving product responses use opaque
  selection refs and do not include raw URLs, normalized URLs, request context,
  provider exceptions, or secret codes.
- Passed `cargo nextest run -p nako-api admin_contract --no-fail-fast` with 5
  tests.
- Passed `cargo nextest run -p nako-api admin_resource_search_product_response --no-fail-fast`
  with 1 test.

### 2026-05-28 - RSPF-030/RSPF-040/RSPF-050

- Added a host-owned transient resource-search session store in
  `nako-server::app::addons`.
- Added product search execution that uses the typed addon client, clamps host
  limits, keeps diagnostic behavior separate, and returns display-safe result
  cards with opaque `search_id`/`selection_id` values.
- Added explicit selected-link intake handoff that reads raw links only from the
  host session, records `resource_search_selection`, and reports true
  idempotent replay from `acquisition_intake`.
- Added Admin HTTP routes for product search and selected-link intake candidate
  creation.
- Added HTTP tests proving product search does not expose raw URLs, passwords,
  context payloads, provider messages, or image URLs, and proving selection
  writes/replays intake candidates from opaque IDs.
- Passed `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`
  with 2 tests.
- Passed `cargo nextest run -p nako-server acquisition_intake --no-fail-fast`
  with 8 tests.
- Passed `cargo nextest run -p nako-api admin_contract --no-fail-fast` with 5
  tests.
- Passed `cargo fmt --all -- --check`.
- Passed `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`.
- Passed `git diff --check`; Git reported expected Windows line-ending
  warnings only.

### 2026-05-28 - RSPF-060

- Reviewed the lane against `DESIGN.md`, `TODO.md`, ADR boundaries, and the
  current diff.
- Review result: no blocking or important findings remain.
- Tightened the transient selection session after review so each selection
  stores only a result metadata/count snapshot plus the selected raw link,
  avoiding repeated storage of unrelated raw links.
- Re-ran `cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast`
  after the review fix with 2 tests passing.
- Re-ran `cargo nextest run -p nako-server acquisition_intake --no-fail-fast`
  with 8 tests passing.
- Re-ran `cargo nextest run -p nako-api admin_contract --no-fail-fast` with 5
  tests passing.
- Re-ran `cargo fmt --all -- --check`.
- Re-ran `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`.
- Re-ran `git diff --check`; Git reported expected Windows line-ending
  warnings only.

## Review Gate

Run `review-workstream` before accepting completed implementation slices.
Run `verify-rust-workstream` before marking the lane complete.
