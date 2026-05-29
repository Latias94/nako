# Web Admin Generated Artifacts Automation - Route/API Readiness

Status: Active
Last updated: 2026-05-28

## Route Target

| Frontend route | Surface | Status | Notes |
| --- | --- | --- | --- |
| `/admin/automation/generated-artifacts` | Admin | Implemented | New `web/` route with route-owned pagination and read-only proposal diagnostics. |

## Generated Admin Contracts

The committed generated Admin contract already includes the first proposal route
and review boundaries:

| Contract | Generated symbol | Path or shape | First use |
| --- | --- | --- | --- |
| Proposal list route | `ADMIN_API_ROUTES.generatedArtifactProposals` | `/admin/v1/automation/generated-artifacts/proposals` | Live proposal list data source. |
| Proposal query | `AdminGeneratedArtifactProposalsQuery` | `limit`, `offset` | Route pagination mapping. |
| Proposal list response | `AdminGeneratedArtifactProposalListResponse` | `admin_api_version`, `public_api_version`, `proposals`, `page` | Data-source contract test and read model mapping. |
| Proposal diagnostic | `AdminGeneratedArtifactProposal` | target, provenance, payload summary, readiness, status, timestamps | UI table/cards. |
| Review-plan route | `ADMIN_API_ROUTES.generatedArtifactReviewPlan` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan` | Guarded detail/action planning. |
| Review request | `AdminGeneratedArtifactReviewRequest` | `decision: "accept" | "reject"` | Confirmation payload. |
| Review-plan response | `AdminGeneratedArtifactReviewPlanResponse` | `plan: AdminGeneratedArtifactAcceptancePlan` | Boundary display before mutation. |
| Review route | `ADMIN_API_ROUTES.generatedArtifactReview` | `/admin/v1/automation/generated-artifacts/{artifact_id}/review` | Deferred until mutation guard requirements pass. |
| Review response | `AdminGeneratedArtifactReviewResponse` | decision, artifact status, replay flag, acceptance plan | Guarded accept/reject result. |

## First Read-Only Mapping

`WAGA-020` verified and implemented the read-model mapping:

| UI field | Contract field |
| --- | --- |
| Proposal id | `id` |
| Kind/capability/status | `kind`, `capability`, `status` |
| Target | `target.kind`, `target.library_id`, `target.item_id`, `target.source_id` |
| Provider/job provenance | `provenance.provider_id`, `provider_name`, `job_id`, `capability`, `attempt_count` |
| Safety fingerprints | `idempotency_key_fingerprint`, `prompt_fingerprint`, `payload.payload_fingerprint` |
| Payload summary | `payload.valid_json`, `shape`, `payload_bytes`, counts, textual/explanation booleans, `confidence_milli` |
| Readiness | `readiness.status`, `actionable`, `reasons` |
| Timestamps | `created_at`, `updated_at`, `accepted_at`, `provenance.artifact_created_at` |

## WAGA-020 Audit Result

`WAGA-020` added:

- `AdminApiClient.getGeneratedArtifactProposals(query)`;
- `createAdminReadModelsDataSource().loadGeneratedArtifacts(query)`;
- `ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE`;
- `AdminGeneratedArtifactsReadModel`;
- data-source contract coverage for fixture fallback, live query
  serialization, Bearer authorization, DTO-to-read-model mapping, and redaction
  of non-contract raw fields.

The read model intentionally exposes only summarized proposal facts. It does not
carry raw prompts, raw generated payload bodies, provider raw responses, local
paths, Source Locators, credentials, storage handles, or secrets.

## WAGA-030 Route Evidence

WAGA-030 added `web/src/features/admin/admin-generated-artifacts.tsx`, Admin
navigation, router state for `limit` and `offset`, route contract coverage,
route-state coverage, and live rendering coverage.

The route is read-only. It renders proposal id, kind, capability, status,
target ids, provider/job provenance, payload summary, readiness, safe
fingerprints, timestamps, and fixture/live source state. Tests assert that raw
prompt text, raw generated payload bodies, provider raw responses, local paths,
Source Locators, credentials, and bearer tokens do not render.

Review-plan and accept/reject actions remain deferred pending WAGA-040.

## Review Guard Requirements

Any frontend review action must show `AdminGeneratedArtifactAcceptancePlan`
before calling `generatedArtifactReview`. The UI must display:

- `decision`, `status`, `action`, `reasons`, `capability`, and `kind`;
- readiness status and reasons;
- target summary and payload summary;
- boundary flags:
  `accepted_into_canonical_metadata`, `writes_sidecar`,
  `writes_library_files`, `applies_immediately`, and
  `requires_metadata_authority_apply`;
- idempotent replay result after review.

If these cannot be made clear in this lane, accept/reject remains a follow-on.

## Required Gates

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Closeout should also include browser smoke for desktop and mobile viewports.
