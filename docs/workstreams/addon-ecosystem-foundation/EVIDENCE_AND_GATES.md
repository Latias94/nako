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
- `crates/nako-addon-client`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-metadata-scraper`

## Notes

- Addon Event Delivery must remain distinct from webhook delivery even if both
  read from durable event outbox concepts.
- Addon Package and Addon Suite metadata must not weaken Addon manifest,
  protocol, scope, grant, or token checks.
- Network Tunnel Provider behavior remains outside Nako core until a future
  ADR explicitly grants that authority.
