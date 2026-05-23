# Addon Task Runtime Contract - Evidence And Gates

Status: Completed
Last updated: 2026-05-23

## Closeout Gates

Baseline docs gate:

```bash
git diff --check
```

Closeout Rust gates:

```bash
cargo fmt --all -- --check
cargo check -p nako-api -p nako-server
cargo nextest run -p nako-addon-client calls_declared_task_path_with_host_owned_run_envelope --no-fail-fast
cargo nextest run -p nako-server addon --no-fail-fast
```

## Follow-on Official-Addon Task Smoke

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/official-addon-e2e-smoke.ps1
```

The current official smoke proves hosted health and resource diagnostics. It
does not yet prove task-path dispatch because the published official metadata
scraper does not expose an Addon Task declaration; that task smoke is a
follow-on.

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
| 2026-05-23 | ATRC-060 | Closed the lane and split authenticated outbound task dispatch credential management plus official-addon task-path smoke coverage into follow-ons. | Pass |
| 2026-05-23 | ATRC-060 | `cargo fmt --all -- --check`; `cargo check -p nako-api -p nako-server`; `cargo nextest run -p nako-addon-client calls_declared_task_path_with_host_owned_run_envelope --no-fail-fast`; `cargo nextest run -p nako-server addon --no-fail-fast`; `git diff --check`. | Pass |

## Closeout Evidence

Closeout completed with:

- a clear host-owned task runtime boundary;
- fresh docs and runtime gates;
- explicit split/defer notes for source catalog, package signing, provider
  breadth, process supervision, and authenticated outbound sidecar dispatch
  credential management.

Follow-ons:

- authenticated outbound task dispatch credential storage and resolution for
  `AddonAuth::Bearer` and `AddonAuth::SharedSecret`;
- official-addon task-path smoke coverage;
- Addon Source Catalog / marketplace discovery;
- package signing and trust roots;
- provider breadth beyond the first companion addon;
- direct process/container supervision.
