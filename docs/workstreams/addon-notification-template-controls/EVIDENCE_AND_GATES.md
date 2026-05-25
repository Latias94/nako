# Addon Notification Template Controls — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-notification-bridge template --no-fail-fast
```

Run from `F:\SourceCodes\Rust\nako-official-addons` after NTC-020 creates the
renderer tests.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-template-controls/WORKSTREAM.json > $null
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

### 2026-05-25 — NTC-010 Safe Template Contract

Claim: Safe summary template controls are sidecar-owned and limited to a small
whitelist of event facts and payload keys. Raw event payload values, Nako-owned
templates, and general template engines are out of scope.

Result: passed. Allowed tokens are `event_id`, `event_kind`, `subject_kind`,
`subject_id`, `occurred_at`, `attempt`, and `payload_keys`.

### 2026-05-25 — NTC-020 Through NTC-040 Renderer, Provider Wiring, And Docs

Claim: `nako-notification-bridge` now supports safe summary templates through
`NAKO_NOTIFICATION_BRIDGE_TEMPLATE_SUMMARY`. Unknown or malformed tokens fail
closed before provider sends, diagnostics do not echo template text, and
providers receive only rendered summaries from whitelisted event facts.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\template.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\config.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\http_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\discord_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\routes.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`
- `F:\SourceCodes\Rust\nako-official-addons\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\CHANGELOG.md`

Commands:

```powershell
cargo fmt --package nako-notification-bridge
cargo nextest run -p nako-notification-bridge template --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path 'addons\notification-bridge\smoke.local.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
cargo build -p nako-notification-bridge
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
```

Result: passed. Focused template nextest ran 6 tests. Full
`nako-notification-bridge` nextest ran 25 tests. Default local smoke passed
after rebuilding the sidecar binary.

### 2026-05-25 — NTC-050 Closeout

Claim: Addon Notification Template Controls is complete for the safe summary
template slice. The target state is met without adding Nako-managed templates,
raw payload value access, or a general-purpose template engine.

Review: No blocking workstream-compliance or code-quality finding remained.
Admin UI/API template management is deferred.
