# Addon Notification Platform Adapters — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Scope And Adapter Freeze

- [x] NPL-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-platform-adapters,F:\SourceCodes\Rust\nako-official-addons]
  Goal: Freeze the first named platform adapter, credential ownership, payload
  safety rules, and validation gates.
  Validation: `python -m json.tool docs/workstreams/addon-notification-platform-adapters/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm this lane does not add provider concepts to Nako core.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`.
  Result: Selected `discord_webhook` as the first named platform adapter
  because it can be fixture-tested locally, needs no bot account for the
  default path, and proves a platform payload shape distinct from generic
  `http_webhook`.
  Handoff: Continue with NPL-020 after `discord_webhook` remains the selected
  first platform adapter.

## M1 — Platform Configuration And Diagnostics

- [x] NPL-020 [owner=codex] [deps=NPL-010] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Add default-disabled `discord_webhook` configuration and redaction-safe
  health/diagnostics status without sending provider requests yet.
  Validation: `cargo nextest run -p nako-notification-bridge discord --no-fail-fast`.
  Review: Confirm webhook URLs, shared secrets, and platform-specific fields are
  not echoed in diagnostics, ACK output, logs, or tests.
  Evidence: config and diagnostics tests.
  Result: Added default-disabled `discord_webhook` env configuration and
  redaction-safe health/diagnostics facts.
  Handoff: Continue with NPL-030.

## M2 — Fixture-Backed Platform Send

- [x] NPL-030 [owner=codex] [deps=NPL-020] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Send a fixed safe `library.scanned` notification to a local
  `discord_webhook` fixture when explicitly configured.
  Validation: `cargo nextest run -p nako-notification-bridge discord --no-fail-fast`
  and `cargo check -p nako-notification-bridge --tests`.
  Review: Confirm payload values, secrets, and target URLs remain redacted from
  ACK/error output.
  Evidence: fixture request assertions and retryable/non-retryable failure tests.
  Result: Implemented fixture-backed Discord webhook sends and fail-closed
  multi-provider protection before any provider request is sent.
  Handoff: Continue with NPL-040.

## M3 — Docs And Smoke Alignment

- [x] NPL-040 [owner=codex] [deps=NPL-030] [scope=F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge,F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md,docs/workstreams/addon-notification-platform-adapters]
  Goal: Update operator docs and default smoke assertions for the new disabled
  platform provider.
  Validation: `pwsh -File addons/notification-bridge/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:19110`
  and `cargo nextest run -p nako-notification-bridge --no-fail-fast`.
  Review: Confirm docs do not imply Nako manages platform credentials or
  sidecar lifecycle.
  Evidence: README and smoke output.
  Result: Updated operator docs, compose defaults, root README, CHANGELOG, and
  default smoke assertions for disabled `discord_webhook`.
  Handoff: Continue with NPL-050.

## M4 — Closeout

- [x] NPL-050 [owner=planner] [deps=NPL-040] [scope=docs/workstreams/addon-notification-platform-adapters]
  Goal: Close the lane or split additional platform adapters into follow-ons.
  Validation: final package gates, JSON parse, fmt check, and `git diff --check`.
  Review: Run review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: Closed after review and final verification. Additional platform
  adapters are deferred to future named adapter lanes.
  Handoff: DONE.
