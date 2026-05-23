# Addon Task Runtime Contract - Evidence And Gates

Status: Active
Last updated: 2026-05-23

## Required Gates

Baseline docs gate:

```bash
git diff --check
```

Future Rust gate, once task runtime code exists:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-server addon --no-fail-fast
```

Future runtime smoke, once a task surface exists:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | ATRC-010 | Lane opened after the source catalog and manager lifecycle lanes clarified their own boundaries. | Pass |
| 2026-05-23 | ATRC-010 | Reference study completed against the existing Addon Task declaration and routing-plan model; bulk-metadata-scrape remains a declaration example, not a runtime contract. | Pass |
| 2026-05-23 | ATRC-020/030 | Added `AddonTaskRun` domain/repository model, SQLite/PostgreSQL `addon_task_runs`, Admin create/list/get/retry routes, Addon runtime claim/progress/complete/fail/cancel routes, and redaction-safe DTOs. | Pass |
| 2026-05-23 | ATRC-030 | Claim/progress lease responses now carry host-authored execution input for the Addon Sidecar while the admin summary remains redaction-safe. | Pass |
| 2026-05-23 | ATRC-030 | `cargo check -p nako-api -p nako-server` | Pass |
| 2026-05-23 | ATRC-030 | `cargo nextest run -p nako-server addon_task_run --no-fail-fast` | Pass, 3 tests passed |
| 2026-05-23 | ATRC-030 | `cargo fmt --all -- --check` | Pass |
| 2026-05-23 | ATRC-030 | `git diff --check` | Pass |
| 2026-05-23 | ATRC-030 | `cargo nextest run -p nako-server addon --no-fail-fast` | Pass, 58 tests passed |
| 2026-05-23 | ATRC-040 | Added Addon Task request/response protocol envelopes, direct dispatch mode, client task-path call outcome, target `job_id` claim filtering, and server direct dispatch terminal handling. | Pass |
| 2026-05-23 | ATRC-040 | `cargo check -p nako-api -p nako-server` | Pass |
| 2026-05-23 | ATRC-040 | `cargo nextest run -p nako-addon-client calls_declared_task_path_with_host_owned_run_envelope --no-fail-fast` | Pass, 1 test passed |
| 2026-05-23 | ATRC-040 | `cargo nextest run -p nako-server addon_task_run_direct_dispatch --no-fail-fast` | Pass, 3 tests passed |
| 2026-05-23 | ATRC-040 | `cargo nextest run -p nako-server addon_task_run_ --no-fail-fast` | Pass, 6 tests passed |
| 2026-05-23 | ATRC-040 | `cargo fmt --all -- --check` | Pass |
| 2026-05-23 | ATRC-040 | `git diff --check` | Pass |
| 2026-05-23 | ATRC-040 | `cargo nextest run -p nako-server addon --no-fail-fast` | Pass, 61 tests passed |

## Closeout Evidence

Closeout requires:

- a clear host-owned task runtime boundary;
- fresh docs and runtime gates;
- explicit split/defer notes for source catalog, package signing, provider
  breadth, process supervision, and authenticated outbound sidecar dispatch
  credential management.
