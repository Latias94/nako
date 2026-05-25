# Addon Event Scheduler And Replay — Evidence And Gates

Status: Complete
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

### In-Flight Guard And Automatic Retry Gate

```powershell
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_delivery --no-fail-fast
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

### 2026-05-25 — AESR-020 Due Work Selection

Claim: Addon Event scheduler due work can be listed from durable state without
loading or returning raw outbox payload values to admin diagnostics. The query
covers enabled addons, matching event subscription routing plans, SQLite and
PostgreSQL repository parity, attempt summaries, retry timestamps, succeeded
attempt suppression, and manifest/grant checks in the server diagnostic layer.

Commands:

```powershell
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_delivery --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
git diff --check
```

Result: passed.

Additional check:

```powershell
cargo fmt --all -- --check
```

Result: blocked by pre-existing parallel scan-addon worktree changes in
`crates/nako-server/src/app/addons.rs`,
`crates/nako-server/src/app/jobs.rs`, and
`crates/nako-server/src/app/tests/startup.rs`. AESR-020 touched files were
formatted with targeted `rustfmt --edition 2024`.

### 2026-05-25 — AESR-030 In-Flight Guard And Automatic Retry

Claim: Addon Event delivery now uses a durable repository claim before any
sidecar call. The claim writes a `running` attempt with `lease_expires_at`,
prevents duplicate active claims for the same addon/event/subscription tuple,
allows recovery after lease expiry, respects max attempts, and consumes
`next_retry_at` so retryable failures are skipped until due.

Commands:

```powershell
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_delivery --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/addon-event-scheduler-and-replay/WORKSTREAM.json > $null
```

Result: passed.

Additional formatting check:

```powershell
rustfmt --edition 2024 crates/nako-api/src/extension.rs crates/nako-core/src/addon_event.rs crates/nako-core/src/repository/addon_event.rs crates/nako-db/src/contract_tests.rs crates/nako-db/src/facade.rs crates/nako-db/src/postgres.rs crates/nako-db/src/postgres/events.rs crates/nako-db/src/sqlite/addon_events.rs crates/nako-db/src/sqlite/codec.rs crates/nako-db/src/sqlite/migrations.rs crates/nako-server/src/app/addons/event_runtime.rs crates/nako-server/src/http/tests/addons.rs
```

Result: passed.

### 2026-05-25 — AESR-040 Scheduler Runtime Integration

Claim: Addon Event scheduling is wired into server runtime lifecycle behind an
explicit disabled-by-default config block. When enabled, a supervised background
loop scans pending outbox events, dispatches due/retry-due work through the
existing durable delivery path, honors configured event concurrency, exposes
startup diagnostics, and stops with runtime shutdown. Manual admin delivery
remains available.

Commands:

```powershell
cargo nextest run -p nako-server addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event_delivery --no-fail-fast
cargo nextest run -p nako-server config_round_trips_from_toml --no-fail-fast
cargo nextest run -p nako-api admin_overview_response_serializes_safe_summary_fields --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/addon-event-scheduler-and-replay/WORKSTREAM.json > $null
```

Result: passed. `git diff --check` emitted only existing CRLF conversion
warnings from Git on Windows.

Broader replay/filter and DB scheduler gates were not rerun for AESR-040 because
this slice did not change replay, filter, or repository claim/query code.

Review: no blocking workstream-compliance or code-quality findings. Residual
risk: scheduler event selection currently uses the existing outbox listing page
instead of an oldest-first scheduler-specific repository query; split a follow-up
if sustained high-volume event production requires stronger fairness guarantees.

### 2026-05-25 — AESR-050 Forced Replay And Filters

Claim: Addon Event replay is an explicit admin action with operator intent and
durable audit fields, separate from normal delivery. Normal delivery still skips
already succeeded subscriptions, forced replay writes a new attempt with
`forced_replay` and `replay_reason_code`, and host-side subscription filters
evaluate simple event facts before durable claims and sidecar calls without
exposing raw outbox payload values in responses.
Matching event fact filters continue to allow delivery, so non-empty filters do
not become an accidental blanket deny.

Commands:

```powershell
cargo nextest run -p nako-server addon_event_replay addon_event_filter --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler addon_event_delivery --no-fail-fast
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/addon-event-scheduler-and-replay/WORKSTREAM.json > $null
```

Result: passed. `git diff --check` emitted only Windows CRLF conversion
warnings.

Review: filter execution is intentionally limited to deterministic event facts
(`event_kind`, subject kind/id, `library_id`, and `source_id`). Payload or nested
predicate matching should be split into a separate ADR instead of extending this
lane implicitly.

### 2026-05-25 — AESR-060 Closeout

Claim: Addon Event Scheduler And Replay is complete. The lane now has automatic
due-work scheduling, durable in-flight suppression, retry consumption, bounded
runtime lifecycle integration, explicit forced replay with operator intent,
simple redaction-safe host-side event fact filters, and final closeout docs.

Commands:

```powershell
python -m json.tool docs\workstreams\addon-event-scheduler-and-replay\WORKSTREAM.json > $null
git diff --check
cargo fmt --all -- --check
cargo nextest run -p nako-server addon_event_replay addon_event_filter --no-fail-fast
cargo nextest run -p nako-server addon_event_scheduler addon_event_delivery --no-fail-fast
cargo nextest run -p nako-db addon_event_scheduler --no-fail-fast
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
cargo nextest run -p nako-addon-client calls_declared_event_subscription_path_with_event_envelope --no-fail-fast
cargo check -p nako-core -p nako-db -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-server --tests
```

Result: passed. `git diff --check` emitted only Windows CRLF conversion
warnings.

Review: no blocking workstream-compliance or code-quality findings. Residual
risk remains intentionally bounded: filter matching is limited to deterministic
event facts, and notification bridge/provider breadth is deferred to a separate
follow-on lane.

## Notes

- Do not restart implementation in this closed lane. Open a separate
  notification bridge workstream before adding provider fan-out.
- Forced replay must remain separate from normal scheduled delivery.
- Do not expose raw outbox payloads in scheduler diagnostics.
