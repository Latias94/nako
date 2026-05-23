# Official Addon E2E Alpha2 - TODO

Status: Active
Last updated: 2026-05-23

## M0 - Scope And Evidence Freeze

- [x] OAE2E-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addon-e2e-alpha2]
  Goal: Freeze the alpha2 E2E problem, target state, non-goals, and evidence
  anchors for the Nako server plus official metadata scraper loop.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/official-addon-e2e-alpha2/DESIGN.md
  Handoff: First executable task is OAE2E-020.

## M1 - Release Surface Smoke

- [x] OAE2E-020 [owner=codex] [deps=OAE2E-010] [scope=scripts,docs/deployment,README.md]
  Goal: Prove the released server artifact/image and published Addon package can
  be started in a documented alpha smoke without path-only workspace
  assumptions.
  Validation: run the documented smoke locally, including server help/config
  preflight and Addon process startup in fixture/default mode.
  Review: review-workstream before accepting completion.
  Evidence: EVIDENCE_AND_GATES.md command transcript summary.
  Handoff: DONE. GHCR server image smoke and direct metadata scraper fixture
  smoke passed without provider secrets. Continue with OAE2E-030.

## M2 - Hosted Addon Loop

- [ ] OAE2E-030 [owner=codex] [deps=OAE2E-020] [scope=crates/nako-server,crates/nako-addon-client,scripts]
  Goal: Prove Nako can register `nako-metadata-scraper`, check hosted health,
  and make one hosted resource call through Nako's Addon runtime.
  Validation: focused server/addon tests or smoke script against local services.
  Review: review-workstream for host/runtime boundary compliance.
  Evidence: EVIDENCE_AND_GATES.md with request/response redaction notes.
  Handoff: Do not add Addon process supervision.

## M3 - Compatibility And Diagnostics

- [ ] OAE2E-040 [owner=codex] [deps=OAE2E-030] [scope=crates/nako-addon-protocol,crates/nako-server,docs/guides]
  Goal: Prove unsupported Addon Protocol versions fail with clear, operator-safe
  diagnostics.
  Validation: cargo nextest run -p nako-addon-protocol protocol_version --no-fail-fast; focused server/addon diagnostic test.
  Review: review-workstream for contract and error-shape quality.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Split multi-version adapter support if this grows beyond diagnostics.

## M4 - Operator Docs And Closeout

- [ ] OAE2E-050 [owner=codex] [deps=OAE2E-030] [scope=README.md,docs/deployment,docs/guides]
  Goal: Teach the official Addon alpha loop in user-facing docs, including the
  responsibility split between Nako and the sidecar.
  Validation: docs are link-complete and commands match the smoke evidence.
  Review: review-workstream for docs/workstream compliance.
  Evidence: README.md and deployment/addon guide links.
  Handoff: Split Addon Manager/marketplace/provider breadth.

- [ ] OAE2E-060 [owner=planner] [deps=OAE2E-040,OAE2E-050] [scope=docs/workstreams/official-addon-e2e-alpha2]
  Goal: Close the lane or split remaining alpha2 follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize remaining risks in HANDOFF.md.
