# Research: Plex benchmark for operator journey

- Query: What Plex operator/journey patterns are most transferable to Nako, what should Nako avoid copying, and what is the best one-day follow-on slice?
- Scope: mixed
- Date: 2026-06-11

## Findings

Plex's operator journey is broad but coherent: first-run setup, account/sign-in, remote access, library/share management, explicit playback decision explanations, dashboard/logs, downloads/offline, media optimization, backup/restore, client/device coverage, and troubleshooting.

Most transferable patterns:

1. Readiness-first dashboard
   - A first-screen operator summary that aggregates setup, scan, playback, storage, network, and backup posture.
   - Nako already has this shape in `GET /admin/v1/overview`, so the structure is familiar and low-risk.

2. Explainable playback decisions
   - Plex teaches Direct Play / Direct Stream / Transcode as an operator-facing explanation, not just a player detail.
   - Nako already speaks in Direct Play / Remux / Transcode terms and has typed playback planning.

3. Redacted troubleshooting surfaces
   - Plex separates logs/troubleshooting from the player surface.
   - Nako already has a typed playback support evidence read model, so the missing step is UI projection, not new backend semantics.

Best one-day slice at the time of this benchmark:

- Ship a read-only `Playback Support Evidence` view in Admin Web, preferably linked from the existing item support card or the playback sessions area.
- Why this slice: the backend contract already exists, the client wrapper already exists, and the remaining work is mainly route wiring plus a redacted projection.
- This is the closest Nako equivalent to Plex's dashboard/logs/troubleshooting loop without inventing a new control-plane subsystem.

Update after implementation:

- The read-only `Playback Support Evidence` Admin Web route has shipped.
- The next one-day operator support slice should move up one level: a JSON-only redacted incident bundle export that aggregates existing safe diagnostics into a single support artifact.
- This preserves the same Plex lesson, but avoids reopening a now-complete playback support page.

Do not copy:

1. Plex.tv-centered account/relay/claim flow
   - Nako should stay explicit about local endpoints, reverse proxies, or tunnel providers.

2. Paid gating of operator basics
   - Plex Pass is fine for Plex, but not a good model for core self-hosted diagnostics or troubleshooting.

3. Cloud-dependent support path
   - Nako should not require a vendor portal to inspect logs or diagnose playback.

## Files found

- `docs/architecture/OPERATIONS_RELEASE.md` - operator readiness, backup, diagnostics, and recovery map.
- `docs/architecture/CONTROL_PLANE.md` - remote access, diagnostics, incident bundles, and support boundaries.
- `docs/architecture/PLAYBACK.md` - playback planning, readiness, and runtime diagnostics map.
- `apps/admin-web/src/App.tsx` - current admin route inventory; includes the shipped playback support route but no incident bundle route.
- `apps/admin-web/src/features/overview/OverviewPage.tsx` - readiness dashboard with setup/scan/playback/storage/network/backup checks.
- `apps/admin-web/src/features/settings/SettingsPage.tsx` - network and playback runtime diagnostics/settings.
- `apps/admin-web/src/features/items/ItemDetailPage.tsx` - support card with adjacent support links.
- `crates/nako-server/src/http/admin.rs` - overview, network, jobs, playback runtime diagnostics, and support evidence assembly.
- `crates/nako-api/src/admin/playback.rs` - typed playback support evidence DTO and redaction test.
- `apps/admin-web/src/adminApi/client.ts` - typed client exposes playback support evidence; incident bundle client wiring remains absent.
- `docs/workstreams/admin-web-v2-library-management-and-localization/PARITY_GAP_SPLIT.md` - notes playback support detail as a follow-on.
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/FOLLOW_ON_SPLIT.md` - splits playback support detail into a bounded follow-on.

## Code patterns

- `apps/admin-web/src/App.tsx:230` - route inventory includes `/playback/support`; no dedicated incident bundle route exists yet.
- `apps/admin-web/src/features/overview/OverviewPage.tsx:110,150,211,414` - overview already renders operator readiness checks for setup, media library scan, playback, storage, network, and backup.
- `apps/admin-web/src/features/settings/SettingsPage.tsx:239,529,797` - settings already surface network exposure, tunnel providers, and playback runtime editing.
- `crates/nako-server/src/http/admin.rs:1468,2751,3173,3358,3524,3892,4212` - overview assembly, network diagnostics, queue pressure, playback support evidence, runtime readiness, and backup readiness already exist server-side.
- `crates/nako-api/src/admin/playback.rs:367` - `AdminPlaybackSupportEvidenceResponse` exists and is covered by a redaction test.
- `apps/admin-web/src/adminApi/client.ts:641` and `apps/admin-web/src/adminApi/mockData.ts:2506` - admin-web already has a typed client and mock data for playback support evidence.
- `apps/admin-web/src/features/items/ItemDetailPage.tsx:172` - the item support card offered adjacent support links before the playback support view shipped.
- `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md:53,61,67` - overview and playback readiness are already treated as M1 diagnostics; incident bundles remain deferred.

## External references

- Quick Start: https://support.plex.tv/articles/200264746-quick-start-step-by-step-guides/
- What is Plex: https://support.plex.tv/articles/200288286-what-is-plex/
- Remote Access: https://support.plex.tv/articles/200289506-remote-access/
- Troubleshooting Remote Access: https://support.plex.tv/articles/200931138-troubleshooting-remote-access/
- Accessing a Server Through Relay: https://support.plex.tv/articles/216766168-accessing-a-server-through-relay/
- Direct Play / Direct Stream: https://support.plex.tv/articles/200250387-streaming-media-direct-play-and-direct-stream/
- Streaming Overview: https://support.plex.tv/articles/200430303-streaming-overview/
- Status and Dashboard: https://support.plex.tv/articles/200871837-status-and-dashboard/
- Plex Media Server Logs: https://support.plex.tv/articles/201455336-crash-logs-plex-media-server/
- Plex Web App Logs: https://support.plex.tv/articles/201611836-plex-web-app-logs/
- Downloads Overview: https://support.plex.tv/articles/downloads-overview/
- Downloads on Desktop: https://support.plex.tv/articles/downloads-on-desktop/
- Downloads Sync FAQ: https://support.plex.tv/articles/downloads-sync-faq/
- Media Optimizer Overview: https://support.plex.tv/articles/214079318-media-optimizer-overview/
- Optimized Versions: https://support.plex.tv/articles/213097057-optimized-versions/
- Example Media Optimizer Usage: https://support.plex.tv/articles/214079348-example-media-optimizer-usage/
- Library Actions: https://support.plex.tv/articles/200392106-library-actions/
- Creating and Managing Server Shares: https://support.plex.tv/articles/201105738-creating-and-managing-server-shares/
- Restricting the Shares: https://support.plex.tv/articles/204232573-restricting-the-shares/
- Backing Up Plex Media Server Data: https://support.plex.tv/articles/201539237-backing-up-plex-media-server-data/
- Restore a Database Backed Up via Scheduled Tasks: https://support.plex.tv/articles/202485658-restore-a-database-backed-up-via-scheduled-tasks/
- Opening Plex Web App: https://support.plex.tv/articles/200288666-opening-plex-web-app/
- Connect App to Your Plex Account: https://support.plex.tv/articles/203395277-connect-app-to-your-plex-account/
- Supported Plex Companion Apps: https://support.plex.tv/articles/203082707-supported-plex-companion-apps/
- General Troubleshooting: https://support.plex.tv/articles/200430313-troubleshooting/

## Related specs

- `.trellis/spec/guides/index.md`
- `.trellis/spec/nako-server/backend/index.md`
- `.trellis/spec/nako-api/backend/index.md`
- `.trellis/spec/nako-playback/backend/index.md`
- `.trellis/spec/admin-web/frontend/index.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/PLAYBACK.md`

## Caveats / Not Found

- Plex support docs are product docs, not API contracts; some features are Plex Pass gated or platform-specific.
- I did not find a dedicated Nako logs page or incident bundle page. The closest analogue is the typed support evidence model plus readiness/diagnostic pages.
- The original playback-support one-day recommendation has since shipped; use this benchmark as predecessor context for the incident bundle follow-on.
