# AI Assisted Library Ops — Handoff

Status: Complete
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

AILO-010, AILO-020, AILO-030, AILO-040, and AILO-050 are complete. The lane is
scoped to Generated Artifact proposal queues, redacted Admin diagnostics, and
explicit acceptance planning, not a local model runtime or autonomous writes.

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

AILO-030 added Admin-only Generated Artifact proposal diagnostics:

- `taru-api::admin` owns redacted Admin DTOs for proposal list responses.
- `taru-server::http::admin` exposes
  `/admin/v1/automation/generated-artifacts/proposals`.
- Admin TypeScript contract generation includes `generatedArtifactProposals`
  and remains synchronized with `apps/admin-web/src/adminApi/generated`.
- Admin Web client/data-source/mocks render proposal summaries without raw
  prompt, raw generated payload, provider secret, Source Locator, local path, or
  Public Client protocol exposure.
- Server system tests prove the route is read-only and does not mutate
  canonical metadata.

AILO-040 added explicit Generated Artifact review planning:

- `metadata_cleanup` / `metadata_suggestion` accept plans stage a metadata
  authority review boundary rather than applying Canonical Metadata directly.
- Accepting a ready metadata-cleanup proposal records the Automation Artifact as
  accepted and is idempotent on replay.
- Rejecting a proposal records a no-mutation rejection and is allowed even when
  the proposal is stale.
- Stale or unsupported proposals cannot be accepted.
- Admin review-plan/review routes expose boundary/reason/action facts without
  raw prompt, raw generated payload, provider secret, Source Locator, local
  path, source fingerprint, or Public Client protocol exposure.

AILO-050 is complete. Final closeout evidence is recorded, this workstream is
marked complete, and the remaining work is split into follow-ons rather than
hidden in the AI lane.

## Closeout State

- Task ID: AILO-050
- Status: DONE
- Final scope:
  - `docs/workstreams/ai-assisted-library-ops`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Review result: no blocking findings. The target state is met, and concrete
  provider adapters, local model runtime, embeddings/vector search, Addon
  distribution, downloader protocol, Public Client display, and deeper metadata
  authority apply work remain split follow-ons.

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

- None.

## Next Recommended Action

Return to `post-rpd-product-hardening`.

Recommended next mainline lane: open `addon-runtime-and-distribution`. The core
side-effect boundaries are now stable enough for Addon packaging/distribution
planning, but first slice should stay outside Native Plugin ABI and avoid
direct filesystem/library writes.
