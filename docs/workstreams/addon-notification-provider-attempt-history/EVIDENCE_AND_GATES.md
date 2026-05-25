# Addon Notification Provider Attempt History — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-notification-bridge attempt_history --no-fail-fast
```

Run from `F:\SourceCodes\Rust\nako-official-addons` after NAH-020 creates the
recorder tests.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-provider-attempt-history/WORKSTREAM.json > $null
git diff --check
```

Run from `F:\SourceCodes\Rust\nako`.

### Official Addon Gate

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
```

Run from `F:\SourceCodes\Rust\nako-official-addons`.

## Evidence Anchors

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge`
- `docs/workstreams/addon-notification-provider-adapters/EVIDENCE_AND_GATES.md`

## Recorded Evidence

### 2026-05-25 — NAH-010 History Contract

Claim: Provider attempt history is sidecar-owned, bounded, in-memory, and
redaction-safe. Persistent storage, provider retry queues, Nako core schema
changes, and Admin Web history UI are out of scope.

Result: passed.

### 2026-05-25 — NAH-020 Through NAH-040 Recorder, Provider Wiring, And Docs

Claim: `nako-notification-bridge` now records recent provider outcomes in a
bounded in-memory ring buffer and exposes them through safe health diagnostics.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\attempt_history.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\config.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\http_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\discord_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\routes.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`
- `F:\SourceCodes\Rust\nako-official-addons\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\CHANGELOG.md`

Behavior proven:

- history capacity is configurable with a safe default of 20;
- recent records are bounded;
- disabled, sent, and failure statuses are safe derived facts;
- records exclude raw provider URLs, secrets, headers, message bodies, and raw
  event payload values;
- Nako core attempt schema and retry behavior are unchanged.

Commands:

```powershell
cargo fmt --package nako-notification-bridge
cargo nextest run -p nako-notification-bridge attempt_history --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
```

Result: passed. Focused attempt-history nextest ran 2 tests. Full
`nako-notification-bridge` nextest ran 27 tests.

### 2026-05-25 — NAH-050 Closeout

Claim: Addon Notification Provider Attempt History is complete for the bounded
in-memory diagnostics slice.

Review: No blocking workstream-compliance or code-quality finding remained.
Persistent history and Admin UI display are future work.
