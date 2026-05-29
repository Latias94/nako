# Web Admin Generated Artifacts Automation - Design

Status: Active
Last updated: 2026-05-28

## Problem

Nako has a completed Generated Artifact backend boundary and a closed Admin Web
V2 read-only route, but the new `web/` Admin shell cannot inspect AI-assisted
or automation-generated proposals. WBBP removed the fake Media AI/Automation
surfaces, so those workflows need to reenter through Admin-only diagnostics and
explicit acceptance planning.

The risk is overpromising automation. A free-form assistant panel or Media
automation sidebar would imply autonomous model writes before the frontend has
review-plan semantics, idempotency display, redaction rules, and mutation
confirmation. The correct first UI is an Admin route over generated artifact
proposal contracts.

## Target State

When this lane closes:

- `/admin/automation/generated-artifacts` is a real route in the new `web/`
  shell.
- The route owns normalized pagination search params for `limit` and `offset`.
- `web/src/api/admin` exposes a generated artifact read model backed by
  `AdminGeneratedArtifactProposalListResponse`.
- The page renders proposal kind, capability, target, provider/job provenance,
  payload summary, readiness, status, timestamps, and fingerprints.
- Raw prompt text, raw generated payload bodies, provider raw responses, local
  paths, Source Locators, credentials, and secrets are never rendered.
- Review-plan and accept/reject actions are either guarded by explicit UX,
  idempotency, boundary, and confirmation tests, or split into a follow-on.
- Route contracts, route-state tests, data-source contract tests, TypeScript
  check, bundle budget, and browser smoke evidence pass.

## Scope

In scope:

- `web/src/api/admin/*` generated artifact data-source/read-model additions.
- `web/src/features/admin/*` Admin navigation, page, and display-only workflow.
- `web/src/shell/nako-router.tsx` route and pagination normalization.
- Tests under `web/src/test` for data-source contracts, route rendering,
  pagination state, redaction-sensitive rendering, and mutation guard behavior.
- Documentation of review-plan and review mutation readiness.

Out of scope:

- Backend/Admin API route implementation.
- Provider-specific model adapters, local LLM runtime, embeddings/vector search,
  or Addon distribution.
- Public Client API or Media assistant UI.
- Direct canonical metadata writes, sidecar writes, or library-file mutations.
- Reusing `apps/admin-web` source code directly; the old lane is prior art,
  while `web/` owns its own components and tests.

## Architecture Direction

Keep generated artifacts as Admin-reviewed proposals:

- Generated Admin contracts are the source of truth for routes and DTOs.
- `web/src/api/admin` maps proposal/review DTOs into UI read models and owns
  fixture/live fallback behavior.
- `web/src/shell/nako-router.tsx` owns route search validation and
  serialization.
- `web/src/features/admin` renders proposal diagnostics inside Admin
  operations, not as Media client navigation.
- Accept/reject UI must show the returned boundary fields before enabling a
  destructive-looking action, even when the backend boundary proves no direct
  canonical writes occur.

## Prior Art

- `docs/workstreams/ai-assisted-library-ops`
- `docs/workstreams/addons-automation`
- `docs/workstreams/admin-web-v2-automation-generated-artifacts-route`
- `docs/adr/0004-ai-as-external-automation-first.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`

## Risk Plan

- Contract drift: start with `WAGA-020` and record generated Admin proposal,
  review-plan, and review DTOs before implementation.
- Unsafe rendering: tests must assert that raw prompts, payload bodies, provider
  data, locators, paths, and secrets stay absent.
- Mutation confusion: keep the first route read-only unless review-plan and
  review actions have explicit confirmation and boundary display.
- Bundle growth: require `npm --prefix web run build:budget` for route slices.
