# Admin Web V2 Generated Artifact Review Actions

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 can list Generated Artifact proposals at
`/automation/generated-artifacts`, but operators cannot yet inspect a safe
review plan or make an explicit accept/reject decision from the V2 console.

The backend and generated Admin API contract already expose review-plan and
review routes. The missing work is the administration UX: one proposal, one
decision, one review plan, one confirmation, and one redacted result.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-v2-automation-generated-artifacts-route/`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md`
- `docs/api/HTTP_API.md`

## Problem

Generated Artifact proposals are currently visible only as safe queue rows.
The route does not expose review plans, confirmation, mutation results, or
post-action audit context. Adding buttons directly to the queue would be too
shallow because review actions can change metadata authority or enqueue
follow-on work.

## Target State

When this lane closes:

- Operators can choose one Generated Artifact proposal from the V2 route.
- Admin Web can request and render a safe review plan for `accept` or `reject`.
- The UI requires explicit confirmation before POSTing a review decision.
- The result view shows only redacted action/result summaries, job IDs,
  Generated Artifact IDs, target IDs, payload shape/confidence/fingerprints,
  readiness, and safe error/status text.
- Prompt bodies, payload bodies, provider raw responses, Source Locators, local
  paths, artifact storage handles, tokens, and credentials are never rendered.
- Browser smoke covers the proposal list and one review/confirmation path.

## In Scope

- Route/API readiness audit for generated review-plan and review routes.
- Explicit `AdminApiClient` methods for review plan and review commands.
- `AdminDataSource` methods for route-local review plan/result behavior.
- Route-owned proposal review UI reachable from
  `/automation/generated-artifacts`.
- Confirmation UX for accept/reject.
- Focused route, client, data-source, fallback, mutation, and redaction tests.
- Browser smoke and closeout evidence.

## Out Of Scope

- Autonomous apply or bulk review.
- Raw prompt/payload/provider body inspection.
- Catalog repair, Provider Mapping accept, Artwork selection, NFO writes, or
  arbitrary metadata editing.
- Backend review semantics changes unless the readiness audit finds a blocker.
- Full-site i18n expansion.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Generated Admin API contract already includes review-plan and review routes. | High | `NAKO_ADMIN_ROUTES.generatedArtifactReviewPlan` and `generatedArtifactReview` exist. | GAR-020 must split contract generation/backend work before UI implementation. |
| Review plan/result DTOs are already redaction-safe enough for summary UI. | Medium | Existing server tests cover Generated Artifact review responses and redaction. | Add a backend/API DTO hardening task before rendering review details. |
| One-proposal review is the correct first mutation slice. | High | MBG-050 explicitly excludes bulk review and recommends one proposal first. | Keep queue-only route read-only until a narrower route can be proven. |

## Architecture Direction

- `App.tsx` owns route wiring and search/path ownership.
- `adminApi/client.ts` owns generated Admin API route calls.
- `adminApi/dataSource.ts` owns live/mock fallback and safe route summaries.
- `features/automation/` owns review UI and confirmation states.
- Shared UI components stay generic and receive already-redacted display data.

## Closeout Condition

This lane can close when:

- review-plan and review route readiness is accepted or split with precise
  blockers;
- V2 exposes a guarded one-proposal review workflow;
- focused Admin Web tests cover route rendering, data-source/client calls,
  confirmation behavior, fallback, and unsafe text exclusions;
- final Admin Web gates, `git diff --check`, and browser smoke pass;
- remaining bulk review, catalog repair, and cross-domain actions are split.
