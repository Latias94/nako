# Addon Notification Provider Adapters — Evidence And Gates

Status: Complete
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
```

Run from `F:\SourceCodes\Rust\nako-official-addons`.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/addon-notification-provider-adapters/WORKSTREAM.json > $null
git diff --check
```

### Official Addon Gate

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
```

Run from `F:\SourceCodes\Rust\nako-official-addons`.

### Host Gate

Only required if a provider adapter changes manifest, protocol, scheduler, or
host diagnostics behavior:

```powershell
cargo nextest run -p nako-server addon_event --no-fail-fast
```

## Evidence Anchors

- `docs/workstreams/addon-notification-provider-adapters/DESIGN.md`
- `docs/workstreams/addon-notification-provider-adapters/TODO.md`
- `docs/workstreams/addon-notification-bridge/EVIDENCE_AND_GATES.md`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge`

## Recorded Evidence

### 2026-05-25 — Follow-On Opened From ANB-040

Claim: Provider breadth is split from the ACK-only notification bridge lane into
this named follow-on so implementation cannot start before provider selection,
credential ownership, template ownership, retry behavior, and redaction
requirements are explicit.

Commands:

```powershell
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
git diff --check
```

Result: passed. `git diff --check` emitted only Windows CRLF conversion warnings
from the dirty worktree.

Resolution: ANP-010 recorded the provider selection decision in the evidence
entry below before implementation starts.

### 2026-05-25 — ANP-010 Provider Selection Freeze

Claim: ANP-010 selected `http_webhook` as the first real notification provider
adapter target. The adapter is an outbound HTTP webhook sink owned by
`nako-notification-bridge`, not by Nako core.

Decision evidence:

- local fixture validation is possible without live CI secrets or platform
  accounts;
- webhook target URL, optional shared secret/header, outbound provider call,
  payload shaping, and provider diagnostics remain sidecar/operator-owned;
- Nako core keeps only event scheduling and delivery-attempt ownership;
- provider retry initially reuses the existing Addon Event delivery retry by
  returning safe retryable failures to the host, with any richer sidecar queue
  split into later work.

Commands:

```powershell
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
git diff --check
```

Result: passed. `git diff --check` emitted only Windows CRLF conversion
warnings from the dirty worktree.

Resolution: ANP-020 added sidecar configuration, secret-reference docs, and
redaction-safe diagnostics for `http_webhook` before any send path was added.

### 2026-05-25 — ANP-020 Provider Configuration Contract

Claim: `nako-notification-bridge` now has a sidecar-owned `http_webhook`
configuration contract and redaction-safe diagnostics without enabling provider
sends.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\config.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\routes.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`

Behavior proven:

- default `http_webhook` state is disabled;
- configured/missing/invalid provider states are explicit;
- health JSON and `/ui/diagnostics` expose only safe booleans/status strings;
- raw webhook URL, shared secret, and custom header name are not echoed by
  diagnostics;
- `library.scanned` remains ACK-only and provider fan-out stays disabled.

Commands:

```powershell
cargo fmt --package nako-notification-bridge
cargo nextest run -p nako-notification-bridge http_webhook --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --package nako-notification-bridge -- --check
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
```

Result: passed. Focused `http_webhook` nextest ran 5 tests; full package
nextest ran 11 tests. Local smoke passed against a temporary sidecar on
`127.0.0.1:19110`.

Skipped: no host gate yet. ANP-020 does not change the host protocol, event
scheduler, or Nako-managed provider concepts. The checked-in notification
manifest still has no Nako-owned provider secret references.

Resolution: ANP-030 implemented the first fixture-backed send path and mapped
retryable provider HTTP failures to safe sidecar failures.

### 2026-05-25 — ANP-030 First Provider Send Path

Claim: `nako-notification-bridge` now sends a fixed redaction-safe
`http_webhook` payload behind the existing `library.scanned` event route when
the provider is explicitly configured.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\Cargo.toml`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\http_webhook.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\routes.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\src\config.rs`
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge`

Behavior proven:

- default sidecar behavior remains ACK-only when `http_webhook` is disabled;
- configured sidecar sends exactly one outbound HTTP webhook request to a local
  fixture server;
- outbound payload contains event facts and sorted payload keys, not raw event
  payload values;
- optional shared secret is sent to the provider header but not echoed in ACK
  output or safe diagnostics;
- provider `429` maps to a safe retryable sidecar failure (`503`);
- provider `400` maps to a safe non-retryable sidecar failure (`424`);
- package smoke still passes with the default provider-disabled configuration.

Commands:

```powershell
cargo nextest run -p nako-notification-bridge library_scanned_event_endpoint_sends_http_webhook_payload_without_raw_event_values --no-fail-fast
cargo nextest run -p nako-notification-bridge library_scanned_event_endpoint_returns_retryable_safe_failure_for_rate_limited_http_webhook --no-fail-fast
cargo nextest run -p nako-notification-bridge library_scanned_event_endpoint_returns_non_retryable_safe_failure_for_provider_rejection --no-fail-fast
cargo fmt --package nako-notification-bridge
cargo check -p nako-notification-bridge --tests
cargo nextest run -p nako-notification-bridge http_webhook --no-fail-fast
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo fmt --package nako-notification-bridge -- --check
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
```

Result: passed. Full package nextest ran 14 tests. Focused `http_webhook`
nextest ran 7 tests. Default local smoke passed against a temporary sidecar on
`127.0.0.1:19110`.

Skipped: no host gate yet. ANP-030 does not change the manifest, Addon
Protocol, host scheduler, or Nako-owned provider concepts.

Resolution: ANP-040 finished integration/docs and ran a focused host catalog
gate because the manifest facts remain unchanged.

### 2026-05-25 — ANP-040 Integration And Docs

Claim: Official addon docs, default smoke, and workstream evidence now reflect
the implemented `http_webhook` provider send path, while preserving the
sidecar-owned credential/provider boundary.

Changed scope:

- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge\README.md`
- `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge\smoke.local.ps1`
- `docs/workstreams/addon-notification-provider-adapters`

Behavior proven:

- default sidecar smoke still passes with `http_webhook` disabled;
- default smoke asserts health provider status and event ACK provider status;
- official addon full gate passes after docs/smoke updates;
- focused host catalog tests still pass, proving the official catalog facts
  remain aligned with the unchanged notification bridge manifest shape.

Commands:

```powershell
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
cargo nextest run -p nako-official-addon-catalog notification_bridge --no-fail-fast
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
git diff --check
```

Result: passed. Official addon nextest ran 14 tests. Focused host catalog gate
ran 2 tests. `git diff --check` emitted only Windows CRLF conversion warnings
from the dirty worktrees.

Host gate decision: no broader host gate was required because ANP-040 did not
change the Addon Protocol, host scheduler, manifest schema, or Nako-owned
provider concepts. The focused catalog gate was run as a consistency check.

Resolution: ANP-050 reviewed and closed the lane; remaining provider breadth is
named as follow-on work.

### 2026-05-25 — ANP-050 Closeout

Claim: Addon Notification Provider Adapters is complete. The first provider
adapter, `http_webhook`, is selected, configured, implemented, verified, and
documented while provider ownership remains sidecar-only.

Review: No blocking workstream-compliance or code-quality findings remained at
closeout. The lane target state is met, all task ledger entries are complete,
operator docs describe the shipped behavior, and no journal-only decisions
remain.

Named follow-ons:

- `addon-notification-platform-adapters`
- `addon-notification-template-controls`
- `addon-notification-provider-attempt-history`
- `addon-notification-provider-live-smoke`

Final commands:

```powershell
pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo check -p nako-notification-bridge --tests
cargo fmt --all -- --check
cargo nextest run -p nako-official-addon-catalog notification_bridge --no-fail-fast
python -m json.tool docs\workstreams\addon-notification-provider-adapters\WORKSTREAM.json > $null
git diff --check
```

Result: passed. Official addon nextest ran 14 tests. Focused host catalog gate
ran 2 tests. Default local smoke passed against a temporary sidecar on
`127.0.0.1:19110`. `git diff --check` emitted only Windows CRLF conversion
warnings from the dirty worktrees.
