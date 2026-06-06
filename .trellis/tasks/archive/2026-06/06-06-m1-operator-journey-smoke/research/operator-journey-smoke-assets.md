# Operator Journey Smoke Assets

## Sources Read

- `CONTEXT.md`
- `docs/ROADMAP.md`
- `docs/architecture/LANES.md`
- `docs/ARCHITECTURE.md`
- `docs/workstreams/mvp-release-shape/CLOSEOUT.md`
- `docs/workstreams/web-mvp-live-smoke/HANDOFF.md`
- `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md`
- `scripts/self-host-smoke.ps1`
- `scripts/release-gate.ps1`
- `apps/admin-web/src/surfaces/media/MediaPages.tsx`
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
- `apps/admin-web/src/App.test.tsx`
- `crates/nako-server/src/app/tests/*.rs`

## Existing Evidence

- `scripts/self-host-smoke.ps1` already runs
  `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`.
- `scripts/release-gate.ps1 -Mode fast` already includes formatting,
  diff-check, redaction inventory, API/Admin SDK gates, managed-artwork gates,
  and `nako-server self_host_smoke`.
- `scripts/release-gate.ps1 -Mode playback` adds FFmpeg/ffprobe checks,
  playback hardware reporting, transcode hardware/HLS gates, and server
  `self_host_smoke`.
- `docs/workstreams/mvp-release-shape/CLOSEOUT.md` records the old release
  ladder as passed for core server, Web/Public Client, playback runtime,
  package/container shape, and managed-artwork PostgreSQL compatibility.
- `docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md` records
  deterministic Web smoke coverage for `/media`, library browse, detail,
  browser playback tickets, native video/subtitle rendering, heartbeat through
  `playback_session_id`, and no raw secret/path exposure.
- Admin Web already has route tests for library management, library scan
  commands, media browse/playback surfaces, playback sessions, storage
  diagnostics, catalog governance, source duplicate reconciliation, and
  redaction expectations.

## Gap For Product-Operator M1

The existing assets prove many parts independently, but the first M1 task needs
one named, repeatable operator journey smoke that tells future agents what to
run and what it proves:

- one configured Media Library is visible;
- scan/index work is commandable and observable;
- catalog or media browse can find playable entries;
- playback can choose Direct Play, Remux, or HLS through existing public/media
  surfaces;
- Admin diagnostics and repair surfaces stay available and redaction-safe;
- historical release-ladder evidence remains linked but is not the only
  reading path.

## Recommended First Slice

Use a minimal wrapper/docs-and-test slice rather than a broad product
implementation:

- define an M1 operator journey smoke artifact that composes existing backend
  `self_host_smoke`, Admin Web route/media tests, and release-gate commands;
- add or harden only the smallest missing deterministic test coverage needed
  to connect config -> scan -> browse -> playback -> diagnostics/repair;
- keep actual source hash automation, source duplicate operator flow,
  full player polish, and one-command release ladder packaging as follow-on
  Trellis tasks from the M1 queue.

## Candidate Gates

- `cargo nextest run -p nako-server self_host_smoke --no-fail-fast`
- `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx`
- focused `apps/admin-web/src/App.test.tsx` tests for library scan,
  playback sessions, storage diagnostics, catalog governance, and source
  duplicate diagnostics when touched
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1`
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs`
- `git diff --check`

## Out Of Scope For This Slice

- new schema, public API, generated contract, or release artifact publication;
- automatic source duplicate merging;
- source hash trigger policy implementation;
- broad Media Web or player UX redesign;
- Addon Manager or official Addon Sidecar proof.
