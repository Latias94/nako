# Subtitle Complete Chain Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Workstream docs | Manual review; `git diff --check` | Pass | SCC-010 opened the lane. |
| Protocol subtitle tests | `cargo nextest run -p nako-addon-protocol subtitle --no-fail-fast` | Pending | SCC-020 |
| Protocol/catalog/server check | `cargo check -p nako-addon-protocol -p nako-official-addon-catalog -p nako-server --tests` | Pending | SCC-020 |
| Official subtitle provider tests | `cargo nextest run -p nako-subtitle-provider --no-fail-fast` | Pending | SCC-030 |
| Official subtitle provider check | `cargo check -p nako-subtitle-provider --tests` | Pending | SCC-030 |
| Rust format | `cargo fmt --all -- --check` | Pending | Final gate |
| Diff hygiene | `git diff --check` | Pending | Final gate |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | SCC-010 | ADR and workstream docs opened for host-owned subtitle import chain. | Pass |

## Review Notes

- This lane must not implement subtitle sidecar writes.
- Protocol types must not contain addon-provided filesystem paths, Source
  Locators, remote storage handles, or write policies.
- Library File Write owns future subtitle sidecar persistence.
