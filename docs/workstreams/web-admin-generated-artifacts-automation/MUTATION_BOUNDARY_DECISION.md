# Web Admin Generated Artifacts Automation - Mutation Boundary Decision

Status: Deferred to follow-on
Decided: 2026-05-29
Task: WAGA-040

## Decision

Do not implement review-plan or accept/reject mutation controls in this lane.

`/admin/automation/generated-artifacts` remains a read-only Admin route. Review
actions must split into a focused follow-on lane, tentatively named
`web-admin-generated-artifact-review-mutations`, before any UI button, detail
route, or data-source mutation method is added.

## Contract Inventory

The generated Admin API already exposes:

```text
GET /admin/v1/automation/generated-artifacts/{artifact_id}/review-plan
POST /admin/v1/automation/generated-artifacts/{artifact_id}/review
```

Review request:

```text
decision: "accept" | "reject"
```

Review plan/response includes:

```text
artifact_id
decision
status
action
reasons
capability
kind
target
payload summary
readiness
boundary.accepted_into_canonical_metadata
boundary.writes_sidecar
boundary.writes_library_files
boundary.applies_immediately
boundary.requires_metadata_authority_apply
idempotent_replay
artifact_status
accepted_at
```

## Why It Splits

- The current lane's target route is a read-only proposal diagnostic page.
- Accept/reject changes Generated Artifact status and participates in an
  Acceptance Workflow.
- Review controls need a dedicated route or modal state model before shipping.
- Operators must see boundary flags before confirming a review action.
- Repeated review submissions need explicit idempotent replay behavior.
- Errors and partial readiness must be rendered without implying autonomous
  canonical metadata, sidecar, or library-file writes.
- Cache invalidation after review must be tested against the proposal list and
  review result surfaces.

## Follow-On Requirements

A future guarded mutation lane must define and test:

- Review route shape, URL state, and back navigation.
- Permission and readiness disabled states.
- Confirmation copy for accept and reject.
- Display of review-plan boundary flags before mutation.
- Idempotent replay result handling.
- Result and error rendering after `generatedArtifactReview`.
- Query invalidation and proposal list refresh behavior.
- Data-source contract tests for request serialization and response mapping.
- Route tests that prove no raw prompt, generated payload body, provider raw
  response, local path, Source Locator, credential, bearer token, secret, or
  storage handle is rendered.

## Current Lane Scope

WAGA closes as the read-only Generated Artifacts proposal route lane. It may
link to the future review mutation follow-on, but it must not add review
mutation controls under WAGA.
