# Nako Rebrand Baseline — TODO

Status: Active
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

- [x] NARB-010 [owner=planner] [deps=none] [scope=docs/workstreams/nako-rebrand-baseline]
  Goal: Freeze the aggressive no-compatibility rename boundary and evidence
  gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json
  exist and agree.
  Evidence: docs/workstreams/nako-rebrand-baseline/DESIGN.md
  Handoff: This lane replaces incremental legacy name compatibility work.

## M1 — Product And Documentation Identity

- [x] NARB-020 [owner=codex] [deps=NARB-010] [scope=README.md,CONTEXT.md,docs,assets,apps/admin-web]
  Goal: Move user-facing product copy, docs, Admin Web branding, and canonical
  brand asset references to Nako.
  Validation: Admin Web check/test/build where source changes require it; grep
  gate for reviewed old-name residue.
  Review: Review copy for accidental historical contradictions.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Platform launcher icon generation may split if native asset tooling
  is not available.

## M2 — Rust Workspace Namespace

- [x] NARB-030 [owner=codex] [deps=NARB-020] [scope=Cargo.toml,Cargo.lock,crates]
  Goal: Rename workspace crate directories, package names, binaries, imports,
  generated examples, and internal dependency references to Nako.
  Validation: cargo fmt --all -- --check; cargo check --workspace --tests;
  focused cargo nextest gates for affected packages.
  Review: Ensure no compatibility shim crates or old binary aliases remain.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Any unavoidable generated artifact updates must be documented.

## M3 — Clients, SDKs, Deploy, And Addons

- [x] NARB-040 [owner=codex] [deps=NARB-030] [scope=apps/android,sdk,deploy,scripts,nako-official-addons]
  Goal: Rename Android package roots, SDK names, deployment examples, scripts,
  addon protocol references, and official addon local path dependencies.
  Validation: feasible Gradle checks; npm gates; cargo gates; addon cargo
  check/test after path dependency update.
  Review: Confirm operational docs use Nako service/container/env names.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Repository remote rename remains a separate operational action.

## M4 — Residual Sweep And Closeout

- [x] NARB-050 [owner=codex] [deps=NARB-040] [scope=repo]
  Goal: Run final old-name grep, classify or remove every residual occurrence,
  and record validation results.
  Validation: old-name grep gate; git diff --check; selected build/test gates.
  Review: review-workstream before closeout if available.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Split any remaining platform asset generation or remote rename tasks.
