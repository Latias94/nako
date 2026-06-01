# MVP Gap Matrix

Status: MRS-040 active queue aligned
Last updated: 2026-06-01

`MRS-020` checked the initial MVP release cut against current architecture
docs, completed workstream evidence, selected code/test inventory, deployment
scripts, `web/` Public Client usage, active lane state, and the related
`nako-official-addons` repository. This is a release-planning evidence pass,
not a fresh full test run.

Decision labels:

- `Evidence-backed`: keep in P0; `MRS-030` only needs to choose exact gates.
- `MVP blocker`: finish, split, or explicitly accept before MVP closeout.
- `Conditional`: not a blocker unless the MVP product cut exposes that path.
- `Deferred`: keep out of P0.

| Area | P0 decision | Evidence checked | Verified state | MVP route |
| --- | --- | --- | --- | --- |
| Active queue | MVP blocker | `docs/architecture/LANES.md`; workstream inventory; active PTJCH/GAMA/CSAPA docs. | `PTJCH-220` remains a real release blocker because playback runtime ownership is on the required journey. `GAMA-060` is conditional. `CSAPA-050` is P1 unless desktop-native playback is promoted. Known lane drift for GAMA/CSAPA must not be hidden. | `MRS-040` must align active tails before closeout. |
| Install and release | Evidence-backed | `docs/deployment/SELF_HOSTED.md`; `docs/deployment/RELEASE_CHECKLIST.md`; `scripts/release-gate.ps1`; compose/container files. | Local config-check, self-host smoke, package/compose artifacts, container preflight, and release checklist exist. The remaining gap is naming the MVP smoke ladder, not designing deployment from scratch. | `MRS-030` picks local/container required gates. |
| Access and initial admin | Evidence-backed | `CONTEXT.md`; deployment docs; admin-token config/check docs. | Single-Admin Mode can stay in P0 if the smoke proves initial admin auth and health/config readiness. Multi-user polish is not required. | `MRS-030` includes auth/config smoke. |
| Media library scan and source state | Evidence-backed | `docs/architecture/LIBRARY_PIPELINE.md`; `crates/nako-server/src/app/tests/startup.rs`; storage/source workstreams. | Local library scan, source/probe persistence, tombstone/source state, NFO import on scan, and addon metadata-scrape hooks have code/test evidence. Watcher/debounce productization remains P1 unless first-run scan fails. | `MRS-030` names the focused scan gate. |
| Metadata authority | Evidence-backed | Metadata catalog workstreams; `crates/nako-server/src/app/tests/metadata.rs`; generated-artifact apply tests. | Local inference, NFO, provider job planning, authority/field-level apply behavior, and redaction have evidence. Generated Artifact Web apply is not P0 unless the MVP requires that Web workflow. | Keep backend authority in P0; route `GAMA-060` through `MRS-040` as conditional/P1. |
| Web browse/playback | MVP blocker until smoke-gated | `web-media-live-public-client-parity`; `web/src/api/public/media-data-source.ts`; `web/src/features/media/*`; Public Client protocol/openapi inventory. | Browser/web is the accepted MVP client path. Public Client now exposes library item browse, browser playback tickets, playback session ids, and heartbeat routes. Remaining risk is release-level live Web smoke and any fixture/fallback contract gaps. | `MRS-030` must name a live web browse/detail/playback smoke. If it fails, split a `web-product` blocker. |
| Playback runtime | MVP blocker | `docs/architecture/PLAYBACK.md`; `playback-transcode-jellyfin-class-hardening`; `crates/nako-server/src/app/tests/playback.rs`. | Direct/Remux/HLS has broad test evidence, but `PTJCH-220` still owns runtime session/admission/reuse/cancel/failure classification and diagnostics. This is on the required MVP journey. | Finish/split/accept `PTJCH-220` before MVP closeout. |
| Playback artifact I/O pressure | Conditional | PTJCH follow-on notes and storage/VFS maps. | `PTJCH-310` should decide whether artifact I/O pressure stays in PTJCH or splits to PAIP/storage. Not a standalone MVP blocker unless the playback gate shows unsafe behavior. | Keep as PTJCH follow-on unless gate evidence escalates it. |
| Transcode hardware acceleration | Evidence-backed with P1 breadth | `docs/deployment/SELF_HOSTED.md`; playback/transcode architecture docs; release gate candidates. | FFmpeg/ffprobe config, hardware policy (`none`, `vaapi`, `nvenc`, `quick_sync`), CPU fallback/fail-fast, and diagnostics are documented. Per-vendor smoke matrix is P1 unless MVP claims vendor guarantees. | `MRS-030` gates missing FFmpeg and CPU fallback; vendor matrix stays post-MVP. |
| Storage/VFS health | Evidence-backed | `docs/architecture/STORAGE_VFS.md`; remote-storage/circuit workstreams; `crates/nako-server/src/app/tests/storage.rs`. | Local-first storage, WebDAV preview behavior, remote direct play, storage diagnostics, and durable circuit health have evidence. Cache repair and watcher/debounce remain P1. | `MRS-030` includes storage diagnostic smoke. |
| Admin diagnostics and redaction | Evidence-backed | Startup/admin diagnostics tests; release docs; support-bundle/checklist docs. | Storage, scan/job, playback, runtime readiness, and redaction patterns exist. The MVP gap is selecting a focused redaction/diagnostic gate. | `MRS-030` includes admin overview/support/redaction checks. |
| Addon Sidecar foundation | Evidence-backed | `admin-addon-operations-mvp`; `addon-install-guide-generation`; startup addon scan tests; `nako-official-addons` inventory. | Manual registration, status, token/grant boundary, health check, surfaces, resource diagnostics, install guide, and scan metadata sidecar proof exist. Official addon breadth is not P0, but one smoke proof should be named. | `MRS-030` chooses minimum addon smoke; related-repo changes only if that smoke fails. |
| Remote access | Evidence-backed cookbook baseline | `CONTEXT.md`; `docs/deployment/SELF_HOSTED.md`; `CONTROL_PLANE.md`; `network-access-boundary`. | MVP should document reverse proxy, HTTPS, DDNS, Tailscale, or Cloudflare Tunnel patterns. Built-in tunnel/NAT traversal is explicitly P2. | `MRS-030` decides whether current cookbook is sufficient or needs a docs-only blocker. |
| Large-library API scale | Deferred | Public Client route inventory; control-plane/API scale proposals. | List routes are bounded enough for MVP planning. Cache/ETag/cursor breadth is P1 unless live Web smoke proves a required route is unbounded. | Defer unless `MRS-030` gate fails. |
| Realtime updates | Deferred | `REALTIME_SYNC.md`; Web/media workstreams. | Polling is acceptable for MVP. SSE/WebSocket is post-MVP unless a required flow cannot show progress. | Keep out of P0. |
| Desktop/native clients | Deferred | `client-surface-and-access-product-architecture`; ADR 0026. | Desktop playback strategy is important, but MVP can ship browser/web first. `CSAPA-050` should be split or explicitly deferred, not promoted by default. | `MRS-040` records deferral or spike decision. |

## MRS-040 Active Queue Alignment

| Workstream | Current task | MVP classification | Decision | Next action |
| --- | --- | --- | --- | --- |
| `playback-transcode-jellyfin-class-hardening` | `PTJCH-220` | P0 blocker | Playback Runtime ownership is on the required MVP journey. Keep it active and review through `integrate-lane-results` before removing the blocker. | Parallel worker may run in the playback worktree. MVP closeout waits for DONE or an explicit accepted-risk decision. |
| `generated-artifact-metadata-authority-apply` | `GAMA-060` | Conditional/P1 | Backend route through `GAMA-050` is stable. Web Admin apply confirmation is not required for MVP unless the product cut exposes Generated Artifact apply in the first release. Existing lane drift is visible but non-MVP-blocking. | Do not start under the MVP campaign. Reconcile GAMA separately before Web apply work. |
| `client-surface-and-access-product-architecture` | `CSAPA-050` | Deferred/P1 | Desktop playback strategy is valuable, but browser/web is the accepted MVP client path. Missing context on CSAPA does not block the MVP release cut. | Record desktop playback as post-MVP; do not assign until after MVP release blockers are under control. |
| `web-product` live MVP smoke | Not active | P0 gate gap | The Web/Public Client path is MVP-critical, but the missing piece is a deterministic live smoke, not broad Web product development. | Split a small `web-product` workstream in `MRS-050` if manual smoke is not sufficient. |
| `operations-release` gate wrapper | Not active | Optional P0 release-ops gap | The ladder is documented. A one-command wrapper is useful only if the team requires fully scripted release proof before alpha. | Split in `MRS-050` if release management wants a single command beyond the documented ladder. |

## MRS-050 Split Questions

- Which startup path is required for MVP release evidence: local binary,
  Docker/compose, or both?
- What is the exact live Web smoke: list libraries, browse library items,
  item detail, browser playback ticket, video element render, and heartbeat?
- What is the minimum playback gate that proves Direct Play, Remux, HLS
  Transcode, missing FFmpeg diagnostics, and CPU fallback without requiring a
  vendor hardware lab?
- Which Addon Sidecar proof is release-blocking: contract/reference addon only,
  or the `official-addon-e2e-smoke.ps1` path?
- Is the current remote-access cookbook sufficient for P0, or does MVP need a
  small docs-only remote-access closeout task?
