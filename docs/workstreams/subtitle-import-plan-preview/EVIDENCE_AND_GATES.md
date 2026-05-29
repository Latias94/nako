# Subtitle Import Plan Preview Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Workstream docs | `git diff --check` | Pass | SIPP-010. |
| API DTO tests | `cargo nextest run -p nako-api subtitle_import_plan --no-fail-fast` | Pass | 1 passed. |
| API contract tests | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Pass | 5 passed. |
| Server HTTP tests | `cargo nextest run -p nako-server addon_subtitle_import_plan --no-fail-fast` | Pass | 3 passed. |
| Package check | `cargo check -p nako-api -p nako-server --tests` | Pass | Final. |
| Rust format | `cargo fmt --all -- --check` | Pass | Final. |
| Diff hygiene | `git diff --cached --check` | Pass | Final pre-commit gate. |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | SIPP-010 | Workstream opened from subtitle selected-reference follow-on. | Pass |
| 2026-05-28 | SIPP-020 | `cargo nextest run -p nako-api subtitle_import_plan --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`. | Pass |
| 2026-05-28 | SIPP-030 | `cargo nextest run -p nako-server addon_subtitle_import_plan --no-fail-fast`. | Pass |
| 2026-05-28 | SIPP-040 | `cargo check -p nako-api -p nako-server --tests`; `cargo fmt --all -- --check`; `git diff --cached --check`. | Pass |

## Review Notes

- Import plan preview must not write files or queue write jobs.
- The selected subtitle provider candidate remains server-owned; clients submit
  only host IDs and safe policy choices.
