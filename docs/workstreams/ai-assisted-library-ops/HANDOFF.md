# AI Assisted Library Ops — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is open as the next mainline child of `post-rpd-product-hardening`
after Network Access Boundary closed.

Prerequisites are complete:

- `metadata-provider-breadth` made provider capabilities, matching ambiguity,
  and cross-provider conflict review explainable.
- `nfo-link-authority`, `link-apply-and-import-promotion`, and
  `nfo-sidecar-promotion-apply` made local metadata, NFO, link, and library file
  writes accepted/audited rather than implicit.
- `downloads-watch-folder-intake` proved acquisition candidates and Managed
  Import handoff without direct library writes.
- `network-access-boundary` proved remote access policy/readiness, trusted
  proxy/header behavior, origin enforcement, and Admin-only network diagnostics
  without built-in NAT traversal runtime.
- `addons-automation` and `taru-automation` already provide an external
  automation provider/job/artifact foundation, but not the product-specific
  Generated Artifact review/acceptance semantics this lane needs.

AILO-010 and AILO-020 are complete. The lane is scoped to Generated Artifact
proposal queues, redacted Admin diagnostics, and explicit acceptance planning,
not a local model runtime or autonomous writes.

AILO-020 added backend Generated Artifact proposal/readiness semantics over
existing Automation Artifacts:

- `taru-core` now has proposal target, provenance, payload summary, and
  readiness vocabulary.
- `taru-db` can list Generated Artifact proposals from existing automation
  artifacts without adding a parallel AI artifact store.
- Proposal readiness detects invalid payloads, missing/mismatched targets,
  missing provider/job records, accepted/rejected artifacts, and mismatched job
  input.
- Proposal summaries expose fingerprints/counts/booleans/confidence, not raw
  prompt text, raw generated text, provider secrets, Source Locators, local
  paths, or source fingerprints.
- `taru-automation` and `taru-server::app` tests prove proposal listing does not
  mutate canonical metadata.

## Active Task

- Task ID: AILO-030
- Owner: codex
- Files:
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-api/src/admin_contract.rs`
  - `crates/taru-server/src/http/admin.rs`
  - `apps/admin-web/src/adminApi`
- Validation:
  - `cargo nextest run -p taru-api admin_contract --no-fail-fast`
  - `cargo nextest run -p taru-server http::tests::system --no-fail-fast`
  - `npm run check` from `apps/admin-web`
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: READY
- Review: expose Admin-only proposal diagnostics from the AILO-020 read model
  without raw prompt/output/path/source/provider-secret leakage and without
  Public Client API or `taru-client-protocol` mutation.

## Decisions Since Opening

- Existing Automation Artifacts are the substrate; do not create a parallel AI
  artifact store unless a concrete contract gap forces it.
- Generated Artifact proposals must be target-scoped, provenance-rich,
  confidence/explanation-aware, and stale-target-checkable.
- Raw prompts, raw generated output, provider raw responses, local paths, and
  Source Locators are unsafe for Admin diagnostics by default.
- Provider-specific OpenAI/local sidecar adapters are follow-ons. The first lane
  proves proposal/acceptance semantics using existing provider abstractions.
- Addon runtime/distribution remains downstream. Addons may later produce
  Generated Artifacts, but they should consume this lane rather than define a
  separate mutation path.

## Blockers

- None for AILO-030.

## Next Recommended Action

Execute AILO-030: add Admin-only Generated Artifact proposal diagnostics and
typed Admin Web support over the AILO-020 proposal read model. Keep acceptance
apply out of scope until AILO-040.
