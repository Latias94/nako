# Research: next parallel feature candidates

- Query: Research the next disjoint feature-backed implementation candidate for Nako after the current renderer playback transport flow extraction.
- Scope: internal
- Date: 2026-06-04

## Findings

### Current task boundary

The active task resolver returned no active task, but the prompt gave an exact
task directory and target file. This research uses that explicit path.

`research/renderer-flow-scan.md` says the renderer slice should keep HTTP
renderer ticket/url authoring in `http/renderer.rs`, playback compatibility in
`nako-playback`, Remux startup in `remux_flow`, HLS startup in `hls_flow`, and
avoid API/DTO/schema changes. During this research, `renderer_flow.rs` appeared
in `crates/nako-server/src/app/playback/`, and `mod.rs` now delegates
`start_renderer_playback_session` to it.

Treat these as active/high-conflict files for parallel work:

- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/renderer_flow.rs`
- `crates/nako-server/src/app/playback/hls_flow.rs`
- `crates/nako-server/src/app/playback/remux_flow.rs`

### Files found

- `docs/architecture/LANES.md` - lane ownership and idle next-action queue; no active implementation lane is selected, and metadata, storage, playback, web, and control-plane lanes list candidate follow-ons.
- `docs/architecture/STORAGE_VFS.md` - storage/VFS status; VFS cache action preview and source fingerprint escalation policy are shipped, with executable repair and hash execution follow-ons.
- `docs/architecture/LIBRARY_PIPELINE.md` - library/metadata/artwork status; Douban endpoint depth and artwork delivery cache/placeholder follow-ons remain open.
- `docs/architecture/OPERATIONS_RELEASE.md` - release/ops status; playback release-gate mode shipped, hardware matrix diagnostics remain open.
- `.trellis/tasks/06-03-06-03-long-horizon-architecture-queue/prd.md` - parent queue; records completed 06-04 slices and remaining architecture-backed follow-ons.
- `.trellis/tasks/archive/2026-06/*/task.json` - recent task evidence used instead of git history due the no-git researcher constraint.
- `.trellis/tasks/06-04-playback-renderer-transport-flow-extraction/research/renderer-flow-scan.md` - current renderer extraction scope and conflict boundary.
- `crates/nako-server/src/app/playback/mod.rs` - active renderer entry types and delegation point.
- `crates/nako-server/src/app/playback/renderer_flow.rs` - newly extracted renderer orchestration module.
- `crates/nako-vfs/src/lib.rs` and `crates/nako-vfs/src/cache.rs` - current VFS repair diagnostic/action vocabulary.
- `crates/nako-server/src/app/storage.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/storage.rs` - Admin storage diagnostics mapping and DTO surface.
- `crates/nako-metadata/src/providers/douban.rs`, `crates/nako-metadata/src/mapping/douban.rs`, `crates/nako-metadata/src/tests.rs` - Douban provider endpoint and capability boundary.
- `crates/nako-server/src/app/artwork.rs`, `crates/nako-server/src/app/artwork/variant.rs`, `crates/nako-server/src/http/catalog.rs` - selected artwork image read, variant derivation, cache, and conditional response path.
- `crates/nako-core/src/media/source.rs`, `crates/nako-library/src/ingestion/source_commit.rs` - source fingerprint escalation policy and ingestion planning.
- `scripts/release-gate.sh`, `scripts/release-gate.ps1`, `crates/nako-transcode/src/hardware.rs` - playback release gate and hardware diagnostics seam.

### Code patterns

- `crates/nako-server/src/app/playback/mod.rs:545` defines `RendererPlaybackTransportPlan`; `mod.rs:588` defines `StartRendererPlaybackSessionRequest`; `mod.rs:808` exposes `PlaybackAppService::start_renderer_playback_session`; `mod.rs:812` delegates to `renderer_flow::start_renderer_playback_session`.
- `crates/nako-server/src/app/playback/renderer_flow.rs:13` is the active extracted renderer flow; `renderer_flow.rs:36`, `renderer_flow.rs:59`, and `renderer_flow.rs:128` branch Direct Play, Remux, and HLS transport planning.
- `docs/architecture/LANES.md:21` says no active implementation lane is selected; `LANES.md:31-35` lists idle metadata, playback, storage, web, and control-plane next actions.
- `docs/architecture/STORAGE_VFS.md:33` marks VFS cache diagnostics plus action preview shipped and points to executable refresh/remediation actions; `STORAGE_VFS.md:94` names `proposed:vfs-cache-repair-operator-actions`.
- `crates/nako-vfs/src/lib.rs:175` defines `VfsCacheRepairAction`; `lib.rs:277` maps classifications to actions; `crates/nako-server/src/app/storage.rs:286` exposes `latest_vfs_cache_repair_diagnostic`; `crates/nako-api/src/admin/storage.rs:96` mirrors the Admin action enum.
- `docs/architecture/LIBRARY_PIPELINE.md:33` keeps Douban endpoint-backed TV/episode depth open; `crates/nako-metadata/src/providers/douban.rs:91` reports `supports_hierarchy: false`; `douban.rs:104` and `douban.rs:155` reject non-movie search/fetch before HTTP; `tests.rs:4142` covers that rejection.
- `docs/architecture/LIBRARY_PIPELINE.md:36` keeps metadata-only ETag preflight, placeholders, and derivative policy open; `.trellis/spec/nako-server/backend/http-api-patterns.md:474` explicitly excludes metadata-only ETag preflight from the shipped selected-artwork cache-control slice.
- `crates/nako-server/src/http/catalog.rs:117` and `catalog.rs:139` call `read_selected_image` for GET and HEAD; `catalog.rs:452` builds the shared selected image response; `crates/nako-server/src/app/artwork.rs:933` reads selected image bytes; `artwork.rs:968` derives original/variant bytes and ETag.
- `docs/architecture/STORAGE_VFS.md:30` marks source fingerprint escalation policy shipped but hash execution open; `crates/nako-core/src/media/source.rs:64` defines `PartialHash` and `FullHash`; `source.rs:574` and `source.rs:604` test advisory escalation decisions; `crates/nako-library/src/ingestion/source_commit.rs:199` carries the decision into ingestion planning.
- `docs/architecture/OPERATIONS_RELEASE.md:27` marks playback release-gate mode shipped; `OPERATIONS_RELEASE.md:30` keeps per-host FFmpeg/hardware smoke matrix open; `scripts/release-gate.sh:110` runs only `ffmpeg -version`, `ffprobe -version`, and self-host playback smoke; `scripts/release-gate.ps1:192` mirrors those checks.
- `.trellis/spec/nako-server/backend/http-api-patterns.md:42` requires HTTP response wire types to come from `nako-api`; `.trellis/spec/nako-server/backend/quality-guidelines.md:16` requires bounded/paginated list surfaces; `.trellis/spec/nako-api/backend/admin-and-public-contracts.md:90` documents the Admin VFS cache repair preview contract; `.trellis/spec/nako-vfs/backend/index.md:13` points repair work at storage failure/error handling guidance.

### Ranked candidates

| Rank | Candidate | Target files/crates | Why user-visible or architecture-backed | Likely tests/gates | Overlap risk with renderer flow | Recommended first slice |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | VFS cache repair executable refresh action | `crates/nako-vfs/src/lib.rs`, `crates/nako-vfs/src/cache.rs`, `crates/nako-server/src/app/storage.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/storage.rs`; generated Admin Web contract only if DTO/route shape changes. | Operators already see a redaction-safe repair preview; architecture now explicitly asks for executable refresh/remediation. This turns diagnostics into a recovery feature without touching playback. | `cargo check -p nako-vfs -p nako-api -p nako-server --tests`; `cargo nextest run -p nako-vfs vfs_cache --no-fail-fast`; `cargo nextest run -p nako-api admin_storage --no-fail-fast`; `cargo nextest run -p nako-server storage --no-fail-fast`; `cargo fmt --all -- --check`; generated Admin contract gate if DTOs change. | Low. Avoids all current renderer/HLS/Remux flow files. Coordinate only if another task edits `http/admin.rs` or Admin storage DTOs. | Add one Admin-only action path for `RefreshCache` on a URI/latest-failure scoped repair target, returning a redacted result. Keep `FixBackendConfiguration` and `InspectFailure` plan-only; no purge/delete, durable job, schema change, or Web UI in the first slice. |
| 2 | Douban TV/episode endpoint depth first slice | `crates/nako-metadata/src/providers/douban.rs`, `crates/nako-metadata/src/mapping/douban.rs`, `crates/nako-metadata/src/tests.rs`; possibly provider fixtures only. | Long-horizon queue and library architecture repeatedly name `douban-tv-episode-endpoint-depth`. Current Douban code rejects Series/Season/Episode before HTTP, so endpoint-backed support is a real metadata feature. | `cargo check -p nako-metadata --tests`; `cargo nextest run -p nako-metadata douban --no-fail-fast`; broaden to `cargo check -p nako-core -p nako-metadata --tests` if provider subject/candidate graph types change. | Very low. Stays in `nako-metadata`; no playback app files, API, schema, or generated contracts in the recommended slice. | Implement endpoint-backed Series search/fetch capability with mocked HTTP fixtures and honest capability claims. Keep Season/Episode child graph preview or accepted hierarchy application out unless the endpoint contract is verified and covered. |
| 3 | Selected artwork metadata-only ETag preflight | `crates/nako-server/src/app/artwork.rs`, `crates/nako-server/src/app/artwork/variant.rs`, `crates/nako-server/src/http/catalog.rs`, `crates/nako-server/src/http/tests/addons.rs`; possibly `nako-api` only if public docs/tests need contract updates. | Control-plane and library maps keep metadata-only ETag preflight and broader delivery cache policy open. Current HEAD/conditional responses still read and derive image bytes before returning headers/304, so this improves client performance for real artwork routes. | `cargo check -p nako-server --tests`; `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`; `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`; `cargo fmt --all -- --check`. | Low. Uses catalog/artwork routes and services only. No overlap with playback renderer flow. | Add metadata-only 304/HEAD support for original selected artwork using selected/artifact metadata and the existing safe ETag formula before `artifact_store.read`. Keep resized variants on the existing byte-read path until variant dimensions and cached derivatives are specified. |
| 4 | Source fingerprint partial-hash execution | `crates/nako-core/src/media/source.rs`, `crates/nako-library/src/scan.rs`, `crates/nako-library/src/ingestion/source_commit.rs`, likely VFS-backed read helpers and scan tests; avoid DB/schema in first slice if possible. | Storage architecture says the policy seam is shipped but hash execution, operator queueing, and diagnostics remain. Current code only recommends `PartialHash`/`FullHash`; it does not execute hash reads. This improves duplicate/move reconciliation for weak evidence. | `cargo check -p nako-core -p nako-library -p nako-vfs --tests`; `cargo nextest run -p nako-core source_fingerprint --no-fail-fast`; `cargo nextest run -p nako-library source_commit --no-fail-fast`; add focused VFS read/hash tests if introduced. | Low to medium. No playback renderer files, but it coordinates with storage/VFS and scan/probe lanes. | Implement bounded partial-hash execution for the single-weak-candidate case only, with a small configured byte budget and redacted diagnostics. Do not add full-file hashing, operator queue, or schema changes in the first slice. |
| 5 | Playback release hardware diagnostics matrix | `scripts/release-gate.sh`, `scripts/release-gate.ps1`, `docs/deployment/RELEASE_CHECKLIST.md`, `docs/deployment/SELF_HOSTED.md`, `crates/nako-transcode/src/hardware.rs`; possibly Admin playback diagnostics later. | Operations architecture says playback gate mode shipped but per-host FFmpeg/hardware smoke matrix and container device pass-through evidence remain. This is operator-visible release readiness, not mechanical cleanup. | `cargo check -p nako-transcode --tests`; `cargo nextest run -p nako-transcode hardware --no-fail-fast`; script dry-runs for both shell variants where available; docs checks if release-gate docs change. | Low. Does not need `nako-server/src/app/playback/*`. It may touch playback-themed docs/scripts but not renderer transport flow. | Add an optional hardware diagnostics mode/report that reuses existing `nako-transcode` hardware capability diagnostics and does not require a GPU to pass. Keep actual container/device smoke as documented optional evidence. |

### Recommendation

Choose **VFS cache repair executable refresh action** as the next parallel
feature-backed implementation candidate.

Reasons:

- It is explicitly architecture-backed after the action-preview slice shipped.
- It is operator-visible: diagnostics become a bounded recovery action.
- It has the lowest overlap with the current renderer transport extraction.
- The first slice can remain narrow: one `RefreshCache` action, redacted result,
  no schema, no durable job, no playback route changes.

If the implementation terminal must avoid `nako-server/src/http/admin.rs` due
another Admin API task, choose **Douban TV/episode endpoint depth first slice**
instead; it is the cleanest crate-disjoint fallback.

### Related specs

- `.trellis/spec/guides/index.md`
- `.trellis/spec/guides/cross-layer-thinking-guide.md`
- `.trellis/spec/guides/code-reuse-thinking-guide.md`
- `.trellis/spec/nako-server/backend/index.md`
- `.trellis/spec/nako-server/backend/http-api-patterns.md`
- `.trellis/spec/nako-server/backend/quality-guidelines.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-vfs/backend/index.md`
- `.trellis/spec/nako-metadata/backend/index.md`
- `.trellis/spec/nako-metadata/backend/quality-guidelines.md`
- `.trellis/spec/nako-library/backend/index.md`
- `.trellis/spec/nako-transcode/backend/index.md`

### External references

No external references were consulted. This pass intentionally stayed internal:
architecture maps, Trellis specs, current code, current task research, and
archived Trellis task evidence were enough to rank parallel candidates. The
Douban candidate should verify current official endpoint behavior before
implementation changes capability claims.

## Caveats / Not Found

- `python3 ./.trellis/scripts/task.py current --source` returned no active
  task. The output file path came from the prompt, not task resolver state.
- Git history was not inspected because the Trellis researcher scope forbids
  any git operation. "Recent" evidence was inferred from 2026-06 archived
  Trellis task files and architecture docs updated on 2026-06-04.
- The workspace changed while reading: `renderer_flow.rs` appeared after the
  initial playback module listing. This reinforces the recommendation to avoid
  `mod.rs`, `renderer_flow.rs`, `hls_flow.rs`, and `remux_flow.rs`.
- `VFS cache repair executable refresh action` may need Admin DTO/contract
  generation if it adds a new route or response shape. Keep the first slice
  plan/result DTO small and redaction-safe.
- The Douban candidate is intentionally ranked as a disjoint fallback, but its
  exact endpoint design needs external API verification before implementation.
