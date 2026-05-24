# Addon Outbound Task Dispatch Credentials - Evidence And Gates

Status: Completed
Last updated: 2026-05-25

## Required Gates

Baseline docs gate:

```bash
git diff --check
```

Initial compile gates:

```bash
cargo fmt --all -- --check
cargo check -p nako-addon-client -p nako-server
```

Validation gates once the first implementation slice exists:

```bash
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-server addon_task_run_direct_dispatch --no-fail-fast
cargo nextest run -p nako-server addon --no-fail-fast
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-24 | AOTDC-010 | Current direct dispatch in `crates/nako-server/src/app/addons/task_runtime.rs` passes no outbound credential into `call_addon_task_with_outcome`; `crates/nako-addon-client/src/lib.rs` already injects bearer and shared-secret headers when a token is supplied. | Pass |
| 2026-05-24 | AOTDC-020 | `outbound_task_dispatch_secret_env` now persists through addon registration, the Admin API summary surface exposes the env reference, and `crates/nako-server/src/app/addons.rs` provides host-side env resolution with safe failure text. | Pass |
| 2026-05-24 | AOTDC-030 | `crates/nako-server/src/app/addons/task_runtime.rs` now resolves host-owned outbound credentials for direct dispatch, injects bearer/shared-secret headers through `call_addon_task_with_outcome`, and fails safely when the configured secret reference is missing. Verified with `cargo check -p nako-addon-client -p nako-server`, `cargo nextest run -p nako-addon-client --no-fail-fast`, `cargo nextest run -p nako-server addon --no-fail-fast`, focused direct-dispatch cases, and `git diff --check`. | Pass |
| 2026-05-25 | AOTDC-060 | Closeout verification passed: `cargo fmt --all -- --check`; `cargo check -p nako-addon-client -p nako-server` with existing `nako-server` dead-code warnings only; `cargo nextest run -p nako-addon-client --no-fail-fast` passed 15 tests; `cargo nextest run -p nako-server addon_task_run_direct_dispatch --no-fail-fast` passed 6 tests; `cargo nextest run -p nako-server addon --no-fail-fast` passed 69 tests; `git diff --check` passed. | Pass |

## Closeout Evidence

Closeout completed on 2026-05-25 with:

- a clear outbound credential storage and resolution boundary using
  `outbound_task_dispatch_secret_env`;
- fresh docs and runtime gates;
- explicit split notes for any vault/provider abstraction that proves wider
  than env-backed references.
