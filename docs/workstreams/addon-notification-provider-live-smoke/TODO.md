# Addon Notification Provider Live Smoke — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Live Smoke Contract

- [x] NLS-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-provider-live-smoke]
  Goal: Freeze opt-in behavior, required env vars, redaction rules, and default
  CI skip behavior.
  Validation: `python -m json.tool docs/workstreams/addon-notification-provider-live-smoke/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm no live secret is required by package gates.
  Evidence: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`.
  Result: Frozen to opt-in local-only smoke with default skip and no CI live
  secret requirement.
  Handoff: Continue with NLS-020.

## M1 — Opt-In Script

- [x] NLS-020 [owner=codex] [deps=NLS-010] [scope=F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge]
  Goal: Add an opt-in live smoke script for `http_webhook` provider delivery
  that skips by default and prints only safe status.
  Validation: PowerShell parser check and default skip run.
  Review: Confirm URL, secret, and provider response body are not printed.
  Evidence: script and parser/default-skip output.
  Result: Added `smoke.live.ps1`; parser and default-skip gates passed.
  Handoff: Continue with NLS-030.

## M2 — Docs And Release Notes

- [x] NLS-030 [owner=codex] [deps=NLS-020] [scope=F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge,F:\SourceCodes\Rust\nako-official-addons\README.md,docs/workstreams/addon-notification-provider-live-smoke]
  Goal: Document live smoke usage, required local env vars, and CI exclusion.
  Validation: parser/default-skip gate plus docs review.
  Review: Confirm docs do not contain example real secrets or live URLs.
  Evidence: README and script output.
  Result: Documented opt-in env vars, local-only execution, and CI exclusion.
  Handoff: Continue with NLS-040.

## M3 — Closeout

- [x] NLS-040 [owner=planner] [deps=NLS-030] [scope=docs/workstreams/addon-notification-provider-live-smoke]
  Goal: Close the lane or split platform-specific live smoke into later work.
  Validation: final parser/default-skip gate, JSON parse, and `git diff --check`.
  Review: Run review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: Closed after parser/default-skip verification. Enabled live-provider
  execution remains operator-provided and is not required by CI.
  Handoff: DONE.
