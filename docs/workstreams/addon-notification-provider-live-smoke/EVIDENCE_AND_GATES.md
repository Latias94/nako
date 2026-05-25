# Addon Notification Provider Live Smoke — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.live.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
pwsh -File addons/notification-bridge/smoke.live.ps1
```

Run from `F:\SourceCodes\Rust\nako-official-addons` after NLS-020 creates the
script. The second command must skip by default unless explicit live-smoke env
vars are set.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-provider-live-smoke/WORKSTREAM.json > $null
git diff --check
```

Run from `F:\SourceCodes\Rust\nako`.

### Live Smoke Script Gate

```powershell
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.live.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
pwsh -File addons/notification-bridge/smoke.live.ps1
```

Run from `F:\SourceCodes\Rust\nako-official-addons`. The default run must skip
without requiring secrets.

### Official Addon Regression Gate

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
```

Run from `F:\SourceCodes\Rust\nako-official-addons`.

## Evidence Anchors

- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`
- `docs/workstreams/addon-notification-provider-adapters/EVIDENCE_AND_GATES.md`

## Recorded Evidence

### 2026-05-25 — NLS-010 Live Smoke Contract

Claim: Live provider smoke is local-only, skipped by default, and requires
explicit `NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE=1` opt-in. CI and package gates
must not require live provider secrets.

Result: passed.

### 2026-05-25 — NLS-020 Through NLS-030 Script And Docs

Claim: `addons/notification-bridge/smoke.live.ps1` exists, parses, skips by
default, and verifies provider-send ACK shape only when explicitly enabled.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge\smoke.live.ps1`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\CHANGELOG.md`

Commands:

```powershell
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.live.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
Remove-Item Env:NAKO_NOTIFICATION_BRIDGE_LIVE_SMOKE -ErrorAction SilentlyContinue; pwsh -File addons/notification-bridge/smoke.live.ps1
```

Result: passed. Default run skipped without requiring provider secrets.

Skipped: enabled live-provider execution was not run because no live provider
secret or endpoint should be required by this lane or CI.

### 2026-05-25 — NLS-040 Closeout

Claim: Addon Notification Provider Live Smoke is complete for the opt-in script
and default-skip behavior.

Review: No blocking workstream-compliance or code-quality finding remained.
