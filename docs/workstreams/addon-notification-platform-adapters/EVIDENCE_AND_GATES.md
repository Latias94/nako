# Addon Notification Platform Adapters — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-notification-bridge discord --no-fail-fast
```

Run from `F:\SourceCodes\Rust\nako-official-addons` after NPL-020 creates the
first `discord_webhook` tests.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-platform-adapters/WORKSTREAM.json > $null
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

### Host Gate

Only required if manifest facts, Addon Protocol, host scheduler behavior, or
official catalog facts change:

```powershell
cargo nextest run -p nako-official-addon-catalog notification_bridge --no-fail-fast
```

Run from `F:\SourceCodes\Rust\nako`.

## Evidence Anchors

- `docs/workstreams/addon-notification-provider-adapters/EVIDENCE_AND_GATES.md`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`

## Recorded Evidence

### 2026-05-25 — NPL-010 Scope And Adapter Freeze

Claim: `discord_webhook` is the first named platform adapter. It is selected
because it can be validated with local HTTP fixtures, needs no bot account for
the default path, and has a concrete platform payload shape distinct from
generic `http_webhook`.

Result: passed. Scope remains sidecar-only; Nako core provider concepts,
provider credentials, templates, platform retry queues, and live CI secrets
remain out of scope.

### 2026-05-25 — NPL-020 Through NPL-040 Platform Adapter Proof

Claim: `nako-notification-bridge` now supports a default-disabled
`discord_webhook` provider with redaction-safe diagnostics, fixture-backed
send behavior, fail-closed multi-provider protection, operator docs, and
default smoke assertions.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\config.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\discord_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\lib.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\routes.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`
- `F:\SourceCodes\Rust\nako-official-addons\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\CHANGELOG.md`

Behavior proven:

- default `discord_webhook` state is disabled;
- health and diagnostics expose only safe provider status and booleans;
- configured `discord_webhook` sends exactly one Discord-compatible HTTP
  webhook request to a local fixture;
- outbound Discord payload contains fixed text, event facts, and payload keys,
  not raw event payload values;
- ACK output and safe errors do not echo webhook URLs or raw payload values;
- simultaneous configured provider send paths fail closed before any provider
  request is sent;
- default local smoke still passes with all provider send paths disabled.

Commands:

```powershell
cargo fmt --package nako-notification-bridge
cargo nextest run -p nako-notification-bridge discord --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
cargo build -p nako-notification-bridge
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
```

Result: passed. Focused Discord nextest ran 4 tests. Full
`nako-notification-bridge` nextest ran 19 tests. Default local smoke passed
against a temporary sidecar on `127.0.0.1:19110`.

### 2026-05-25 — NPL-050 Closeout

Claim: Addon Notification Platform Adapters is complete for the first named
platform adapter. `discord_webhook` is implemented, documented, default
disabled, fixture-tested, and verified without moving provider concepts into
Nako core.

Review: No blocking workstream-compliance or code-quality finding remained.
The lane target state is met, multi-provider duplicate-send risk is handled by
fail-closed configuration validation, and additional platform breadth is
deferred to future named adapter lanes.

Final commands:

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
git diff --check
python -m json.tool docs\workstreams\addon-notification-platform-adapters\WORKSTREAM.json
git diff --check -- docs/workstreams/README.md docs/workstreams/addon-notification-platform-adapters docs/workstreams/addon-notification-template-controls docs/workstreams/addon-notification-provider-attempt-history docs/workstreams/addon-notification-provider-live-smoke
```

Result: passed. Diff checks emitted only Windows CRLF conversion warnings from
the dirty worktrees.
