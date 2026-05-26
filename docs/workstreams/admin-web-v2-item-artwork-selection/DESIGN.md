# Admin Web V2 Item Artwork Selection

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 can inspect Media Items and shows artwork readiness, but
operators cannot yet review item-scoped Managed Artwork candidates/artifacts or
make a guarded Selected Artwork choice from the console.

The backend already exposes Admin item artwork gallery, select, and unpublish
routes. The current Admin Web generated contract does not yet expose those
routes or DTOs, so the first implementation work must make the Admin Web
contract explicit before adding UI controls.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md`
- `docs/workstreams/admin-web-v2-generated-artifact-review-actions/CLOSEOUT.md`
- `docs/api/HTTP_API.md`
- Managed Artwork backend lanes:
  - `docs/workstreams/managed-artwork-gallery-candidate-management/`
  - `docs/workstreams/managed-artwork-public-serving-selection/`
  - `docs/workstreams/selected-artwork-unpublish-delete-policy/`

## Problem

Operators need to compare available artwork for a Media Item, understand which
artifact is currently published as Selected Artwork, select a replacement, and
unpublish a slot when needed. Adding buttons to the item detail readiness row
would be too shallow because artwork selection has publication, public image,
artifact retention, and redaction implications.

## Target State

When this lane closes:

- `/items/:itemId` exposes a clear path to an item-scoped artwork view.
- Admin Web can load and render the item artwork gallery with safe summaries
  for Artwork Candidates, Managed Artwork Artifacts, and current Selected
  Artwork.
- Operators can select/replace one Selected Artwork slot from an eligible
  artifact only after explicit confirmation.
- Operators can unpublish one Selected Artwork slot only after explicit
  confirmation.
- Result views show redacted summaries, changed/idempotent state, safe image
  refs, item IDs, artifact IDs, and image kinds.
- `source_uri`, `storage_uri`, `managed-artwork://`, cache URIs, local paths,
  artifact roots, content hashes, provider URLs/query strings, file contents,
  tokens, and credentials are never rendered.
- Browser smoke covers item detail navigation, artwork gallery, one select
  confirmation path, and one unpublish confirmation path.

## In Scope

- Route/API readiness audit for item artwork gallery, select, and unpublish.
- Generated Admin Web contract coverage for the needed item artwork routes and
  DTOs.
- Explicit `AdminApiClient` methods for item artwork gallery, select, and
  unpublish.
- `AdminDataSource` safe projections for gallery rows and mutation results.
- Route-owned Admin Web UI reachable from `/items/:itemId`.
- Explicit confirmation UX for select/replace and unpublish.
- Focused route, client, data-source, fallback, mutation, and redaction tests.
- Browser smoke and closeout evidence.

## Out Of Scope

- Artwork Candidate accept, ingest processing, ingest requeue, artifact
  lifecycle cleanup, storage-drift repair, remediation, or thumbnail eviction.
- Provider search, scraping, ranking, or automatic artwork choice.
- Direct file upload or sidecar artwork writes.
- Public Client gallery/candidate browsing.
- Catalog repair, Generated Artifact review, NFO writes, metadata editing,
  settings mutation, users/permissions/Library Access, or full-site i18n.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Backend Admin routes for item artwork gallery/select/unpublish already exist. | High | `crates/nako-server/src/http/admin.rs` registers `/admin/v1/items/{item_id}/artwork`, `/artwork/{kind}/select`, and `/artwork/{kind}/selection`. | AWA-020 must split backend route work before Admin Web UI. |
| Admin DTOs are redaction-safe enough for UI summaries. | Medium | `docs/api/HTTP_API.md` and `nako-api::admin::managed_artwork` tests document redaction expectations. | Add backend/API DTO hardening before rendering gallery details. |
| Admin Web generated contract currently lacks item artwork routes/DTOs. | High | `NAKO_ADMIN_ROUTES` has no item artwork entries. | AWA-030 must add generated contract coverage before frontend UI work. |
| Item-scoped gallery/select/unpublish is the correct first artwork slice. | High | MBG follow-on split recommends item artwork selection after Generated Artifact review. | Keep lifecycle/remediation/ingest controls out of this lane. |

## Architecture Direction

- `App.tsx` owns route wiring and item/artwork path ownership.
- `adminApi/generated/contract.ts` must be generated from `nako-api` contract
  source, not hand-edited.
- `adminApi/client.ts` owns generated Admin API route calls.
- `adminApi/dataSource.ts` owns live/mock fallback and safe route summaries.
- `features/items/` or `features/artwork/` owns the item artwork UI.
- Shared components receive already-redacted display data.

## Closeout Condition

This lane can close when:

- gallery/select/unpublish route readiness is accepted or blockers are split;
- V2 exposes a guarded item artwork gallery and select/unpublish workflow;
- focused Admin Web and contract tests cover route rendering, data-source/client
  calls, confirmation behavior, mutation error behavior, fallback, and unsafe
  text exclusions;
- final Admin Web gates, relevant Rust/Admin contract gates, `git diff --check`,
  and browser smoke pass;
- ingest, lifecycle cleanup, remediation, provider search, and upload breadth
  remain split.

## Closeout Result

Closed 2026-05-25. Admin Web V2 now exposes a route-owned
`/items/:itemId/artwork` Managed Artwork gallery with guarded select/replace
and unpublish workflows, redaction-safe result rendering, visible mutation
failure states, no fake mutation fallback, full Admin Web gates, focused
Admin contract verification, and desktop/mobile browser smoke evidence.
