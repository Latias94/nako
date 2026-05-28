# Addon Resource Link Check Contract - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-addon-protocol resource_link_check --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-addon-protocol resource_link_check --no-fail-fast
cargo nextest run -p nako-addon-client resource_link_check --no-fail-fast
```

### Contract Closeout Gate

```bash
cargo nextest run -p nako-addon-protocol -p nako-addon-client resource_link_check --no-fail-fast
```

### Static Gates

```bash
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client --tests
git diff --check
```

## Evidence Anchors

- `docs/workstreams/addon-resource-link-check-contract/DESIGN.md`
- `docs/workstreams/addon-resource-link-check-contract/TODO.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`

## Run Log

2026-05-28:

- `cargo nextest run -p nako-addon-protocol resource_link_check --no-fail-fast`
  passed: 3 tests run, 3 passed. This proves stable wire names, dedicated
  scope enforcement, typed DTO round-trip, envelope validation, and debug
  redaction for request links.
- `cargo nextest run -p nako-addon-client resource_link_check --no-fail-fast`
  passed: 6 tests run, 6 passed. This proves the typed helper calls the declared
  path, uses the `resource_link_check` envelope, validates granted scope,
  validates manifest-declared scope, rejects wrong request schema before HTTP,
  and rejects wrong/invalid response payloads after HTTP.
- `cargo nextest run -p nako-addon-protocol -p nako-addon-client resource_link_check --no-fail-fast`
  passed: 9 tests run, 9 passed.
- `cargo fmt --all -- --check` passed.
- `cargo check -p nako-addon-protocol -p nako-addon-client --tests` passed.
- `git diff --check` passed with Windows line-ending warnings only.

## Review Notes

No blocking findings remain.

The lane intentionally stops at protocol and client helper. Server/product
routes, Admin UI, actual checker providers, downloader execution, cloud-drive
transfer, and password/code persistence remain follow-ons.
