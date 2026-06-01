# Web MVP Live Smoke - Design

Status: Active
Last updated: 2026-06-01

## Why This Lane Exists

The MVP release ladder requires a reproducible Web/Public Client smoke for the
browser-first media experience. Existing Web parity work proves many individual
contracts, but the release cut needs one focused evidence lane that names the
MVP path and avoids expanding into desktop, native playback, backend contract,
or generated-artifact strategy work.

## Relevant Authority

- `AGENTS.md`
- `CONTEXT.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/mvp-release-shape/CAMPAIGNS.md`
- `docs/workstreams/mvp-release-shape/EVIDENCE_AND_GATES.md`
- `docs/workstreams/web-media-live-public-client-parity/`
- `docs/workstreams/admin-media-management-context-links/`

## Problem

Gate 3 in the MVP release ladder requires proof that `/media`,
`/media/library`, `/media/detail`, browser-ticket playback, native video/subtitle
rendering, playback heartbeat, and redaction-safe surfaces can be checked as a
coherent Web MVP path. The repo has partial evidence spread across route,
data-source, and component tests, but no dedicated smoke artifact for release
review.

## Target State

The Web MVP path is represented by a named, repeatable smoke test and fresh
evidence:

- `/media` loads through Public Client media data sources.
- `/media/library?id=<library_id>` loads library-scoped item browse through the
  Public Client library browse route.
- `/media/detail?id=<item_id>&type=<media_type>` renders live item detail and
  safe source context.
- Browser playback ticket creation produces browser-safe media/subtitle URLs.
- `VideoPlayer` renders native media and subtitle elements.
- Playback heartbeat uses `playback_session_id`.
- Checked Web surfaces do not expose bearer tokens, raw local paths, source
  locators, or secret payloads.

## In Scope

- `docs/workstreams/web-mvp-live-smoke/`
- `web/src/test`
- `web/src/api/public` if the smoke reveals a local Web data-source defect
- `web/src/features/media` if the smoke reveals a local route/component defect
- existing Web smoke evidence paths when needed

## Out Of Scope

- Rust backend, Public Client API, Admin API, generated client, or schema
  contract changes.
- Tauri/native desktop playback strategy.
- GAMA, CSAPA, or MVP planner task-ledger changes beyond required workstream
  links.
- New product scope such as downloads, music, photos, acquisition, or mobile.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public Client library-scoped item browse is now available to Web. | High | `web/src/api/public/media-data-source.ts` calls `client.listLibraryItems`; route tests reference `/libraries/{id}/items`. | Stop and return `NEEDS_CONTEXT` if the generated SDK/backend contract is missing. |
| Browser playback ticket responses expose `playback_session_id`. | High | `PublicPlaybackPlan.playbackSessionId` maps `mediaTicket.playback_session_id`. | Stop and return `NEEDS_CONTEXT` if the SDK contract no longer exposes it. |
| Web MVP evidence can be added without backend changes. | High | Existing WMLP evidence covers live data-source, route, and player seams. | Return to planner if the smoke requires Public/Admin API changes. |

## Architecture Direction

The browser MVP stays a thin Web product over the Public Client API:

- `web/src/api/public` maps Public Client DTOs into UI-safe read models.
- Media routes and components consume those read models without importing Admin
  mutation or backend internals.
- Playback uses browser tickets and playback sessions, not raw local paths,
  source locators, bearer tokens, or direct filesystem handles.
- Management context links remain safe handoffs to Admin surfaces; mutating
  behavior stays Admin-owned.

## Closeout Condition

This lane can close when:

- the dedicated Web MVP smoke passes with the broader Web gates;
- evidence is recorded in `EVIDENCE_AND_GATES.md`;
- residual MVP risks are limited to other lanes such as `playback-transcode`;
- follow-ons are either split or explicitly deferred.

