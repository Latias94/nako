# Addon Notification Provider Adapters — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Provider Selection Freeze

- [x] ANP-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-provider-adapters,F:\SourceCodes\Rust\nako-official-addons]
  Goal: Freeze first-provider selection criteria, pick the first narrow
  provider or explicitly split again, and record credential/template/retry
  ownership before implementation.
  Validation: `python -m json.tool docs/workstreams/addon-notification-provider-adapters/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm Nako core still owns only event scheduling and attempt
  records; provider behavior remains sidecar-owned.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`, and provider decision notes.
  Result: Selected `http_webhook` as the first provider target because it can
  be validated with a local fixture, needs no platform account, keeps provider
  URLs/secrets/templates inside the sidecar/operator boundary, and does not
  require Nako core or Addon Protocol changes.
  Handoff: Continue with ANP-020 configuration and redaction-safe diagnostics
  for `http_webhook`.

## M1 — Provider Configuration Contract

- [x] ANP-020 [owner=codex] [deps=ANP-010] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Add sidecar configuration, secret reference documentation, and
  redaction-safe diagnostics for the selected provider without sending a live
  notification yet.
  Validation: focused `cargo nextest run -p nako-notification-bridge <provider-filter> --no-fail-fast`.
  Review: Check that no raw provider token, recipient, or message body is
  echoed in health, diagnostics, logs, or test assertions.
  Evidence: provider config tests and updated operator docs.
  Result: Added sidecar-owned `http_webhook` env configuration, safe config
  status, redaction-safe health/diagnostics output, operator docs, and smoke
  assertions. The send path remains disabled and no provider call is made.
  Handoff: Continue with ANP-030 for the first provider send path.

## M2 — First Provider Send Path

- [x] ANP-030 [owner=codex] [deps=ANP-020] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-notification-bridge]
  Goal: Implement one provider send path behind the existing
  `library.scanned` event ACK flow, with fixture-backed tests and no live CI
  secrets.
  Validation: `cargo nextest run -p nako-notification-bridge --no-fail-fast`
  plus provider-specific smoke or fixture gate.
  Review: Check provider retry ownership, rate-limit behavior, and
  redaction-safe failure reporting.
  Evidence: provider send tests, smoke fixture, and docs.
  Result: Implemented fixture-backed `http_webhook` sends with fixed
  redaction-safe JSON payloads, optional shared-secret header, retryable
  408/429/5xx/transport failure mapping, non-retryable provider rejection
  mapping, and no live CI secrets.
  Handoff: Continue with ANP-040 for host/doc integration or split if host
  contract changes are needed.

## M3 — Integration And Docs

- [x] ANP-040 [owner=codex] [deps=ANP-030] [scope=docs,F:\SourceCodes\Rust\nako-official-addons]
  Goal: Update official addon docs, smoke commands, and Nako workstream
  evidence for the chosen provider; add host tests only if the manifest or
  protocol contract changes.
  Validation: official addon full gate plus focused host gate when applicable.
  Review: Confirm operator setup is explicit and does not imply Nako manages
  sidecar lifecycle or provider secrets.
  Evidence: README, smoke script, and `EVIDENCE_AND_GATES.md`.
  Result: Updated default smoke to assert provider-disabled ACK output, updated
  packaging smoke notes, verified official addon gates, and ran a focused
  catalog host gate because the host manifest facts remain unchanged.
  Handoff: Continue with ANP-050 closeout.

## M4 — Closeout

- [x] ANP-050 [owner=planner] [deps=ANP-040] [scope=docs/workstreams/addon-notification-provider-adapters]
  Goal: Close the provider adapter lane or split additional providers into
  separate follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: Run review-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Result: Closed the lane after review and final gates. Remaining provider
  breadth is named as follow-on work in `DESIGN.md` and `HANDOFF.md`.
  Handoff: DONE. This lane is closed.
