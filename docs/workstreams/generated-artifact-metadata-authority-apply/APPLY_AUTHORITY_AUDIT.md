# Generated Artifact Metadata Authority Apply - Audit

Status: Active
Last updated: 2026-05-29

## Existing Admin API

| Operation | Method | Path | Body | Response |
| --- | --- | --- | --- | --- |
| Proposal list | `GET` | `/admin/v1/automation/generated-artifacts/proposals` | query `limit`, `offset` | `AdminGeneratedArtifactProposalListResponse` |
| Review plan | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan` | `{ decision: "accept" | "reject" }` | `AdminGeneratedArtifactReviewPlanResponse` |
| Review | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review` | `{ decision: "accept" | "reject" }` | `AdminGeneratedArtifactReviewResponse` |
| Metadata apply plan | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan` | empty | `AdminGeneratedArtifactMetadataApplyPlanResponse` |

The review routes are correct but intentionally incomplete for apply. They
answer "can this proposal be accepted or rejected?" They do not answer "which
Canonical Metadata fields would change?"

## Current Code Findings

- `GeneratedArtifactAcceptanceBoundary::deferred_metadata_authority()` sets
  `accepted_into_canonical_metadata`, `writes_sidecar`, `writes_library_files`,
  and `applies_immediately` to false, while setting
  `requires_metadata_authority_apply` to true.
- `AutomationApp::review_generated_artifact` only transitions artifact status
  through `set_automation_artifact_status`; it does not call
  `MetadataApplication`.
- `automation_app_reviews_metadata_cleanup_proposal_without_canonical_mutation`
  proves accepted metadata cleanup proposals leave `MediaItem.metadata`
  unchanged and redact generated private content from the response.
- `nako-automation` rejects provider outcomes that claim
  `accepted_into_canonical_metadata`, preserving the external-automation
  boundary.
- `MetadataApplication` is reusable inside `nako-server` for final apply, but
  it currently accepts an already-normalized `CanonicalMetadata` incoming value
  and emits a coarse changed/no-changed report. Generated Artifact apply needs
  a pre-apply plan shape before this commit step.
- NFO sidecar import apply is a useful lifecycle reference: accept preview,
  apply with idempotent replay, reject stale content before mutation, commit
  audit/outcome, and preserve field locks.

## Missing Contract

The next backend contract should introduce two concepts:

- Apply plan: redacted, field-level preview for an accepted metadata Generated
  Artifact. It should include status, reasons, target identity, payload summary,
  candidate/current values where safe, skipped fields, lock effects, stale
  target reasons, and an executable flag.
- Apply result: idempotent and audited confirmation that revalidates the plan
  immediately before mutation, delegates to host metadata application, and
  reports changed/skipped/blocked field counts without echoing raw generated
  payload.

## Initial API Direction

GAMA-020 fixed the read-only apply-plan route:

| Operation | Method | Path | Body | Response |
| --- | --- | --- | --- | --- |
| Metadata apply plan | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan` | empty | `AdminGeneratedArtifactMetadataApplyPlanResponse` |

The apply-plan response exposes status, executable flag, reasons, target,
payload summary, field-level actions, redacted current/incoming value summaries,
and apply/skipped/noop field counts.

The apply route remains a `GAMA-030/GAMA-050` concern:

| Operation | Method | Path | Body | Response |
| --- | --- | --- | --- | --- |
| Metadata apply | `POST` | `/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply` | `idempotency_key`, optional accepted field set | `AdminGeneratedArtifactMetadataApplyResponse` |

The important invariant is that review and metadata apply stay separate route
operations.

## Redaction Rules

Apply-plan and apply responses must not expose:

- raw `artifact_json`
- raw prompt text
- provider raw response
- Source Locator
- local path
- storage handle
- credential, bearer token, or secret
- free-form generated explanation unless it is explicitly transformed into a
  bounded diagnostic enum or redacted summary

## First Slice Decision

GAMA-020 shipped a read-only apply plan for accepted
`AutomationArtifactKind::MetadataSuggestion` artifacts with an item target. It
computes field actions from the current item, locks, library metadata refresh
mode, and supported generated metadata patch fields without touching Canonical
Metadata.
