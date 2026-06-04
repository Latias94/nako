# Library / Metadata / Catalog / NFO Architecture Findings

Date: 2026-06-04
Scope: read-only architecture inspection for a 10-hour media-server improvement
campaign. No Rust code changes were made.

## Evidence Read

- `CONTEXT.md`: Nako glossary for Media Library, Media Source, Local Inference,
  Provisional Hierarchy, Hierarchy Confirmation, Canonical Metadata, NFO Import,
  NFO Export, NFO Round Trip, Managed Artwork, Browse Facet, Sort Key.
- `docs/architecture/LIBRARY_PIPELINE.md:8-20`: target chain from stable file
  candidate through metadata, catalog projection, artwork derivatives, and
  client delivery.
- `docs/architecture/LIBRARY_PIPELINE.md:24-37`: progress matrix showing
  watcher/debounce, artwork delivery, related hierarchy application, and NFO
  polish as remaining lanes.
- `docs/architecture/WORKSTREAM_LINKS.md:157-217`: historical evidence and
  proposed lanes for library, metadata, NFO, and artwork.
- ADRs: `docs/adr/0007-metadata-merge-policy-and-local-authority.md`,
  `docs/adr/0008-nfo-as-local-metadata-boundary.md`,
  `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`,
  `docs/adr/0012-durable-scan-state-and-source-tombstones.md`,
  `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`,
  `docs/adr/0021-video-first-media-server-domain-model.md`,
  `docs/adr/0035-addon-native-metadata-writeback.md`.
- Specs: `.trellis/spec/nako-library/backend/index.md`,
  `.trellis/spec/nako-metadata/backend/index.md`,
  `.trellis/spec/nako-catalog/backend/index.md`,
  `.trellis/spec/nako-nfo/backend/index.md`.

## Ranked Opportunities

### 1. Productize Watch Folder Stable Intake Runtime

Type: feature development with a small interface-deepening refactor.

User-visible value: users copying large media files into a watched Media
Library should not get premature probe/scan failures, and the admin surface
should explain whether realtime monitoring is actually active.

Problem: the stable-candidate kernel exists, and the server runtime polls it,
but the architecture map still marks watcher/debounce weak. The runtime uses a
fixed 5s tick and enqueues a full library scan whenever newly ready candidates
exist; that is useful but still shallow as a product boundary.

Evidence:

- `docs/architecture/LIBRARY_PIPELINE.md:26-37` calls durable scan/source
  tombstones shipped, but watcher/debounce weak and needing product integration.
- `crates/nako-library/src/intake.rs:3-51` has the stable observation kernel
  with `STABLE_INTAKE_REQUIRED_OBSERVATIONS = 2`.
- `crates/nako-server/src/app/watch_folder_runtime.rs:91-141` starts per-library
  polling loops under the runtime supervisor.
- `crates/nako-server/src/app/watch_folder_runtime.rs:150-195` discovers watch
  candidates and enqueues a library scan when `newly_ready_candidates > 0`.
- `crates/nako-server/src/app/watch_folder_runtime.rs:216-280` has redaction-safe
  coverage diagnostics for started, disabled, unsupported, and missing roots.

Recommended work:

- Deepen `WatchFolderRuntimeAppService::tick_library` into a small
  "watch-folder intake plan" interface: discover, suppress, enqueue decision,
  redaction-safe summary.
- Add a read-only admin diagnostic route or extend the existing operations read
  model with coverage and last-tick facts.
- Keep OS watcher daemon work out of the 10-hour goal; polling productization
  is enough.

Risk: medium. It touches scan scheduling and can create duplicate jobs if the
idempotency/admission rule is loose.

Parallelizability: high. Mostly `nako-library` + `nako-server` app/http tests,
with little overlap with metadata/artwork/NFO lanes.

Serial dependencies: decide whether the admin diagnostic response belongs under
system/operations or library admin routes before implementation.

Likely gates:

- `cargo nextest run -p nako-library intake --no-fail-fast`
- `cargo nextest run -p nako-server watch_folder --no-fail-fast`
- `cargo nextest run -p nako-server library --no-fail-fast`
- `cargo check -p nako-library -p nako-server --tests`

### 2. Surface Provider Review Related Hierarchy Application

Type: feature development, mostly route/application wiring.

User-visible value: when provider graph previews know series -> season or
season -> episode relationships, an admin can confirm the related hierarchy
instead of only accepting the root Provider Mapping. This moves Nako closer to a
usable TV/anime library workflow.

Problem: the core `nako-metadata` service already has a safe related hierarchy
application interface, but the server application and HTTP surfaces currently
focus on root review planning/application and batch root apply.

Evidence:

- `docs/architecture/LIBRARY_PIPELINE.md:31-34` says provider graph previews are
  shipped but accepted-review application or Admin/Web governance is needed
  before preview graph depth becomes accepted hierarchy.
- `.trellis/spec/nako-metadata/backend/directory-structure.md:29-32` says
  Candidate Review governance separates build plan, decide, root mapping apply,
  and related hierarchy apply.
- `crates/nako-metadata/src/candidate_review.rs:390-483` implements
  `apply_related_hierarchy` with accepted root checks and provisional state
  confirmation.
- `crates/nako-metadata/src/candidate_review.rs:524-703` rejects unsafe,
  ambiguous, non-root-anchored, or unsupported related hierarchy shapes.
- `crates/nako-server/src/app/metadata.rs:504-612` builds/list root application
  plans but does not expose related hierarchy application.
- `crates/nako-server/src/http/metadata.rs:52-83` exposes refresh, attempts,
  raw responses, candidate search, maintenance, and cleanup routes, but no
  related hierarchy apply route.

Recommended work:

- Add an Admin-only plan/apply route for related hierarchy application, reusing
  the `nako-metadata` service rather than duplicating checks in `nako-server`.
- Keep it separate from existing root apply and batch root apply. Do not
  silently apply related hierarchy as a side effect of accepting the root.
- Add response counters for confirmed items, provider subjects, provider
  mappings, and changed/noop.

Risk: medium-high. It touches hierarchy state and Admin contracts; however the
mutation kernel is already tested inside `nako-metadata`.

Parallelizability: medium. It can run beside watcher/artwork/NFO, but should
not run in parallel with another worker editing `crates/nako-server/src/app/metadata.rs`,
Admin metadata DTOs, or generated API contracts.

Serial dependencies: route/DTO shape approval; then implementation. If public
client exposure is desired, that is a separate follow-on.

Likely gates:

- `cargo nextest run -p nako-metadata candidate_review_related_hierarchy --no-fail-fast`
- `cargo nextest run -p nako-server metadata_candidate --no-fail-fast`
- `cargo check -p nako-metadata -p nako-api -p nako-server --tests`

### 3. Artwork Delivery Cache Placeholder / Derivative Persistence

Type: performance-oriented feature development with cache interface design.

User-visible value: catalog pages feel faster because selected artwork can be
served as stable small variants and placeholders instead of re-decoding and
resizing source artifacts on every request.

Problem: selected artwork currently has conditional GET and variant generation,
but variant bytes appear to be derived on demand from the original artifact.
That is acceptable as a baseline, but it pushes CPU and latency into client
image reads.

Evidence:

- `docs/architecture/LIBRARY_PIPELINE.md:36` says selected artwork private
  cache-control and exact conditional GET are shipped, with metadata-only ETag,
  placeholders, and broader derivative policy remaining.
- `docs/architecture/WORKSTREAM_LINKS.md:209-213` lists
  `proposed:artwork-delivery-cache-placeholder`.
- `crates/nako-server/src/http/catalog.rs:452-510` implements selected artwork
  response, exact ETag match, `304 Not Modified`, and private cache-control.
- `crates/nako-server/src/app/artwork/variant.rs:146-198` decodes and resizes
  variants, always encoding resized variants as PNG.
- `crates/nako-server/src/app/artwork.rs:567-605` already has gallery and
  lifecycle diagnostics that can expose derivative/cache readiness later.

Recommended work:

- Start with metadata-only ETag preflight or a small derivative metadata cache;
  avoid a full binary derivative store unless the API/design is approved.
- Add a placeholder/blurhash evidence field only if the storage and DTO contract
  are clear. Otherwise deliver a documented derivative cache plan and one
  focused server-side cache slice.
- Preserve selected artwork cache invalidation based on selected id, artifact
  id, updated time, and variant dimensions.

Risk: medium. Bad cache invalidation makes stale artwork visible; binary cache
storage can expand quickly.

Parallelizability: medium-high. It mostly touches artwork app/http/tests and
does not need metadata or NFO changes.

Serial dependencies: decide whether derivative bytes are persisted in the
existing managed artwork artifact store or kept as an in-process/read-through
cache.

Likely gates:

- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `cargo nextest run -p nako-server artwork --no-fail-fast`
- `cargo check -p nako-server --tests`

### 4. Split / Deepen `nako-catalog` Hydration Internals

Type: refactor-only.

User-visible value: indirect but important: safer catalog graph/search changes,
fewer regressions when adding browse facets, people, collections, provider
subjects, and image projections.

Problem: `nako-catalog/src/lib.rs` has grown into a large orchestration module
that loads snapshots, performs lookup, builds graph replacements, and builds
search projection in one file. The spec currently permits staying in `lib.rs`
only until groups stop being readable.

Evidence:

- `.trellis/spec/nako-catalog/backend/directory-structure.md` says split only
  when graph builders or projection helpers become large enough, and keep
  search ranking out of this crate.
- `crates/nako-catalog/src/lib.rs:96-183` loads a hydration snapshot with many
  repository calls.
- `crates/nako-catalog/src/lib.rs:185-294` loads lookup data for people, genres,
  tags, collections, studios, and image assets.
- `crates/nako-catalog/src/lib.rs:319-372` plans a full graph replacement and
  search projection.
- `crates/nako-catalog/src/lib.rs:1238-1616` contains extensive tests in the
  same file.

Recommended work:

- Split private modules: `snapshot`, `lookup`, `graph`, `search_projection`,
  and `tests` or `fixtures`.
- Keep the public `CatalogHydrationPort` and exported functions stable.
- Do not change SQL, schema, DTOs, or search ranking in this refactor.

Risk: low-medium. Mostly mechanical, but broad movement can conflict with
parallel catalog feature work.

Parallelizability: low if another worker touches catalog; high otherwise.

Serial dependencies: run after any campaign task that changes catalog behavior,
or reserve catalog ownership for one worker.

Likely gates:

- `cargo nextest run -p nako-catalog --no-fail-fast`
- `cargo check -p nako-catalog -p nako-search -p nako-api --tests`

### 5. Implement Metadata Refresh Validation-Only Mode

Type: feature development.

User-visible value: admins can preview provider refresh effects without
mutating Canonical Metadata, Provider Mapping, catalog graph, or search
projection. This is useful before bulk metadata maintenance.

Problem: metadata profiles already have a `ValidationOnly` mode in the domain
model, but the strategy rejects it as unsupported.

Evidence:

- ADR 0007 requires metadata refresh to be idempotent and explainable with
  local authority and field locks.
- `crates/nako-metadata/src/strategy.rs:429-439` explicitly rejects
  `MetadataRefreshMode::ValidationOnly` as not implemented.
- `crates/nako-server/src/app/metadata_application.rs:21-60` already plans
  merge and catalog projection without committing inside this module, so a dry
  run could reuse the planning path if carefully bounded.

Recommended work:

- Add a validation-only result shape that reports candidate provider, merge
  delta summary, lock effects, and would-write projection counts without
  committing.
- Keep raw provider attempt persistence policy explicit: either record attempts
  as diagnostics or keep validation fully non-mutating; do not mix silently.

Risk: medium. Dry-run semantics often leak hidden writes unless the write
boundary is explicit.

Parallelizability: medium. It touches metadata strategy and possibly server
metadata DTOs; avoid running beside opportunity 2 if both change metadata HTTP
or app contracts.

Serial dependencies: define whether provider attempts/raw cache are allowed in
validation mode.

Likely gates:

- `cargo nextest run -p nako-metadata strategy --no-fail-fast`
- `cargo nextest run -p nako-server metadata --no-fail-fast`
- `cargo check -p nako-metadata -p nako-server --tests`

### 6. Add Episode / Series NFO Export Coverage

Type: feature development with codec-policy work.

User-visible value: TV/anime users get local metadata writeback closer to other
self-hosted media servers, not movie-only NFO export.

Problem: import already understands episode hierarchy enough to confirm a
provisional hierarchy in place, and codec parsing recognizes `episodedetails`,
but export currently skips non-movie items.

Evidence:

- ADR 0008 treats NFO as a local metadata boundary and starts with a minimal
  movie NFO codec.
- `crates/nako-nfo/src/codec.rs:434-445` maps `<movie>` and
  `<episodedetails>` roots and extracts series/season/episode hierarchy facts.
- `crates/nako-nfo/src/import.rs:341-395` confirms episode hierarchy by
  updating series, season, and episode in place.
- `crates/nako-nfo/src/export.rs:203-205` skips NFO export for any item kind
  that is not `MediaKind::Movie`.
- `crates/nako-nfo/src/preview.rs:259-269` tells users NFO export currently
  supports movie sidecars only.

Recommended work:

- If chosen for the 10-hour goal, implement a narrow episode export preview and
  render path for `episodedetails` only. Do not attempt full series/season
  metadata sidecar policy in the same slice.
- Preserve existing NFO Round Trip behavior and backup policy.

Risk: medium-high. NFO formats are compatibility-heavy and easy to over-expand.

Parallelizability: high if isolated to `nako-nfo`; low if it also changes
server NFO admin DTOs.

Serial dependencies: decide whether first slice exports episode sidecars only
  or also series/season sidecars.

Likely gates:

- `cargo nextest run -p nako-nfo --no-fail-fast`
- `cargo nextest run -p nako-server nfo --no-fail-fast`
- `cargo check -p nako-nfo -p nako-server --tests`

### 7. Bound Probe Work by Streamed Pages / Cancellation Checkpoints

Type: refactor and reliability improvement.

User-visible value: large libraries and remote storage probes use less memory
and respond better to cancellation/failure without waiting for a full source
list to load.

Problem: probe orchestration loads all Media Sources into a vector before
probing. That is simple, but it scales poorly and makes cancellation/admission
harder to apply per page.

Evidence:

- `crates/nako-library/src/probe.rs:154-162` loads all sources and buffers
  unordered probe tasks.
- `crates/nako-library/src/probe.rs:193-221` pages through all media sources
  into a single vector before returning.
- `crates/nako-library/src/probe.rs:224-246` persists probe failures with
  classified retryability, so page-level work can preserve current failure
  semantics.

Recommended work:

- Add page-by-page probe execution with the same bounded concurrency per page.
- Add optional cancellation checkpoints only if the existing job runtime can
  pass cancellation into `nako-library` cleanly; otherwise keep this as a
  refactor-only streamed pagination task.

Risk: medium. Ordering and summary counts must stay deterministic.

Parallelizability: high. It lives mostly in `nako-library`.

Serial dependencies: if cancellation is included, coordinate with control-plane
job runtime ownership.

Likely gates:

- `cargo nextest run -p nako-library probe --no-fail-fast`
- `cargo nextest run -p nako-server nfo --no-fail-fast` only if job
  cancellation patterns are reused
- `cargo check -p nako-library --tests`

### 8. Douban TV / Episode Endpoint Depth

Type: provider feature development.

User-visible value: better Chinese metadata coverage for TV/anime-like media
when Douban data is preferred.

Problem: Douban is intentionally movie-only today. This is valuable but less
parallel-safe than watcher/artwork because it requires provider endpoint
research, mapping, capability claims, and tests.

Evidence:

- `docs/architecture/LIBRARY_PIPELINE.md:33` names Douban TV/episode endpoint
  depth as a follow-on.
- `docs/architecture/WORKSTREAM_LINKS.md:214` lists
  `proposed:douban-tv-episode-endpoint-depth`.
- `crates/nako-metadata/src/providers/douban.rs:91-95` declares no hierarchy
  support and movie endpoint notes.
- `crates/nako-metadata/src/providers/douban.rs:103-158` rejects non-movie
  search/fetch requests.

Recommended work:

- Treat as a separate research/feature task unless the 10-hour goal is
  explicitly provider-focused.
- Do not combine with related hierarchy application in the same worker unless
  the route/DTO contracts are frozen.

Risk: high. External provider behavior and payload shapes can drift.

Parallelizability: medium. Provider code is isolated, but tests and capability
claims overlap with metadata route diagnostics.

Serial dependencies: provider endpoint research and capability-claim decision.

Likely gates:

- `cargo nextest run -p nako-metadata douban --no-fail-fast`
- `cargo nextest run -p nako-server metadata --no-fail-fast`

## 10-Hour Campaign Shape

Recommended mode: PLAN -> ASSIGN after this artifact is reviewed. Do not start
with schema/API churn. The highest-return campaign is three feature slices plus
one refactor/verification lane.

### Hour 0.0-0.5: Commander Preflight

- Confirm no one else is editing `nako-api`, `nako-server/src/app/metadata.rs`,
  `nako-catalog/src/lib.rs`, or artwork/NFO files.
- Decide whether related hierarchy application may add Admin API/contract
  surface in this goal. If no, replace it with catalog split or probe streaming.
- Open focused Trellis implementation tasks if the campaign is approved.

### Hour 0.5-1.5: Parallel Design Contracts

Worker A: Watch folder runtime productization.

- Owns: `crates/nako-library/src/intake.rs`,
  `crates/nako-server/src/app/watch_folder_runtime.rs`, related app/http tests.
- Must not touch metadata/artwork/NFO.

Worker B: Provider related hierarchy Admin wiring.

- Owns: `crates/nako-metadata/src/candidate_review.rs` only if missing helper
  behavior is found, otherwise `nako-api`/`nako-server` metadata app/http tests.
- Must not change root batch apply semantics.

Worker C: Artwork delivery cache preflight / derivative plan.

- Owns: `crates/nako-server/src/app/artwork*`,
  `crates/nako-server/src/http/catalog.rs`, catalog HTTP tests.
- Must not add schema unless explicitly approved.

Worker D: Refactor/verification lane.

- Owns either `nako-catalog` module split or `nako-library` probe streaming,
  not both.
- Must stay refactor-only unless the commander promotes it.

### Hour 1.5-6.5: Implementation Wave

Run A, B, and C in parallel if Admin API changes for B are approved. Keep D as
refactor-only or verification support if thread capacity is limited.

Recommended ordering:

1. A starts immediately. It is independent and high user value.
2. C starts immediately if derivative persistence is not required. If binary
   cache persistence is required, pause for design.
3. B starts after a 15-minute route/DTO shape review because it touches shared
   Admin contracts.
4. D starts only if there is no concurrent catalog behavior work.

### Hour 6.5-8.0: Integration / Conflict Window

- Merge or integrate A and C first; they are lower shared-contract risk.
- Integrate B after generated/admin contract checks.
- If D is a catalog split, integrate it last to reduce merge noise.

### Hour 8.0-9.5: Quality Gates

Minimum combined gates:

- `cargo check -p nako-library -p nako-metadata -p nako-catalog -p nako-nfo -p nako-server --tests`
- `cargo nextest run -p nako-library intake --no-fail-fast`
- `cargo nextest run -p nako-server watch_folder --no-fail-fast`
- `cargo nextest run -p nako-metadata candidate_review_related_hierarchy --no-fail-fast`
- `cargo nextest run -p nako-server metadata --no-fail-fast`
- `cargo nextest run -p nako-server catalog --no-fail-fast`
- `cargo nextest run -p nako-nfo --no-fail-fast` if NFO is selected
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate ./.trellis/tasks/<task-dir>`

### Hour 9.5-10.0: Close / Split Follow-ons

- Update Trellis spec only for new durable contracts: watch-folder runtime plan,
  related hierarchy Admin route, artwork derivative/cache contract, or NFO
  episode export policy.
- Commit implementation and task/archive changes separately.
- Split unfinished high-risk work into follow-on Trellis tasks instead of
  stretching the 10-hour goal.

## Recommended 10-Hour Goal

If the user wants maximum visible product progress:

1. Ship Watch Folder Stable Intake Runtime productization.
2. Ship Provider Review Related Hierarchy Admin application if API surface is
   approved; otherwise do `nako-catalog` split.
3. Ship selected artwork metadata-only ETag/preflight or first derivative cache
   slice without schema.
4. Use the remaining lane for focused refactor/check support, not a fourth
   feature.

If the user wants lowest risk:

1. Watch Folder Stable Intake Runtime productization.
2. `nako-catalog` hydration module split.
3. `nako-library` probe streamed-page refactor.
4. Artwork cache preflight only, no persistence.

## Stop Conditions

- Any schema migration need not already covered by an ADR.
- Public Client API contract changes.
- Two workers need to edit `crates/nako-server/src/app/metadata.rs` at the same
  time.
- Provider endpoint behavior cannot be proven with local fixtures.
- Watch-folder changes enqueue duplicate scans under a repeated stable
  candidate test.
- Artwork cache invalidation cannot be expressed in a stable ETag contract.
