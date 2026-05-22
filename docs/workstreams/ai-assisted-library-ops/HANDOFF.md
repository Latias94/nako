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

AILO-010 is complete. The lane is scoped to Generated Artifact proposal queues,
redacted Admin diagnostics, and explicit acceptance planning, not a local model
runtime or autonomous writes.

## Active Task

- Task ID: AILO-020
- Owner: codex
- Files:
  - `crates/taru-core/src/automation.rs`
  - `crates/taru-db`
  - `crates/taru-automation`
  - `crates/taru-server/src/app/automation.rs`
- Validation:
  - `cargo nextest run -p taru-db automation --no-fail-fast`
  - `cargo nextest run -p taru-automation --no-fail-fast`
  - focused server automation tests
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Status: READY
- Review: deepen Generated Artifact proposal/readiness semantics without
  canonical metadata, sidecar, Media Source, Managed Import, library file,
  Public Client API, or `taru-client-protocol` mutation.

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

- None for AILO-020.

## Next Recommended Action

Execute AILO-020: define and test stable Generated Artifact proposal/readiness
semantics around existing Automation Artifacts, preserving the non-autonomous
write boundary.
