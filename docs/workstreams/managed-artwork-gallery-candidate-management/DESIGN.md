# Managed Artwork Gallery Candidate Management Design

Status: Completed
Last updated: 2026-05-19

## Problem

The Managed Artwork pipeline is now safe enough to serve selected images, but
selection is still narrow: operators can publish a known artifact ID, yet they
cannot inspect an item's artwork choices as a cohesive gallery or deliberately
switch between candidates and stored artifacts from a redacted management view.

Without an explicit gallery boundary, future UI work is likely to reach into raw
candidate rows, artifact storage metadata, or old `ImageAsset` provenance. That
would undo the redaction work from the previous lanes and blur the difference
between an Artwork Candidate, a Managed Artwork Artifact, and Selected Artwork.

## Target State

- Admin APIs expose an item-scoped artwork management view.
- The view separates:
  - Artwork Candidates: proposed choices and their safe metadata;
  - Managed Artwork Artifacts: fetched/stored internal artifacts eligible for
    selection;
  - Selected Artwork: current public image references per presentation slot.
- The view returns public-safe image references for selected artwork and safe
  Admin-only IDs for candidates/artifacts.
- Operators can intentionally publish or replace Selected Artwork by choosing a
  Managed Artwork Artifact.
- Future unpublish behavior is explicit and tested before it can delete or hide
  public images.
- Public Client item image responses remain selected-artwork only.

## Redaction Policy

Admin gallery and command responses must not include:

- `storage_uri`;
- `managed-artwork://...`;
- local paths or artifact root paths;
- raw candidate source URLs;
- `source_uri`;
- `cache_uri`;
- provider query strings;
- addon tokens or provider credentials;
- file contents;
- artifact content-hash values.

The first read model can expose boolean capability fields such as
`has_stored_artifact`, `has_content_hash`, or `selected`, but not the underlying
secret or storage values.

## Route Direction

Preferred first route:

```text
GET /admin/v1/items/{item_id}/artwork
```

The item-scoped path keeps the management view aligned with the operator task:
"show me all artwork choices for this item." Artifact-specific commands can
continue to use the existing artifact path:

```text
POST /admin/v1/artwork/artifacts/{artifact_id}/publish
```

Later commands may add:

```text
POST   /admin/v1/items/{item_id}/artwork/{kind}/select
DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection
```

The item/kind-scoped select command shipped in this lane. Unpublish remains
deferred because it needs explicit retention and public image visibility
policy.

## Architecture Direction

- `taru-core` owns domain records and repository trait additions when needed.
- `taru-db` should provide a query-oriented repository method for item-scoped
  artwork management state rather than forcing HTTP code to stitch unrelated
  rows.
- `taru-api` owns explicit Admin DTOs; do not reuse persistence records.
- `taru-server::app::artwork` owns management orchestration and redaction-safe
  response construction.
- `taru-server::http::admin` only parses IDs/query parameters and maps app
  responses.
- Public Client DTOs remain unchanged unless a later Public gallery lane is
  opened.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Admin management should come before Public candidate browsing. | High | Public Client currently has a clean selected-artwork-only contract. | Keep Public DTOs unchanged in this lane. |
| Item-scoped gallery is the right first read model. | Medium | Operators compare choices per item and image kind, not by raw artifact ID. | Keep artifact publish route compatible and add item-scoped view first. |
| Candidate source URL must stay internal even for Admin. | High | Previous lanes consistently redacted source/cache/storage authority. | Return source kind and safe dimensions only; add tests for forbidden strings. |

## Splits

- Persisted variant cache/eviction remains outside this lane.
- Durable retry/requeue/cancellation remains
  `managed-artwork-ingest-runtime-controls`.
- Missing-artifact repair/re-ingest remains a repair lane.
- Public Client candidate/gallery browsing should be a later lane after the
  Admin management model stabilizes.

## Closeout Condition

This lane can close when the item-scoped Admin gallery read model and at least
one safe selection management action are implemented, documented, and verified
with tests proving no raw source, cache, storage, path, token, or content-hash
values leak through Admin/Public responses.

Status: Closed. The shipped read model is
`GET /admin/v1/items/{item_id}/artwork`, and the shipped selection command is
`POST /admin/v1/items/{item_id}/artwork/{kind}/select`.
