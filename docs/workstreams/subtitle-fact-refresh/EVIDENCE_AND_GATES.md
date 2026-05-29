# Subtitle Fact Refresh Evidence And Gates

| Gate | Command | Result |
| --- | --- | --- |
| API stream DTO tests | `cargo nextest run -p nako-api media_stream --no-fail-fast` | Passed: 1 test |
| API subtitle apply contract test | `cargo nextest run -p nako-api subtitle_import_apply --no-fail-fast` | Passed: 1 test |
| Admin contract tests | `cargo nextest run -p nako-api admin_contract --no-fail-fast` | Passed: 5 tests |
| Server subtitle import tests | `cargo nextest run -p nako-server addon_subtitle_import --no-fail-fast` | Passed: 7 tests |
| Rust check | `cargo check -p nako-api -p nako-server --tests` | Passed |
| Format | `cargo fmt --all -- --check` | Passed |
| Diff check | `git diff --check` | Passed |

## Evidence Log

- 2026-05-28: SFR-010 opened the lane and selected media probe streams as the
  sidecar subtitle fact read model.
- 2026-05-28: SFR-020 exposed stream `origin` and `disposition` through the
  public media stream DTO and generated Admin TypeScript contract.
- 2026-05-28: SFR-030 refreshed sidecar subtitle facts after import apply and
  covered idempotent repeated apply.
- 2026-05-28: SFR-040 passed focused API/server gates and closeout checks.
