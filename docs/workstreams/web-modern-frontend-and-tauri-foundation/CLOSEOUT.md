# Web Modern Frontend And Tauri Foundation Closeout

Status: Closed
Closed: 2026-05-28

## Closeout Claim

This lane is closed as a foundation lane.

Nako now has a release-oriented `web/` frontend package, first API-backed
Media/Admin route families, a Tauri desktop shell path, and an explicit
retirement plan for the old `apps/admin-web` validation console.

This closeout does not claim full product parity, native desktop playback
quality, old Admin Web deletion, Addon Manager lifecycle parity, Admin mutation
parity, complete redaction corpus migration, i18n, or Jellyfin-class breadth.
Those are split follow-ons.

## Delivered

- `web/` Vite React product frontend using React 19, TypeScript, Tailwind v4,
  shadcn-style local primitives, TanStack Router/Query, and lucide icons.
- Route-owned Media and Admin surfaces under `/media/*` and `/admin/*`.
- Nako-owned visual tokens, active navigation, setup/account routes, and
  generated artwork placeholders instead of v0/third-party bundled assets.
- `web/src/api/media` using Public Client SDK boundaries and fixture/live
  results.
- `web/src/api/admin` using a generated Admin API TypeScript contract local to
  `web/`.
- `nako-api` generator drift coverage for both the old Admin generated
  contract and the new `web/` generated contract.
- Media Web first slice: Continue Watching, libraries, library detail, item
  detail, source selection, and browser playback ticket request handling.
- Admin Web first slice: overview, libraries, jobs, Addons, and settings
  readiness, intentionally read-only.
- Tauri v2 shell in `web/src-tauri`, Nako-owned app icons, no-secret server
  profile bootstrap commands, runtime `/health` verification before profile
  storage, and Windows build smoke.
- `RUST_CAPABILITY_GAPS.md` recording server/Rust capability gaps the frontend
  must not fake.
- `ADMIN_WEB_RETIREMENT_PLAN.md` recording why `apps/admin-web` remains
  validation-only and what must happen before deletion.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: old Admin Web is not retired; it is deliberately retained by
  parity plan because it still covers mutation, redaction, i18n, query mapping,
  and Media playback state responsibilities that `web/` does not yet own.
- Important: this lane closes by splitting remaining work, not by claiming the
  whole product frontend is done.

### Code Quality

- Blocking: none.
- Important: `web/src/router.tsx` is now a useful tracer but is too large for
  long-term route ownership. Split route modules before adding broad Admin or
  Media depth.
- Important: production `web/` JS is 411.41 kB minified / 126.68 kB gzip. Add
  route-level splitting before growing Addon Manager, admin tables, or playback
  depth.
- Important: Tauri bootstrap stores a server base URL only. It intentionally
  does not implement credential vault storage or native playback.

### Missing Gates

- Full Rust workspace tests were not run; this lane changed frontend/Tauri shell
  code and docs, and used focused Rust gates for `nako-api` contract drift and
  `web/src-tauri`.
- No live backend smoke was run for the new `web/` frontend. Fixture/live seams
  and browser route smoke were validated; live server smoke remains a release
  readiness gate.

## Follow-Ons

1. Addon Manager UI productization: install/update/remove, grants, credentials,
   sidecar lifecycle, trust UX, and hosted-page boundary.
2. Admin mutation parity in `web/`: settings raw cache, library metadata
   profile, generated artifact review, catalog mapping review, artwork
   selection, job commands, and confirmation/redaction tests.
3. Redaction corpus migration: shared fixture corpus or route tests that cover
   the unsafe local paths, raw provider/source fields, output paths, Addon
   secrets, and generated artifact payloads currently tested in `apps/admin-web`.
4. Media playback depth: playback decision preview, progress writes, pause/ended
   flush, watched state, richer source/version UX, and live server smoke.
5. Native desktop playback core: Rust/native player integration split from
   WebView playback per ADR 0026.
6. Route-level code splitting and bundle budget for `web/`.
7. i18n decision and implementation for `web/`, especially if `zh-Hans` remains
   a release requirement.
8. Old Admin Web deletion task after every `ADMIN_WEB_RETIREMENT_PLAN.md` row is
   ready, moved to CI, or split to accepted follow-ons.
9. Remote access/operator network UX, acquisition/download intake, notifications,
   backup/restore operations, and other self-hosted server breadth tracked in
   `RUST_CAPABILITY_GAPS.md`.

## Evidence Anchors

- `docs/workstreams/web-modern-frontend-and-tauri-foundation/EVIDENCE_AND_GATES.md`
- `docs/workstreams/web-modern-frontend-and-tauri-foundation/RUST_CAPABILITY_GAPS.md`
- `docs/workstreams/web-modern-frontend-and-tauri-foundation/ADMIN_WEB_RETIREMENT_PLAN.md`
- `web/src/api/media/client.ts`
- `web/src/api/admin/client.ts`
- `web/src/api/runtime.ts`
- `web/src/api/desktop.ts`
- `web/src/router.tsx`
- `web/src-tauri/src/lib.rs`
- `.playwright-cli/wmft-050-media-watch-desktop.png`
- `.playwright-cli/wmft-060-admin-settings-mobile.png`
- `.playwright-cli/wmft-070-setup-desktop.png`
- `.playwright-cli/wmft-070-setup-mobile.png`
