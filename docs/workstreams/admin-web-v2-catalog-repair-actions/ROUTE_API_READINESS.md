# Admin Web V2 Catalog Repair Actions - Route/API Readiness

Status: Accepted With Backend/API Contract Work
Last updated: 2026-05-25
Task: CRA-020

## Readiness Claim

Provider Mapping accept/reject is the accepted first Catalog Governance repair
action, but Admin Web is not ready to implement UI controls yet.

The existing read-only queue is safe and useful as an entry point. It does not
expose raw Local Inference evidence, Source Locators, provider raw bodies, NFO
paths, tokens, or secrets. The missing piece is a redaction-safe item repair
context, review-plan route, and confirmed mutation route for one Media Item and
one Provider Mapping.

CRA-030 must add backend/Admin API contract work before CRA-050 can add any
repair controls to Admin Web.

## Existing Route Inventory

| Route or capability | Current surface | Readiness | CRA decision |
| --- | --- | --- | --- |
| Governance queue | `GET /admin/v1/catalog/governance/items` | Ready. Lists unknown/low-confidence items and mapping/duplicate counts. | Reuse as the `/catalog/governance` entry point. |
| Generated Admin contract | `NAKO_ADMIN_ROUTES.catalogGovernanceItems`, `AdminCatalogGovernanceItemListResponse` | Ready for queue only. | Keep as read-only until CRA-030 adds new route constants and DTOs. |
| Admin Web page | `/catalog/governance` | Ready as a filtered queue. | Add only navigation to repair context after review-plan routes exist. |
| Item repair detail | Not present. | Missing. | CRA-030 must add a redaction-safe detail/readiness route. |
| Provider Mapping review plan | Not present. | Missing. | CRA-030 must add before any mutation UI. |
| Provider Mapping mutation | Not present. | Missing. | CRA-040 must add after review-plan semantics are stable. |

## Backend Capability Inventory

Ready concepts:

- `ProviderMappingStatus` already supports `candidate`, `accepted`, and
  `rejected`.
- `ProviderMappingRepository` can list mappings for one Media Item and upsert a
  mapping.
- `get_provider_subject` can resolve safe Provider Subject display facts for a
  mapping.
- `CatalogGovernanceRepository::list_catalog_governance_items` already joins
  Local Inference, Provider Mapping counts, duplicate relationship counts, and
  representative source display facts.
- `AdminCatalogGovernanceItem::from_record` redacts raw
  `LocalInferenceEvidence.evidence_value` while preserving typed summary facts
  such as confidence, inferred kind/title/year, evidence source, and evidence
  presence.

Gaps CRA-030/CRA-040 should address:

- There is no item-scoped catalog repair detail DTO that includes Provider
  Mapping candidate summaries.
- There is no mapping-ID lookup or status-update repository method. CRA-030 can
  either add an item-scoped lookup/update method or use the existing
  item-scoped list plus upsert, but the chosen API must reject mismatched
  `item_id`/`mapping_id` pairs.
- There is no Admin API review-plan DTO describing current status, target
  status, idempotent state, side effects, and redaction boundary.
- There is no confirmed Admin mutation route with idempotent result semantics.
- The generated Admin Web contract has no route constants, request type, or
  response types for catalog repair detail, review-plan, or mutation.

## Accepted First Action

The first repair action is one Provider Mapping review:

```text
decision = accept | reject
scope = one Media Item ID + one Provider Mapping ID
```

Expected semantics:

- `accept` changes one mapping to `accepted`.
- `reject` changes one mapping to `rejected`.
- repeating the same decision is an idempotent replay.
- changing from `accepted` to `rejected`, or from `rejected` to `accepted`, is
  allowed only through the same explicit review-plan and confirmation flow.
- the action updates Provider Mapping status only.
- the action must not mutate Canonical Metadata, Provider Subjects, Local
  Inference Evidence, Source Duplicate Relationships, hierarchy, NFO sidecars,
  Library Files, artwork, playback state, or Public Client API state.

Rationale:

- Provider Mapping has an existing bounded status vocabulary.
- It can be scoped to stable IDs and safe Provider Subject display facts.
- It directly addresses the `missing_accepted_provider_mapping` governance
  issue without requiring arbitrary metadata editing.
- It avoids the higher-risk repair classes that need stronger domain semantics.

## Proposed Admin API Shape For CRA-030/CRA-040

CRA-030 should add a redaction-safe detail/readiness route and a review-plan
route. CRA-040 should add the mutation route. The exact suffixes can change
during implementation, but the accepted shape is item-scoped:

```text
GET  /admin/v1/catalog/governance/items/{item_id}
POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan
POST /admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review
```

The review-plan and review request body should be:

```json
{
  "decision": "accept"
}
```

Required safe response fields:

- Admin/Public API version strings where current Admin DTO style expects them.
- Media Item ID, Media Library ID, title, kind, and current governance issues.
- Provider Mapping ID, current status, target status, confidence, source, and
  Provider Subject safe summary.
- Provider Subject safe summary: subject ID, provider, subject kind,
  subject key, optional title, release year, and locale.
- Readiness status, actionable boolean, and stable reason codes.
- Boundary booleans: updates Provider Mapping status, does not update
  Canonical Metadata, does not write NFO, does not write Library Files, does
  not apply immediately to metadata fields.
- Mutation result: decision, changed boolean, idempotent replay boolean,
  previous status, current status, item ID, mapping ID, and safe audit reason
  codes.

## Deferred Repair Classes

These remain split until their own review-plan and redaction contracts are
explicit:

- unknown-item classification;
- Local Inference evidence inspection or apply;
- hierarchy repair, split, merge, or reparenting;
- duplicate-source merge/split;
- rematch/provider search;
- Provider Subject creation/editing;
- NFO import/export/write actions;
- arbitrary Canonical Metadata editing.

## Safe Projection Rules

Admin API and Admin Web may render:

- Media Item, Media Library, Media Source, Provider Mapping, and Provider
  Subject IDs;
- display titles, kinds, release year/date, counts, statuses, confidence
  values, reason codes, and first-party route paths;
- Provider Subject key values when they are typed external IDs, not request
  URLs or provider response payloads;
- booleans that summarize whether evidence exists, whether a mapping changes,
  and whether the request is idempotent.

Admin API and Admin Web must not render:

- raw Local Inference `evidence_value`;
- Source Locators;
- local filesystem paths;
- NFO sidecar paths, raw XML, or file contents;
- provider raw response bodies;
- provider request URLs or query strings;
- token, secret, credential, or authorization-header values;
- arbitrary metadata payloads not summarized by a typed redacted DTO.

## Generated Contract Work

CRA-030 should add generated contract coverage before any UI route depends on
the new API:

- route constants for catalog repair detail and Provider Mapping review-plan;
- request type for `decision: "accept" | "reject"`;
- response types for repair detail and review-plan.

CRA-040 should extend the same generated contract with the confirmed mutation
response type and `AdminApiClient`/`AdminDataSource` wrappers.

## HTTP API Documentation

`docs/api/HTTP_API.md` should not list the proposed repair routes as existing
routes during CRA-020. CRA-030/CRA-040 must update the Admin route inventory
and catalog governance section when the routes are implemented and tested.

## Handoff

Continue with CRA-030 backend/API repair detail and Provider Mapping
review-plan work. Do not add Admin Web repair buttons until generated Admin
contract coverage exists for the detail/review-plan route and redaction tests
prove the responses exclude unsafe evidence.
