# Addon Manager Lifecycle Automation - TODO

Status: Completed
Last updated: 2026-05-23

## M0 - Lifecycle Boundary Freeze

- [x] AMG-010 [owner=planner] [deps=none] [scope=docs/workstreams/addon-manager-lifecycle-automation,docs/adr,docs/workstreams]
  Goal: Freeze the Addon Manager problem, target state, non-goals, and first
  manager-owned registry/plan slice without folding marketplace, package
  signing, provider breadth, or direct process/container supervision into the
  first implementation step.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json,
  and HANDOFF.md agree on the selected slice and split follow-ons.
  Review: Confirm the manager boundary does not collapse the existing
  sidecar/distribution split, require Docker socket authority, or import a
  native plugin ABI.
  Evidence: docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md;
  docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md
  Handoff: First executable task is AMG-020.

## M1 - Manager Discovery And Source Shape

- [x] AMG-020 [owner=codex] [deps=AMG-010] [scope=docs,guides,crates/nako-server,crates/nako-api]
  Goal: Define the first managed addon source shape and the minimal registry /
  permission / token plan surface for a user-owned addon lifecycle slot.
  Validation: Addon Manager docs and API shapes are explicit enough that the
  first install/update/remove plan slice can be implemented without revisiting
  the boundary split.
  Review: Keep package signing, marketplace hosting, provider breadth, and
  process/container supervision out of the manager bootstrap slice.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Continue with the first operator-confirmed install/update/remove
  plan slice only after the source shape is stable.

## M2 - First Lifecycle Slot

- [x] AMG-030 [owner=codex] [deps=AMG-020] [scope=crates/nako-server,crates/nako-db,crates/nako-api,docs]
  Goal: Implement the smallest Nako-owned addon lifecycle plan slot with
  explicit operator confirmation for install/update/remove and visible health
  or install-guide state.
  Validation: focused server tests plus a repeatable manager smoke with one
  official addon source.
  Review: Confirm the manager does not absorb package signing, marketplace
  policy, or process/container supervision.
  Evidence: EVIDENCE_AND_GATES.md.
  Handoff: Keep rollback and update-policy handling bounded to the plan layer.

## M3 - Closeout Or Split

- [x] AMG-060 [owner=planner] [deps=AMG-030] [scope=docs/workstreams/addon-manager-lifecycle-automation]
  Goal: Close the lane or split marketplace/package-signing/provider-breadth
  and process-supervision follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. The first manager-owned registry/plan slot is complete.
  Marketplace, package signing, provider breadth, rollback/update policy, and
  process/container supervision are follow-on lanes.
