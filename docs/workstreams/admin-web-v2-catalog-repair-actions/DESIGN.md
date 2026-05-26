# Admin Web V2 Catalog Repair Actions

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 can browse Media Items, inspect one Media Item, and list the
Catalog Governance queue for unknown and low-confidence items. Operators still
cannot safely act on that queue. Adding repair buttons directly to table rows
would be unsafe because Provider Mapping, Local Inference, duplicate-source,
hierarchy, and NFO decisions can change Canonical Metadata authority or media
structure.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/api/HTTP_API.md`
- `docs/workstreams/admin-catalog-governance-read-model/`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/CLOSEOUT.md`
- `docs/workstreams/admin-web-v2-generated-artifact-review-actions/CLOSEOUT.md`
- `docs/workstreams/admin-web-v2-item-artwork-selection/CLOSEOUT.md`

## Problem

The current `/catalog/governance` route is intentionally read-only. It shows
unknown kind, weak Local Inference, missing accepted Provider Mapping, and
duplicate relationship counts, but it cannot expose enough safe context for an
operator to make a repair decision. Directly wiring mutation controls would
either under-inform the operator or leak raw provider, path, Source Locator, or
NFO evidence that belongs behind Admin redaction boundaries.

## Target State

When this lane closes:

- `/catalog/governance` has a clear path to one-item repair context.
- Admin API exposes redaction-safe catalog repair readiness/detail data needed
  for the first bounded action.
- The first catalog repair mutation is mediated by a dry-run or review-plan
  response before any state change.
- Admin Web requires explicit prepare and confirm steps before mutation.
- Mutation results render item IDs, Provider Mapping IDs, action, changed or
  idempotent state, and safe audit/result summaries only.
- Raw Local Inference `evidence_value`, Source Locators, local paths, provider
  raw responses, provider request URLs, NFO sidecar paths or XML, tokens,
  credentials, and arbitrary metadata payloads are never rendered.
- Unknown-item classification, hierarchy split/merge, duplicate-source repair,
  rematch/provider search, NFO writes, and arbitrary metadata editing are
  either completed in this lane or explicitly split with evidence.

## In Scope

- Route/API readiness audit for Catalog Governance detail, Provider Mapping
  review, and first repair mutation semantics.
- Redaction-safe Admin DTOs for the first repair context.
- Generated Admin Web contract coverage for any new Admin routes.
- Explicit `AdminApiClient` and `AdminDataSource` methods for first repair
  review-plan and mutation routes.
- Admin Web route or modal reachable from `/catalog/governance`.
- Explicit confirmation UX and visible mutation failure states.
- Focused backend, contract, route, data-source, UI, fallback, mutation, and
  redaction tests.
- Browser smoke and closeout evidence.

## Out Of Scope

- Bulk repair or automatic repair.
- Raw metadata editing forms.
- Provider raw body display, provider credential display, or raw provider
  request diagnostics.
- Raw Local Inference evidence values, Source Locators, local filesystem paths,
  NFO file paths, NFO XML, sidecar contents, tokens, and credentials.
- Generated Artifact review, Managed Artwork selection, item NFO status/actions,
  playback support detail, settings mutation, users/permissions/Library Access,
  and full-site i18n.
- Public Client API, Public Client OpenAPI, SDK, or `nako-client-protocol`
  changes.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The read-only Catalog Governance queue already exists. | High | `GET /admin/v1/catalog/governance/items`, Admin Web `/catalog/governance`, and the M60 read-model workstream are present. | CRA-020 must split read-model work before any repair UI. |
| Provider Mapping is the safest first repair action candidate. | Medium | `ProviderMappingStatus` supports `candidate`, `accepted`, and `rejected`, and repositories can list/upsert mappings. | CRA-020 may select a different first action or split Provider Mapping prerequisites. |
| Admin API lacks repair review-plan and mutation routes for this UI. | High | Current generated Admin Web contract exposes `catalogGovernanceItems` only for this surface. | CRA-030 must add backend/API contract work before UI controls. |
| Local Inference and duplicate-source evidence need extra redaction before operator action. | High | MBG follow-on split says raw evidence values and repair semantics are not safe in the read slice. | Keep those actions split until safe DTOs and dry-run semantics exist. |

## Architecture Direction

- `nako-core` owns repair domain records, IDs, and repository traits.
- `nako-db` owns backend-neutral persistence and query/adaptation for Provider
  Mapping, Local Inference, and duplicate-source facts.
- `nako-api::admin` owns redaction-safe Admin DTOs and generated Admin Web
  contract source.
- `nako-server` owns route composition, auth, query parsing, error mapping,
  and application orchestration.
- `apps/admin-web` owns route state, operator confirmation UI, and safe display
  projections through `AdminApiClient` and `AdminDataSource`.

The first repair flow must look like the Generated Artifact and Item Artwork
lanes: read context, prepare/review, confirm, mutate, render redacted result,
and make failures visible. Mock fallback is allowed for read context only; it
must not fake successful repair mutations.

## Closeout Condition

This lane can close when:

- route/API readiness accepts the first catalog repair action or splits
  blockers;
- backend/Admin API semantics exist for redaction-safe review-plan and mutation;
- Admin Web exposes the workflow behind explicit confirmation;
- focused and full gates, relevant Rust/Admin contract gates, `git diff
  --check`, and browser smoke pass;
- remaining catalog repair breadth is either completed, deferred, or split.

## Closeout

Closed 2026-05-25. This lane shipped the first bounded Catalog Governance
repair workflow: Provider Mapping accept/reject for one Media Item and one
Provider Mapping, mediated by redaction-safe detail, review-plan, explicit
confirmation, idempotent mutation result semantics, safe Admin Web rendering,
focused/full gates, and browser smoke.

Remaining Catalog Governance repair breadth is intentionally split to later
lanes: rematch/provider search, duplicate-source merge/split, hierarchy repair,
NFO writes, arbitrary metadata editing, and broader users/permissions or
Library Access work.
