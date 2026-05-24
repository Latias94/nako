# Addon Ecosystem Foundation — Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-db addon_task --no-fail-fast
```

This is the first implementation gate because Addon Task idempotency is the
smallest correctness slice with a clear failure mode.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-ecosystem-foundation/WORKSTREAM.json > $null
git diff --check
```

### Addon Task Gate

```powershell
cargo nextest run -p nako-db addon_task --no-fail-fast
cargo nextest run -p nako-server addon_task --no-fail-fast
```

### Catalog Drift Gate

```powershell
cargo nextest run -p nako-server addon_source_catalog --no-fail-fast
cargo nextest run -p nako-official-addon-catalog --no-fail-fast
```

Run the matching official addon manifest/config tests in
`F:\SourceCodes\Rust\nako-official-addons` after cross-repo catalog changes.

### Addon Event Delivery Gate

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
```

### Formatting And Broader Rust Gate

```powershell
cargo fmt --all -- --check
cargo check -p nako-core -p nako-db -p nako-addon-client -p nako-server --tests
```

Use `cargo nextest run --workspace --no-fail-fast` before final closeout when
the touched scope justifies the full gate.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks in this file or in
`HANDOFF.md`.

## Evidence Anchors

- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `CONTEXT.md`
- `docs/workstreams/addon-ecosystem-foundation/DESIGN.md`
- `docs/workstreams/addon-ecosystem-foundation/TODO.md`
- `docs/workstreams/addon-ecosystem-foundation/MILESTONES.md`
- `crates/nako-core/src/addon_task.rs`
- `crates/nako-db/src/sqlite/addon_tasks.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/addons/task_runtime.rs`
- `crates/nako-official-addon-catalog`
- `crates/nako-addon-client`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-metadata-scraper`

## Recorded Evidence

### 2026-05-25 — AEF-020 Addon Task Request Fingerprints

Claim: Addon Task run creation now stores a deterministic request fingerprint;
same addon id plus idempotency key plus matching fingerprint replays the
existing run, while the same key with different request facts returns a safe
conflict.

Commands:

```powershell
cargo nextest run -p nako-db sqlite_event_addon_automation_contract_addon_task_run_idempotency_fingerprint --no-fail-fast
cargo nextest run -p nako-server addon_task_run_runtime_is_host_owned_and_reports_progress_result --no-fail-fast
rustfmt --edition 2024 --check crates\nako-core\src\addon_task.rs crates\nako-db\src\contract_tests.rs crates\nako-db\src\sqlite\addon_tasks.rs crates\nako-db\src\postgres\addon_tasks.rs crates\nako-server\src\app\addons\task_runtime.rs crates\nako-server\src\http\tests\addons.rs
cargo check -p nako-core -p nako-db -p nako-server --tests
python -m json.tool docs\workstreams\addon-ecosystem-foundation\WORKSTREAM.json > $null
git diff --check
```

Result: passed. `git diff --check` emitted CRLF conversion warnings only.

Notes:

- PostgreSQL behavior is covered by the same contract function behind the
  ignored `postgres_event_addon_automation_contract_addon_task_run_idempotency_fingerprint`
  gate; it was not run because `NAKO_TEST_POSTGRES_URL` is not configured for
  this local pass.
- The broad worktree has unrelated concurrent changes outside this task, so
  formatting was checked on the touched Rust files rather than with
  `cargo fmt --all -- --check`.

### 2026-05-25 — AEF-030 Official Catalog Drift Prevention

Claim: Nako's built-in official metadata scraper catalog descriptor and the
official metadata scraper runtime manifest now share the same catalog facts for
version, protocol, resources, diagnostics entry point, hosted page,
configuration schema, task declaration, runtime reference, and default provider
toggles.

Commands:

```powershell
cargo nextest run -p nako-server addon_source_catalog --no-fail-fast
cargo nextest run -p nako-official-addon-catalog --no-fail-fast
cargo nextest run -p nako-metadata-scraper addon_manifest checked_in_example_manifest_matches_runtime_manifest --no-fail-fast
cargo check -p nako-official-addon-catalog -p nako-server --tests
cargo check -p nako-metadata-scraper --tests
```

Result: passed.

Notes:

- `nako-official-addon-catalog` is the shared facts crate. Nako server uses it
  for the built-in official source catalog descriptor, while the official
  metadata scraper uses it to build its runtime manifest.
- Nako's source catalog remains discovery and install-guide data only: the
  catalog resolve test still proves no registration or job is created and no
  package/process/container lifecycle control is implied.
- Official addon tests ran in `F:\SourceCodes\Rust\nako-official-addons`.

## Notes

- Addon Event Delivery must remain distinct from webhook delivery even if both
  read from durable event outbox concepts.
- Addon Package and Addon Suite metadata must not weaken Addon manifest,
  protocol, scope, grant, or token checks.
- Network Tunnel Provider behavior remains outside Nako core until a future
  ADR explicitly grants that authority.
