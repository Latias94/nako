# Nako Rebrand Baseline — Milestones

Status: Active
Last updated: 2026-05-22

## M0 — Scope Freeze

Exit criteria:

- No-compatibility rename policy is explicit.
- Workstream docs exist and agree.
- Target namespace conventions are recorded.

## M1 — Product Identity

Exit criteria:

- README, CONTEXT, brand docs, Admin Web copy, and active docs use Nako.
- Canonical Nako icon is referenced where the product icon is needed.
- Any old-name residue is either removed or explicitly classified.

## M2 — Rust Workspace Namespace

Exit criteria:

- Crate directories and package names use `nako-*`.
- Rust imports and crate identifiers use `nako_*`.
- CLI binaries and generated examples use Nako names.
- Rust formatting and check gates pass.

## M3 — Clients, SDKs, Deploy, Addons

Exit criteria:

- Android package namespace uses `dev.nako`.
- SDK package examples and generated code refer to Nako.
- Deployment files, config examples, scripts, and docs use Nako service and
  environment names.
- Official addons depend on `nako-addon-protocol`.

## M4 — Closeout

Exit criteria:

- Final grep gate has no unintended old-name residue.
- Validation evidence is recorded.
- Follow-on tasks are split only for external operational work such as remote
  repository renames or optional platform icon regeneration.
