# Addon Resource Search Product Flow - Evidence And Gates

Status: Active
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
cargo nextest run -p nako-server addon_resource_search_product acquisition_intake --no-fail-fast
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

## Review Gate

Run `review-workstream` before accepting completed implementation slices.
Run `verify-rust-workstream` before marking the lane complete.
