# Generated Artifact Review Route/API Readiness

Status: Accepted
Last updated: 2026-05-25
Task: GAR-020

This document decides whether Admin Web V2 can implement the first
Generated Artifact review/action workflow on existing Admin API routes.

## Decision

GAR-030 can start without backend work.

The first review workflow should use generated Admin API methods for:

- `POST /admin/v1/automation/generated-artifacts/{artifact_id}/review-plan`
- `POST /admin/v1/automation/generated-artifacts/{artifact_id}/review`

Both routes accept:

```json
{
  "decision": "accept"
}
```

where `decision` is `accept` or `reject`.

## Route Readiness Matrix

| Route or capability | Current surface | Readiness | GAR decision |
| --- | --- | --- | --- |
| Proposal queue | `GET /admin/v1/automation/generated-artifacts/proposals` | Ready and already used by the read-only V2 route. | Reuse as the list entry point and proposal summary source. |
| Review plan | `POST /admin/v1/automation/generated-artifacts/{artifact_id}/review-plan` | Ready in generated Admin API contract and server route. Returns `AdminGeneratedArtifactAcceptancePlan`. | Use before mutation. The UI must show a safe plan and require confirmation before review. |
| Review action | `POST /admin/v1/automation/generated-artifacts/{artifact_id}/review` | Ready in generated Admin API contract and server route. Returns `AdminGeneratedArtifactReviewResponse`. | Use only after explicit confirmation. Do not silently mock successful mutation. |
| Review decision values | `AdminGeneratedArtifactReviewRequest.decision` | Ready. Generated TypeScript type is `"accept" | "reject"`. | Use a route-local segmented control or equivalent explicit choice. |
| Result audit summary | `AdminGeneratedArtifactReviewResponse` | Ready. Includes artifact id, decision, status, accepted timestamp, idempotent replay flag, and plan. | Render redacted result summary after command completes. |
| Raw prompt/payload inspection | Not present in generated Admin DTOs. | Correctly absent. | Do not add raw inspection UI in this lane. |

## Safe Projection Rules

The route data source must project generated DTOs before rendering:

- show proposal/artifact ID, decision, status, action, reasons, target IDs,
  capability, kind, payload shape, confidence, payload size, fingerprints,
  readiness, and boundary booleans;
- show whether the plan applies immediately, writes sidecars, writes library
  files, accepts into canonical metadata, or requires metadata authority apply;
- do not render prompt bodies, payload bodies, raw provider responses, Source
  Locators, local paths, artifact storage handles, resolved credentials,
  bearer tokens, or raw response bodies;
- treat `review-plan` as a safe preview, but treat `review` as a mutation that
  must return a real result or a visible error;
- deterministic mock fallback is acceptable for the plan/read path, but not as
  a fake successful accept/reject mutation.

## UI Shape

GAR-030 should add a review-plan surface reachable from
`/automation/generated-artifacts`, preferably as a route-owned detail path such
as:

```text
/automation/generated-artifacts/$artifactId/review
```

Decision state should be route-local, for example:

```text
?decision=accept
?decision=reject
```

GAR-040 should add the confirmed mutation flow:

1. operator chooses `accept` or `reject`;
2. Admin Web fetches the matching review plan;
3. operator confirms the exact decision;
4. Admin Web posts the review command;
5. UI renders the redacted result and refresh affordance.

## Documentation Note

`docs/api/HTTP_API.md` already documented Generated Artifact public/addon
surfaces but its Admin route inventory did not list the generated Admin
proposal/review routes. GAR-020 updates that route inventory so future Admin
Web work does not look like it is relying on an undocumented route.

## Split Conditions

Split before UI if any of these become true during implementation:

- the review DTO starts exposing raw prompt/payload/provider bodies;
- review semantics begin applying directly to Canonical Metadata or sidecars
  without review-plan boundary flags;
- bulk review is required;
- the UI needs catalog repair, Provider Mapping accept, Artwork selection, NFO
  writes, or arbitrary metadata editing to make the first review workflow useful.
