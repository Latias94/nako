# Media Browse And Item Detail Route/API Readiness

Status: Accepted
Last updated: 2026-05-25

This document records MBG-020. It decides which current routes can support the
first `/catalog` and `/items/:itemId` Admin Web V2 implementation and which
gaps must remain split.

## Decision

MBG-030 can start without backend work.

The first `/catalog` route should use explicit public-read bridge methods for:

- `GET /items?limit=...&offset=...`
- `GET /search?q=...&facet=...&limit=...&offset=...`

The first `/items/:itemId` route can use explicit public-read bridge methods
for:

- `GET /items/{item_id}`
- `GET /items/{item_id}/credits`
- `GET /items/{item_id}/images`
- bounded `GET /sources/{source_id}/probe` calls for sources already present on
  the item detail response

Admin-only supporting reads may be linked or summarized only when already
available through generated Admin API contract methods. Mutation and repair
actions stay out of the first browse/detail slice.

## Route Readiness Matrix

| Route or capability | Current surface | Readiness | MBG decision |
| --- | --- | --- | --- |
| Catalog list | `GET /items` | Ready as public read. Supports pagination only. | Use `publicCatalogItemsBridge` for the first `/catalog` default view. |
| Catalog search | `GET /search` | Ready as public read. Supports query, repeated facets encoded as comma-separated route query, and pagination. | Use `publicCatalogSearchBridge`; keep filters simple in first slice. |
| Item detail | `GET /items/{item_id}` | Ready as public read. Includes item, sources, credits, genres, tags, collections, studios, and images. | Use `publicItemDetailBridge` as the primary `/items/:itemId` source. |
| Item credits | `GET /items/{item_id}/credits` | Ready as public read. | Optional section refresh if detail response is insufficient; otherwise detail response may be enough. |
| Item images | `GET /items/{item_id}/images` | Ready as public read. | Use for image readiness summaries only; do not fetch image bytes in route tests. |
| Source technical facts | `GET /sources/{source_id}/probe` | Ready as public read. | Use bounded per-source calls in item detail; summarize container/duration/stream counts only. |
| Playback support evidence | `GET /admin/v1/playback/support?source_id=...` | Ready through generated Admin API contract. | Optional support link or summary after item sources are visible; no playback controls. |
| Catalog governance queue | `GET /admin/v1/catalog/governance/items` | Ready through generated Admin API contract. | Keep `/catalog/governance` as the repair queue; `/catalog` may link to it but should not merge its mutation roadmap. |
| Metadata attempts/raw/candidates | `/items/{item_id}/metadata/*` | Exists outside generated Admin API v1 contract and may expose provider diagnostics/raw cache. | Split Admin metadata evidence route or bridge policy before rendering in item detail. First item detail shows readiness/placeholders only. |
| Generated Artifact item artifacts | `GET /items/{item_id}/automation/artifacts` | Exists outside generated Admin API v1 contract. | Split per-item Generated Artifact read/review workflow; first item detail may link to `/automation/generated-artifacts`. |
| Admin item artwork candidates/selection | `/admin/v1/items/{item_id}/artwork*` routes exist in HTTP docs/server but are not in generated Admin Web contract constants. | Backend exists, frontend contract not generated here; mutations need confirmation/audit UX. | Split generated contract and artwork decision workflow. First item detail uses public image refs only. |
| Provider Mapping, Local Inference, NFO sidecar status | No complete generated Admin item detail read model. | Gap. | Represent as readiness placeholders and split governance repair actions after item detail lands. |
| Direct playback, watch state, favorites, ratings | Public/client-user surfaces exist. | Out of Admin Web scope for this lane. | Do not render controls or personal state in Admin Web browse/detail. |

## Bridge Naming

Use explicit names so Admin Web does not blur public and admin contracts:

- `getPublicCatalogItemsBridge(query)`
- `getPublicCatalogSearchBridge(query)`
- `getPublicItemDetailBridge(itemId)`
- `getPublicItemCreditsBridge(itemId)`
- `getPublicItemImagesBridge(itemId)`
- `getPublicSourceProbeBridge(sourceId)`

These bridge methods should live in `adminApi/client.ts` and return local
Admin Web TypeScript shapes until a generated public-client TypeScript contract
is intentionally shared with Admin Web.

## Safe Projection Rules

The route data source must map public DTOs into route summaries before rendering:

- show Media Item IDs, titles, kind, runtime, release date, genre/tag counts,
  image counts, source counts, and safe source filenames;
- show source size, fingerprint presence, duration/container/stream count only
  after probe data is summarized;
- do not render Source Locators, local filesystem paths, raw provider bodies,
  artifact storage handles, playback output paths, bearer tokens, Secret
  References, or raw response bodies;
- keep public image URLs as route paths only when needed for thumbnails; do not
  expose internal artifact storage or source URLs;
- keep unknown enum values as backend comparison strings, not localized product
  copy.

## First Implementation Slice

MBG-030 should implement `/catalog` using:

- route-owned search params: `q`, `facet`, `limit`, `offset`;
- default browse through `GET /items`;
- search mode through `GET /search` when `q` or `facet` is present;
- safe rows with title, kind, release date, runtime, genre/tag counts, image
  count, source count, and a detail link;
- deterministic mock fallback and unsafe-text route tests.

MBG-040 should then implement `/items/:itemId` using:

- `GET /items/{item_id}` as the primary read;
- optional credits/images refresh only if needed for section-local fallback;
- bounded source probe summaries;
- readiness placeholders for NFO/provider/local-inference/repair actions that
  need follow-on Admin API work.
