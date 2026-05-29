# Subtitle Search Candidate Selection Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Workstream docs | Path-scoped `git diff --check` | Pass | SSCS-010. |
| Addon client subtitle tests | `cargo nextest run -p nako-addon-client subtitle --no-fail-fast` | Pass | 6 passed. |
| API subtitle tests | `cargo nextest run -p nako-api subtitle --no-fail-fast` | Pass | 1 passed. |
| API contract tests | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Pass | 5 passed. |
| Server subtitle HTTP tests | `cargo nextest run -p nako-server addon_subtitle --no-fail-fast` | Pass | 3 passed. |
| Package check | `cargo check -p nako-addon-client -p nako-api -p nako-server --tests` | Pass | Final. |
| Rust format | `cargo fmt --all -- --check` | Pass | Final. |
| Diff hygiene | Path-scoped `git diff --check` | Pass | Avoided unrelated web changes. |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | SSCS-010 | Workstream opened from `subtitle-complete-chain/FOLLOW_ONS.md`. | Pass |
| 2026-05-28 | SSCS-020 | `cargo nextest run -p nako-addon-client subtitle --no-fail-fast`; `cargo check -p nako-addon-client --tests`. | Pass |
| 2026-05-28 | SSCS-030 | `cargo nextest run -p nako-api subtitle --no-fail-fast`; `cargo nextest run -p nako-server addon_subtitle --no-fail-fast`. | Pass |
| 2026-05-28 | SSCS-040 | `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo check -p nako-addon-client -p nako-api -p nako-server --tests`; `cargo fmt --all -- --check`; path-scoped `git diff --check`. | Pass |

## Review Notes

- Do not implement subtitle file writes.
- Do not expose raw provider delivery payloads in Admin responses.
- Keep browser/client selection requests empty or host-owned; never accept raw
  subtitle text or URLs back from clients.
