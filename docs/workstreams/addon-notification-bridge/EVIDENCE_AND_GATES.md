# Addon Notification Bridge — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
```

This proves the host scheduler/replay substrate that notification bridge should
reuse. The official sidecar gate is recorded below for ANB-020 and ANB-030.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-bridge/WORKSTREAM.json > $null
git diff --check
```

### Host Event Gate

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
```

### Official Addon Gate

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
```

Run from `F:\SourceCodes\Rust\nako-official-addons`.

### Closeout Hygiene

```powershell
cargo fmt --all -- --check
git diff --check
```

Use the Nako host command from `F:\SourceCodes\Rust\nako` and the official addon
command from `F:\SourceCodes\Rust\nako-official-addons`.

## Evidence Anchors

- `docs/workstreams/addon-notification-bridge/DESIGN.md`
- `docs/workstreams/addon-notification-bridge/TODO.md`
- `docs/workstreams/addon-event-scheduler-and-replay/EVIDENCE_AND_GATES.md`
- `docs/workstreams/addon-notification-provider-adapters/`
- `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `F:\SourceCodes\Rust\nako-official-addons`

## Recorded Evidence

### 2026-05-25 — ANB-010 Scope And Evidence Freeze

Claim: Addon Notification Bridge is opened as a separate follow-on to Addon
Event Scheduler And Replay, with provider credentials and provider fan-out kept
outside Nako core.

Commands:

```powershell
python -m json.tool docs/workstreams/addon-notification-bridge/WORKSTREAM.json > $null
git diff --check
```

Result: passed. `git diff --check` emitted only Windows CRLF conversion
warnings.

### 2026-05-25 — ANB-020 Official Addon ACK Proof

Claim: The official addons repository now contains a minimal
`nako-notification-bridge` sidecar proof. It declares a `library.scanned` event
subscription, exposes a valid manifest and health endpoint, accepts the Addon
Event envelope, returns a redaction-safe ACK with payload keys only, and does
not implement Telegram, Discord, Home Assistant, email, or other provider
fan-out.

Commands, run from `F:\SourceCodes\Rust\nako-official-addons`:

```powershell
cargo nextest run -p nako-notification-bridge manifest event --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-notification-bridge --tests
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
```

Result: passed. Focused gate ran 5 tests; package gate ran 6 tests.

Review: The proof found that current Addon manifest validation rejects
event-only manifests with `EmptyResources`, so the bridge declares a narrow
`webhook` resource pointing at the same event ACK path. This keeps the current
protocol contract unchanged while preserving sidecar ownership of provider
credentials and fan-out.

### 2026-05-25 — ANB-030 Host Scheduler Proof

Claim: Nako can expose the official notification bridge through the built-in
official Addon catalog, register its manifest, health-check the sidecar contract,
sync an executable Addon Event routing plan, and deliver a scheduled
`library.scanned` event to the bridge ACK path through the existing Addon Event
scheduler. No new scheduler semantics or provider fan-out were added to Nako
core.

Changes:

- Added `notification_bridge` facts and install descriptors to
  `nako-official-addon-catalog`.
- Added the notification bridge descriptor to Nako's built-in official Addon
  catalog source.
- Added host tests for catalog resolve and scheduled
  `library.scanned` delivery to the notification bridge ACK path.
- Fixed the official sidecar health response and smoke script so
  `resource_count` matches the declared webhook resource.

Commands:

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
cargo nextest run -p nako-official-addon-catalog notification_bridge --no-fail-fast
python -m json.tool docs\workstreams\addon-notification-bridge\WORKSTREAM.json > $null
cargo fmt --all -- --check
git diff --check
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
git diff --check
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
```

Result: passed. `nako-server addon_event` ran 13 tests, including
`addon_event_scheduler_acknowledges_official_notification_bridge`;
`nako-db event` ran 10 tests; `nako-official-addon-catalog notification_bridge`
ran 2 tests; `nako-notification-bridge` ran 6 tests. Both `git diff --check`
commands exited successfully with only Windows CRLF conversion warnings.

### 2026-05-25 — ANB-040 Provider Split Decision

Claim: Real notification provider adapters are split out of the ACK-only
notification bridge lane into a named follow-on, so provider credentials,
message templates, outbound provider calls, and provider-specific retry remain
sidecar-owned and do not enter Nako core by default.

Decision:

- Do not implement Telegram, Discord, Home Assistant, email, generic webhook, or
  another provider in this lane.
- Open `docs/workstreams/addon-notification-provider-adapters/` as the follow-on
  lane.
- Make ANP-010 the first task: select one provider target or split again before
  implementation starts.

Commands:

```powershell
python -m json.tool docs\workstreams\addon-notification-bridge\WORKSTREAM.json > $null
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
git diff --check
```

Result: passed. Both `WORKSTREAM.json` files parsed successfully. `git diff
--check` exited successfully with only Windows CRLF conversion warnings from the
dirty worktree.

### 2026-05-25 — ANB-050 Closeout

Claim: Addon Notification Bridge is complete. The ACK-only sidecar exists,
Nako can expose and register its official catalog descriptor, the existing
Addon Event scheduler can deliver `library.scanned` to its ACK path, health
contract drift is fixed, and real provider adapters are split into a named
follow-on.

Review: No blocking workstream-compliance or code-quality findings remained
after the stale ANB-030 handoff note and stale ANB-020 evidence note were
updated.

Commands:

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
cargo nextest run -p nako-db event --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
python -m json.tool docs\workstreams\addon-notification-bridge\WORKSTREAM.json > $null
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
cargo fmt --all -- --check
git diff --check
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
git diff --check
```

Result: passed. `nako-server addon_event` ran 13 tests, `nako-db event` ran 10
tests, and `nako-notification-bridge` ran 6 tests. Both repositories passed
`cargo fmt --all -- --check`; both `git diff --check` commands exited
successfully with only Windows CRLF conversion warnings.

## Notes

- Do not add notification provider credentials to Nako core.
- Do not restart scheduler/replay implementation inside this lane.
- Real provider adapters are split into
  `docs/workstreams/addon-notification-provider-adapters/`.
