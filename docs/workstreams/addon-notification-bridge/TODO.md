# Addon Notification Bridge — TODO

Status: Complete
Last updated: 2026-05-25

## M0 — Scope And Evidence Freeze

- [x] ANB-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-notification-bridge,docs/workstreams/README.md]
  Goal: Freeze notification bridge boundaries, non-goals, evidence anchors, and
  first implementation slice after AESR closeout.
  Validation: `python -m json.tool docs/workstreams/addon-notification-bridge/WORKSTREAM.json > $null`
  and `git diff --check`.
  Review: Confirm this lane does not restart scheduler/replay, watch-state,
  MCP, Arr-stack, compatibility protocol, or tunnel work.
  Evidence: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`,
  `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Continue with ANB-020.

## M1 — Official Addon ACK Proof

- [x] ANB-020 [owner=codex] [deps=ANB-010] [scope=F:\SourceCodes\Rust\nako-official-addons]
  Goal: Add a minimal official notification bridge Addon proof that declares a
  `library.scanned` event subscription and returns a redaction-safe ACK without
  calling a third-party provider.
  Validation: focused `cargo nextest run -p <notification-crate> manifest event`
  gate, or equivalent package gate once the crate name exists.
  Review: Check that provider secrets, message templates, and provider fan-out
  are not placed in Nako core.
  Evidence: `nako-notification-bridge` manifest/event/health tests,
  checked-in manifest example, local smoke script, Docker/Compose packaging
  skeleton, and docs.
  Handoff: DONE. Continue with ANB-030 for host registration/scheduler proof.

## M2 — Host Scheduler Proof

- [x] ANB-030 [owner=codex] [deps=ANB-020] [scope=crates/nako-server,crates/nako-official-addon-catalog,F:\SourceCodes\Rust\nako-official-addons]
  Goal: Prove Nako can register the notification bridge manifest and schedule a
  `library.scanned` event delivery to the sidecar through the existing Addon
  Event runtime.
  Validation: focused Nako Addon Event scheduler gate plus official addon smoke
  or fixture test.
  Review: Confirm no new scheduler semantics are hidden in notification bridge
  implementation.
  Evidence: DONE. Nako official catalog now exposes the notification bridge
  descriptor; server tests prove registration, health-check, routing-plan sync,
  scheduled `library.scanned` delivery, redaction-safe delivery attempts, and
  ACK through the existing Addon Event runtime. Official sidecar tests prove the
  ACK path and health resource count contract.
  Handoff: Continue with ANB-040 for first real provider or explicit split.

## M3 — First Provider Or Provider Split

- [x] ANB-040 [owner=planner] [deps=ANB-030] [scope=docs/workstreams/addon-notification-bridge,docs/workstreams/addon-notification-provider-adapters,F:\SourceCodes\Rust\nako-official-addons]
  Goal: Decide whether to implement one narrow provider adapter now or split
  provider fan-out into a follow-on after the ACK proof.
  Validation: design update plus provider-specific gate if implemented.
  Review: Check credential ownership, redaction, retry, and operator
  configuration boundaries.
  Evidence: DONE. Provider breadth is split into
  `docs/workstreams/addon-notification-provider-adapters/` with ANP-010 as the
  first provider selection task. No provider adapter was implemented in this
  lane.
  Handoff: Continue with ANB-050 closeout.

## M4 — Closeout

- [x] ANB-050 [owner=planner] [deps=ANB-040] [scope=docs/workstreams/addon-notification-bridge]
  Goal: Close the notification bridge lane or split remaining provider breadth
  into narrower follow-ons.
  Validation: `cargo fmt --all -- --check`, focused host/official-addon gates,
  `git diff --check`, and `WORKSTREAM.json` parse.
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: DONE. Final host, DB, official sidecar, fmt, JSON, and diff hygiene
  gates passed. Provider breadth is split into
  `docs/workstreams/addon-notification-provider-adapters/`.
  Handoff: DONE. This lane is closed.
