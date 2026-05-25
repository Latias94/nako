# Addon Notification Template Controls — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Safe Template Contract

- [x] NTC-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-template-controls]
  Goal: Freeze allowed tokens, rejected syntax, diagnostics behavior, and
  provider integration order.
  Validation: `python -m json.tool docs/workstreams/addon-notification-template-controls/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm raw payload values remain unavailable to templates.
  Evidence: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`.
  Result: Frozen to a small sidecar-owned token whitelist. General template
  engines, raw payload values, and Nako-managed templates are out of scope.
  Handoff: Continue with NTC-020.

## M1 — Renderer Proof

- [x] NTC-020 [owner=codex] [deps=NTC-010] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Add a small renderer for whitelisted notification tokens with tests for
  unknown token rejection and raw payload value exclusion.
  Validation: `cargo nextest run -p nako-notification-bridge template --no-fail-fast`.
  Review: Confirm no general-purpose template engine or raw JSON traversal is
  introduced.
  Evidence: renderer unit tests.
  Result: Added a small whitelist renderer with tests for allowed tokens,
  unknown token rejection, malformed token rejection, and raw payload value
  exclusion.
  Handoff: Continue with NTC-030.

## M2 — Provider Wiring

- [x] NTC-030 [owner=codex] [deps=NTC-020] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Wire safe templates into configured providers while preserving default
  fixed summaries.
  Validation: `cargo nextest run -p nako-notification-bridge template --no-fail-fast`
  and `cargo nextest run -p nako-notification-bridge --no-fail-fast`.
  Review: Confirm provider ACK/error output does not echo template literals
  that could contain secrets.
  Evidence: provider fixture tests.
  Result: Wired rendered summaries into provider payloads and fail-closed
  invalid templates before provider sends.
  Handoff: Continue with NTC-040.

## M3 — Docs And Smoke

- [x] NTC-040 [owner=codex] [deps=NTC-030] [scope=F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge,F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge\README.md,docs/workstreams/addon-notification-template-controls]
  Goal: Document safe tokens, defaults, and non-goals; update smoke only if
  default output changes.
  Validation: package gate and smoke gate.
  Review: Confirm docs warn that templates are sidecar/operator-owned.
  Evidence: README and smoke output.
  Result: Documented safe tokens, defaults, diagnostics behavior, and sidecar
  ownership; updated default smoke template diagnostics assertions.
  Handoff: Continue with NTC-050.

## M4 — Closeout

- [x] NTC-050 [owner=planner] [deps=NTC-040] [scope=docs/workstreams/addon-notification-template-controls]
  Goal: Close the lane or split Admin UI/API template management into a later
  workstream.
  Validation: final official addon gates and JSON parse.
  Review: Run review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: Closed after review and final verification. Admin UI/API template
  management remains out of scope.
  Handoff: DONE.
