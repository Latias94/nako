# Web Admin Generated Artifact Review Mutations - Milestones

Status: Closed
Last updated: 2026-05-29

## M0 - Lane Opened

Exit criteria:

- Workstream docs exist.
- Route/API readiness records that review-plan is `POST`.
- Architecture/workstream indexes point to this lane.

## M1 - API And Data-Source Boundary

Exit criteria:

- Admin API client exposes review-plan and review commands.
- Review-plan read model contains only redacted, display-safe facts.
- Review mutation data source returns a domain-specific result.
- Data-source tests assert method/path/body/auth/redaction and fixture
  rejection.

## M2 - Guarded Review Route

Exit criteria:

- Queue rows navigate to the review route.
- Review route owns `artifact_id` and `decision` search state.
- Review-plan facts and boundary flags render before confirmation.
- Confirmation is explicit and disabled when mutation authority is unavailable.
- Live mutation result renders idempotency and status facts.
- Proposal/review queries invalidate after success.

## M3 - Verified Closeout

Exit criteria:

- Full frontend tests, TypeScript check, and bundle budget pass.
- Browser smoke passes desktop and mobile.
- `git diff --check` and `WORKSTREAM.json` JSON validation pass.
- Closeout and handoff are updated.
- One precise Conventional Commit contains only this lane's changes.
