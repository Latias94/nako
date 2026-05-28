# Subtitle Import Apply Evidence And Gates

| Gate | Command | Result |
| --- | --- | --- |
| API DTO tests | `cargo nextest run -p nako-api subtitle_import_apply --no-fail-fast`; `cargo nextest run -p nako-api subtitle_import --no-fail-fast` | Pass. 1 passed; broader subtitle import scope 2 passed. |
| Admin contract | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Pass. 5 passed. |
| Server HTTP tests | `cargo nextest run -p nako-server addon_subtitle_import_apply --no-fail-fast` | Pass. 4 passed. |
| Server regression scope | `cargo nextest run -p nako-server addon_subtitle_import --no-fail-fast` | Pass. 7 passed. |
| Rust check | `cargo check -p nako-api -p nako-server --tests` | Pass. |
| Format | `cargo fmt --all -- --check` | Pass. |
| Diff check | `git diff --check` | Pass. Only Git LF/CRLF warnings on touched and user-edited files. |
| Workstream JSON | `Get-Content docs\workstreams\subtitle-import-apply\WORKSTREAM.json \| ConvertFrom-Json \| Out-Null` | Pass. |

## Evidence Log

- 2026-05-28: SIA-010 opened the workstream and locked host-owned Library
  File Write semantics.
- 2026-05-28: SIA-020 added Admin subtitle import-apply DTOs, route contract,
  and generated TypeScript contract copies.
- 2026-05-28: SIA-030 implemented import apply with plan-key validation,
  inline/download-url content resolution, subtitle text validation,
  idempotent same-content handling, create-missing conflict, replace-existing
  backup, and redacted apply reports.
