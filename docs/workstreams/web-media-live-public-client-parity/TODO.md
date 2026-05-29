# Web Media Live Public Client Parity - TODO

Status: Completed
Last updated: 2026-05-28

## M0 - Open Lane

- [x] WMLP-010 [owner=planner] [deps=WDRP-020] [scope=docs/workstreams/web-media-live-public-client-parity]
  Goal: Open the Media Web live Public Client parity implementation lane.
  Validation: `python -m json.tool docs/workstreams/web-media-live-public-client-parity/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-media-live-public-client-parity`.
  Evidence: Initial design, route/API readiness, task ledger, and handoff.
  Handoff: DONE. Next task is WMLP-020.

## M1 - Public Client Readiness Audit

- [x] WMLP-020 [owner=Codex] [deps=WMLP-010] [scope=web/src/api/public,sdk/typescript/src,crates/nako-client-protocol/src]
  Goal: Audit generated Public Client SDK methods and new `web/` data-source gaps for home rails, search, item detail, library browse, playback tickets, sessions, and playback state.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`; route/API readiness doc updated.
  Evidence: `ROUTE_API_READINESS.md` records supported SDK methods, missing library-scoped item browse/sort contracts, and the WMLP-030 first implementation slice. Validation passed with 13 data-source tests and `tsc --noEmit`.
  Handoff: DONE. Next task is WMLP-030.

## M2 - Live Browse And Detail Parity

- [x] WMLP-030 [owner=Codex] [deps=WMLP-020] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Make `/media`, `/media/search`, `/media/detail`, and `/media/library` use truthful live read models where SDK support exists, with explicit readiness states where it does not.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Evidence: Public Media read-model boundaries, detail/search route integration, and library metadata/source readiness are implemented. `npm --prefix web run test`, `npm --prefix web run check`, and `npm --prefix web run build:budget` passed.
  Handoff: DONE. Next task is WMLP-040.

  Substeps:
  - [x] WMLP-030A: Public Media read-model boundary and contract tests.
  - [x] WMLP-030B: Route `/media`, `/media/search`, and `/media/detail` through live read models.
  - [x] WMLP-030C: Route `/media/library` through library metadata/source readiness without fake scoped item browse.

## M3 - Browser Playback Entry

- [x] WMLP-040 [owner=Codex] [deps=WMLP-030] [scope=web/src/api/public,web/src/features/media/video-player.tsx,web/src/test]
  Goal: Replace mock-only playback entry with a browser-ticket/session-backed playback decision path when the SDK contract is verified.
  Validation: data-source tests for playback decision/browser ticket/session heartbeat; route test for playback entry; `npm --prefix web run build:budget`.
  Evidence: Browser-ticket playback plan and native video source/subtitle rendering are implemented with no-token assertions. Session heartbeat is recorded as a contract follow-on because `BrowserPlaybackTicketResponse` does not expose a playback session id to the web client.
  Handoff: DONE_WITH_CONCERNS. Next task is WMLP-050, with heartbeat/session identity split as follow-on.

  Substeps:
  - [x] WMLP-040A: Browser ticket/subtitle playback contract is isolated in the Public Media data source with no-token assertions.
  - [x] WMLP-040B: `VideoPlayer` can render browser-ticket media URLs and subtitle tracks.
  - [x] WMLP-040C: Split missing browser playback session-id/heartbeat contract as a backend/API follow-on.

## M4 - Playback State

- [x] WMLP-050 [owner=Codex] [deps=WMLP-040] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Wire continue-watching, progress updates, and watched/unwatched state through Public Client user playback-state routes.
  Validation: data-source tests, route-state tests, and `npm --prefix web run test`.
  Evidence: Continue-watching, progress writes, and watched-state writes are exposed through Public Media data source and covered by contract tests. Home continue-watching reads Public Client playback-state data with fixture fallback.
  Handoff: DONE. Next task is WMLP-060.

## M5 - Closeout

- [x] WMLP-060 [owner=planner] [deps=WMLP-050] [scope=docs/workstreams/web-media-live-public-client-parity]
  Goal: Close the lane with browser/Tauri evidence, bundle budget output, and follow-on split for desktop native playback and recommendations.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `npm --prefix web run tauri -- build`; browser smoke for Media routes; `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md closeout row records web tests, TypeScript check, bundle budget, Tauri build, browser smoke, JSON validation, and diff check.
  Handoff: DONE. Return to WDRP-030 or start a selected follow-on.
