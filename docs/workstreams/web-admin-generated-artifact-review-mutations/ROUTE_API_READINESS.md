# Web Admin Generated Artifact Review Mutations - Route/API Readiness

Status: Closed
Last updated: 2026-05-29

## Frontend Routes

| Frontend route | Search state | Purpose |
| --- | --- | --- |
| `/admin/automation/generated-artifacts` | `limit`, `offset` | Proposal queue and guarded action entry points. |
| `/admin/automation/generated-artifacts/review` | `artifact_id`, `decision` | One-artifact review-plan preview and confirmation. |

`decision` accepts only `accept` or `reject`. Missing or invalid decisions
normalize to `accept`; missing `artifact_id` renders a route-owned empty state
with back navigation.

## Admin API Inventory

| Operation | Method | Path | Body | Response |
| --- | --- | --- | --- | --- |
| Proposal list | `GET` | `/admin/v1/automation/generated-artifacts/proposals` | query `limit`, `offset` | `AdminGeneratedArtifactProposalListResponse` |
| Review plan | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan` | `{ decision: "accept" | "reject" }` | `AdminGeneratedArtifactReviewPlanResponse` |
| Review | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review` | `{ decision: "accept" | "reject" }` | `AdminGeneratedArtifactReviewResponse` |

The review-plan route is intentionally modeled as `POST` because the backend
requires a selected decision before it can compute the plan.

## Required Display Facts

Before confirming review, the route must show:

- `artifact_id`
- `decision`
- `status`
- `action`
- `reasons`
- `capability`
- `kind`
- target kind and stable IDs
- payload shape, fingerprint, byte/count facts, textual/explanation booleans,
  confidence
- readiness status, actionable flag, and reasons
- boundary flags:
  `accepted_into_canonical_metadata`,
  `writes_sidecar`,
  `writes_library_files`,
  `applies_immediately`,
  `requires_metadata_authority_apply`

After review, the route must show:

- `artifact_status`
- `accepted_at`
- `idempotent_replay`
- the reviewed decision

## Redaction Rules

The frontend read model and rendered UI must not expose:

- raw prompt text
- raw generated payload body
- provider raw response
- local path
- Source Locator
- credential
- bearer token
- secret
- artifact storage handle

## Readiness Result

Closed at `WGAR-040`: the new `web/` Admin route uses the recorded method,
path, body, and result shape. Tests assert `POST review-plan` and `POST review`
serialization with Bearer auth.
