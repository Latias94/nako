# Web Media Live Public Client Parity - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Open Lane

- [x] WMLP-010 [owner=planner] [deps=WDRP-020] [scope=docs/workstreams/web-media-live-public-client-parity]
  Goal: Open the Media Web live Public Client parity implementation lane.
  Validation: `python -m json.tool docs/workstreams/web-media-live-public-client-parity/WORKSTREAM.json`; `git diff --check -- docs/workstreams/web-media-live-public-client-parity`.
  Evidence: Initial design, route/API readiness, task ledger, and handoff.
  Handoff: DONE. Next task is WMLP-020.

## M1 - Public Client Readiness Audit

- [ ] WMLP-020 [owner=Codex] [deps=WMLP-010] [scope=web/src/api/public,sdk/typescript/src,crates/nako-client-protocol/src]
  Goal: Audit generated Public Client SDK methods and new `web/` data-source gaps for home rails, search, item detail, library browse, playback tickets, sessions, and playback state.
  Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run check`; route/API readiness doc updated.
  Evidence: `ROUTE_API_READINESS.md` updated with supported/missing contracts and first implementation slice confirmed.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M2 - Live Browse And Detail Parity

- [ ] WMLP-030 [owner=Codex] [deps=WMLP-020] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Make `/media`, `/media/search`, `/media/detail`, and `/media/library` use truthful live read models where SDK support exists, with explicit readiness states where it does not.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`.
  Evidence: data-source tests, route tests, and bundle output.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M3 - Browser Playback Entry

- [ ] WMLP-040 [owner=Codex] [deps=WMLP-030] [scope=web/src/api/public,web/src/features/media/video-player.tsx,web/src/test]
  Goal: Replace mock-only playback entry with a browser-ticket/session-backed playback decision path when the SDK contract is verified.
  Validation: data-source tests for playback decision/browser ticket/session heartbeat; route test for playback entry; `npm --prefix web run build:budget`.
  Evidence: playback contract tests and explicit no-token-in-media-url assertion.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M4 - Playback State

- [ ] WMLP-050 [owner=Codex] [deps=WMLP-040] [scope=web/src/api/public,web/src/features/media,web/src/test]
  Goal: Wire continue-watching, progress updates, and watched/unwatched state through Public Client user playback-state routes.
  Validation: data-source tests, route-state tests, and `npm --prefix web run test`.
  Evidence: playback state read/write contract tests.
  Handoff: DONE/BLOCKED/NEEDS_CONTEXT.

## M5 - Closeout

- [ ] WMLP-060 [owner=planner] [deps=WMLP-050] [scope=docs/workstreams/web-media-live-public-client-parity]
  Goal: Close the lane with browser/Tauri evidence, bundle budget output, and follow-on split for desktop native playback and recommendations.
  Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`; `npm --prefix web run tauri -- build`; browser smoke for Media routes; `git diff --check`.
  Evidence: EVIDENCE_AND_GATES.md closeout row.
  Handoff: Return to WDRP-030 or start the selected follow-on.

