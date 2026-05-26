# Admin Web V2 Media Browsing And Item Detail Governance

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

Admin Web V2 now has route-first operational pages and a real Media Library
management workflow, but it still lacks the core governance question operators
ask after scanning a library:

- Which Media Items exist?
- Which Media Sources back an item?
- What Canonical Metadata and local/provider evidence does Nako have?
- Is artwork, NFO, local inference, or provider mapping ready enough for
  repair/review work?
- Where can an operator jump for scan, NFO, generated artifact, artwork, or
  playback support diagnostics?

Jellyfin/Plex-style consoles make item inspection easy, but Nako should not copy
their consumption-first shape. This lane is for administration-supporting
browse/detail, not a poster-wall playback client.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/admin-web-v2-library-management-and-localization/`
- `docs/workstreams/admin-web-v2-catalog-governance-route/`
- `docs/workstreams/admin-catalog-governance-read-model/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/managed-artwork-gallery-candidate-management/`
- `docs/workstreams/playback-transcode-ops-hardening/`
- `docs/api/HTTP_API.md`

## Problem

Current V2 state:

- `/catalog/governance` shows a redacted queue, not a general item browse route.
- Public Client API browse/search/item routes exist, but Admin Web does not yet
  consume them through explicit bridges.
- Item detail evidence is scattered across public item routes, source probe,
  metadata diagnostics, generated artifacts, artwork workstreams, and playback
  support routes.
- Repair/apply mutations are not ready to be mixed into first item detail.
- Full-site i18n is incomplete, so new route copy must either join the catalog
  or remain explicitly split.

## Target State

When this lane closes:

- `/catalog` is a route-owned, governance-oriented browse/search entry.
- `/items/:itemId` is a route-owned item detail page.
- Admin Web data-source bridges public browse/item reads explicitly and maps
  them to safe Admin Web summaries.
- Item detail shows safe Media Item facts, source context, Canonical Metadata
  summary, artwork/Generated Artifact readiness, NFO/provider/local-inference
  readiness, and relevant support links.
- No unsafe Source Locator, local path, artifact storage handle, raw provider
  payload, playback output path, token, or secret-like text is rendered.
- Repair/apply/action workflows are either kept out with visible readiness
  states or split into a dedicated follow-on.
- Browser smoke covers desktop and mobile for `/catalog` and `/items/:itemId`.

## In Scope

- Admin Web routes under `/catalog` and `/items/:itemId`.
- Public-read bridge policy for browse/search/item detail and supporting reads.
- Route-owned URL search params for browse filters.
- Item detail safe read model and fallback behavior.
- Route tests, data-source tests, redaction tests, browser smoke, and workstream
  evidence.
- Route-local i18n message IDs if the added copy is touched broadly enough to
  justify migration in this lane.

## Out Of Scope

- Playback controls, watch-state, favorites, ratings, or user-facing browse
  personalization.
- User, Role, or Library Access management.
- Settings or runtime configuration mutation.
- Catalog repair, Provider Mapping accept, Generated Artifact accept/reject,
  Selected Artwork changes, or other apply mutations.
- Public Client API contract redesign unless a route gap blocks the lane and is
  explicitly split or accepted.
- Copying Jellyfin, Plex, or reference-project UI/source/assets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public item browse/detail routes are sufficient for a first `/catalog` and `/items/:itemId` read slice. | Medium | `ADMIN_API_MATRIX.md` lists public browse/item/credits/images/search routes as console-supporting reads. | MBG-020 must split an Admin read-model backend task before UI implementation. |
| Item detail should be read-heavy before repair actions. | High | Library-management closeout split repair/apply workflows into a later lane. | Keep first item detail as readiness/support, then open governance repair actions. |
| Admin Web can bridge Public Client API reads when names are explicit and summaries are safe. | High | ADR 0027 permits public reads for genuinely client-facing information. | Add a dedicated Admin read model if public DTOs expose unsafe or insufficient data. |
| This lane should not become a playback client. | High | `PRODUCT.md` and ADR 0026 keep Admin Web administration-first. | Route copy and actions must stay governance/support oriented. |

## Architecture Direction

- `adminApi/client.ts` owns route calls. Public Client API calls used by Admin
  Web must be named as bridges, not blended into Admin API methods.
- `adminApi/dataSource.ts` owns safe projection from public/admin DTOs into
  route-local summaries.
- `features/catalog/` should own `/catalog` browse and reuse governance
  vocabulary without coupling to repair mutations.
- `features/items/` should own `/items/:itemId` once item detail is introduced.
- Shared UI components remain neutral and receive already-rendered strings.

## Closeout Condition

This lane can close when:

- `/catalog` and `/items/:itemId` are either implemented or split with precise
  blockers;
- unsafe rendered text tests cover browse/detail routes;
- public-read bridge decisions are documented;
- targeted Admin Web gates and browser smoke pass;
- repair/action follow-ons are split;
- `WORKSTREAM.json`, `TODO.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` are
  updated with final evidence.
