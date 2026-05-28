# Addon Resource Link Check Product Flow - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast
```

## Gate Set

```bash
cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
git diff --check
```

## Evidence Anchors

- `crates/nako-api/src/extension.rs`
- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`

## Run Log

| Date | Evidence | Result |
| --- | --- | --- |
| 2026-05-28 | `cargo nextest run -p nako-server addon_resource_link_check --no-fail-fast` | Pass. 1 test passed, proving opaque-id link-check flow, raw payload rejection, safe response redaction, and addon request context redaction. |
| 2026-05-28 | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Initial run failed because generated Admin TypeScript contracts were stale; after regeneration, pass. 5 tests passed. |
| 2026-05-28 | `cargo nextest run -p nako-api admin_resource_link_check_response_uses_safe_facts_only --no-fail-fast` | Pass. 1 API DTO redaction test passed. |
| 2026-05-28 | `cargo fmt --all -- --check` | Pass. |
| 2026-05-28 | `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests` | Pass. |
| 2026-05-28 | `git diff --check` | Pass with Git LF-to-CRLF working-copy warnings only. |

## Residual Risks

- No Admin UI was added in this lane.
- No concrete checker provider, downloader, cloud-drive transfer, or durable
  password/code persistence was added.
- Link-check results are product-safe summaries only; any future raw diagnostic
  view needs its own redaction and permission review.
