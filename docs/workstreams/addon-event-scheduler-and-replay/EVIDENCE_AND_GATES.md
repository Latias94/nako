# Addon Event Scheduler And Replay — Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
```

This proves the predecessor manual Addon Event Delivery runtime. The first new
implementation gate should narrow to scheduler candidate selection once
AESR-020 adds it.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-event-scheduler-and-replay/WORKSTREAM.json > $null
git diff --check
```

### Due Work Gate

```powershell
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast
```

### Replay And Filter Gate

```powershell
cargo nextest run -p nako-server addon_event_replay --no-fail-fast
cargo nextest run -p nako-server addon_event_filter --no-fail-fast
```

### Existing Event Runtime Regression Gate

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
cargo nextest run -p nako-addon-client calls_declared_event_subscription_path_with_event_envelope --no-fail-fast
```

### Formatting And Broader Rust Gate

```powershell
cargo fmt --all -- --check
cargo check -p nako-core -p nako-db -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
```

Use a narrower closeout gate only when the workspace cost is recorded.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in `HANDOFF.md`.

## Evidence Anchors

- `docs/workstreams/addon-event-scheduler-and-replay/DESIGN.md`
- `docs/workstreams/addon-event-scheduler-and-replay/TODO.md`
- `docs/workstreams/addon-event-scheduler-and-replay/MILESTONES.md`
- `docs/workstreams/addon-ecosystem-foundation/EVIDENCE_AND_GATES.md`
- `crates/nako-core/src/addon_event.rs`
- `crates/nako-core/src/repository/addon_event.rs`
- `crates/nako-db/src/sqlite/addon_events.rs`
- `crates/nako-db/src/postgres/events.rs`
- `crates/nako-server/src/app/addons/event_runtime.rs`
- `crates/nako-server/src/http/addons.rs`

## Recorded Evidence

### 2026-05-25 — AESR-010 Scope And Evidence Freeze

Claim: Addon Event Scheduler And Replay is opened as a durable follow-on to
Addon Ecosystem Foundation, with scheduler/replay scope separated from broad
notification, watch-state, MCP, Arr-stack, compatibility, and tunnel feature
breadth.

Commands:

```powershell
python -m json.tool docs/workstreams/addon-event-scheduler-and-replay/WORKSTREAM.json > $null
git diff --check
cargo fmt --all -- --check
```

Result: passed.

## Notes

- Do not start notification bridge until this lane either ships or explicitly
  accepts scheduler/retry risk.
- Forced replay must remain separate from normal scheduled delivery.
- Do not expose raw outbox payloads in scheduler diagnostics.
