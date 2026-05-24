# Cross-Repo Fearless Boundary Alignment

Status: Active
Last updated: 2026-05-24

## Why This Lane Exists

Nako has already completed several architecture-first lanes:

- server composition and runtime hardening;
- PostgreSQL-ready persistence;
- metadata refresh seams;
- Addon Side Effect and protected-write foundations;
- the official metadata addon runtime, provider registry, provider HTTP
  runtime, side-effect writer, and provider-quality follow-ons.

Those lanes moved the project in the right direction. The next risk is that
the new breadth hardens around overly broad internal Interfaces:

- `NakoDatabase` is backend-neutral, but it is still exposed as a very wide
  delegating facade over nearly every repository trait.
- Library ingestion still mixes Source observation, Local Inference planning,
  search projection, and persistence commit knowledge in one workflow module.
- Metadata provider, NFO, Addon Side Effect, AI-like output, and acceptance
  workflows need one clearer Candidate/Acceptance authority before more write
  paths appear.
- The official metadata scraper has a good sidecar boundary, but
  `MetadataScrapeRuntime` coordinates request decoding, provider fan-out,
  candidate selection, metadata writeback, artwork writeback, and response
  shaping in one module.
- TMDB, Bangumi, and Douban provider modules now contain multiple change axes:
  provider HTTP calls, direct ID lookup, title search strategy, enrichment,
  parser resilience, mapping, degraded candidates, and test fixtures.
- Playback and transcode have useful runtime pieces, but process lifecycle,
  session state, cancellation, hardware/fallback diagnostics, and failure
  taxonomy should converge under one Playback Runtime ownership model before
  adaptive playback or hardware-acceleration APIs grow.

This lane is intentionally cross-repo. The Addon Protocol boundary is where
server architecture and official addon implementation meet, so a server-only
or addon-only refactor would miss the pressure points.

## Relevant Authority

- Nako glossary:
  - `CONTEXT.md`
- ADRs:
  - `docs/adr/0001-modular-monolith-rust-workspace.md`
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0029-postgresql-ready-persistence-boundary.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
  - `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- Related Nako workstreams:
  - `docs/workstreams/future-ready-architecture-refactor/`
  - `docs/workstreams/fearless-architecture-deepening/`
  - `docs/workstreams/fearless-future-architecture-refactor/`
  - `docs/workstreams/addon-architecture-deepening/`
  - `docs/workstreams/metadata-refresh-seam/`
  - `docs/workstreams/metadata-merge-policy-unification/`
  - `docs/workstreams/playback-transcode-ops-hardening/`
  - `docs/workstreams/postgresql-production-readiness/`
- Related official addon workstreams:
  - `../nako-official-addons/docs/workstreams/official-metadata-addon-fearless-refactor/`
  - `../nako-official-addons/docs/workstreams/official-metadata-addon-production-baseline/`
  - `../nako-official-addons/docs/workstreams/official-metadata-addon-side-effect-writer/`
  - `../nako-official-addons/docs/workstreams/official-metadata-addon-provider-relevance-budget/`
  - `../nako-official-addons/docs/workstreams/official-metadata-addon-provider-live-drift-checks/`
- Reference policy:
  - `repo-ref/` repositories are reference material only.
  - Do not copy, port, translate line by line, import schemas, import tests,
    import fixtures, import assets, or import generated code from Jellyfin or
    other reference projects.

## Problem

The project has good high-level crate boundaries, but some Modules are still
too shallow relative to the feature set Nako is aiming for.

The failure mode to avoid is not "large file bad." The failure mode is "one
Interface requires many unrelated changes to be coordinated." That creates
fragile refactors, wider tests, and unclear ownership when adding provider
breadth, AI assistance, Addon Tasks, protected writes, adaptive playback, or
PostgreSQL-backed production operations.

The current highest-risk areas are:

1. Persistence facade width
   - `NakoDatabase` makes backend selection clean, but its backend trait is a
     super-interface over almost every repository.
   - This can turn PostgreSQL parity into "implement the whole world" instead
     of "prove the workflow port this feature needs."
2. Ingestion and Local Inference commit depth
   - Library scan commit code still knows too much about Source observation,
     Local Inference planning, catalog records, evidence, and search
     projection.
3. Metadata acceptance authority
   - NFO, provider refresh, Addon metadata writes, artwork writes, and future AI
     proposals all need a consistent path from candidate evidence to accepted
     Canonical Metadata or Managed Artwork.
4. Official addon scrape runtime depth
   - The official sidecar has the right external boundary, but its internal
     runtime still combines query parsing, provider orchestration, candidate
     selection, writeback coordination, and response shaping.
5. Provider adapter growth
   - TMDB, Bangumi, and Douban adapters are doing too many jobs in one module.
     They need provider-local submodules before more edge-case resilience and
     live-drift logic accumulates.
6. Playback runtime ownership
   - Transcode process execution is isolated, but the product-level Playback
     Runtime should own session lifecycle, cancellation, failure taxonomy,
     hardware diagnostics, and operator-visible state.

## Reference Product Lessons

Jellyfin is useful as a local source-level architecture reference, not as code
to copy. Its mature repository separates API, model/controller abstractions,
providers, local metadata, XBMC/NFO metadata, naming, media encoding, database
implementation, and tests. The Nako lesson is to keep long-lived domain seams
separate even when one server binary composes them.

Plex is proprietary and should not be treated as an implementation reference.
Its useful lesson is product-level: clients need stable server APIs, media
matching must remain operator-correctable, remote access and transcode behavior
must be visible to operators, and the installable server should hide internal
complexity.

Nako should adopt the durable boundary lessons without copying source,
protocols, schemas, plugin compatibility models, or historical constraints.
The Addon Sidecar model remains Nako's own boundary.

## Target State

When this lane closes:

- The server and official addon share one clear architecture map for Addon
  metadata, protected writes, provider candidates, acceptance workflows, and
  host runtime responsibilities.
- `NakoDatabase` remains a backend-neutral composition object, but feature code
  depends on workflow-shaped ports instead of a universal repository facade
  wherever the touched workflow allows it.
- Library ingestion separates Local Inference planning from persistence commit
  enough that future naming rules, duplicate evidence, AI suggestions, and NFO
  authority can evolve independently.
- Metadata/Artwork candidates from providers, NFO, Addons, and future AI
  outputs flow through explicit Candidate/Acceptance vocabulary before becoming
  Canonical Metadata or Managed Artwork.
- The official metadata scraper keeps one installable Addon Sidecar artifact,
  but splits internal scrape runtime responsibilities into deeper Modules.
- TMDB, Bangumi, and Douban provider adapters are decomposed around provider
  client, search strategy, parser/response normalization, mapper, enrichment
  policy, and provider-local fixtures.
- Addon protected-write client responsibilities are aligned with public
  protocol/client crates without forcing private server crates into the addon
  repository.
- Playback Runtime is the owner of transcode session lifecycle and diagnostics;
  `nako-transcode` stays a lower-level execution adapter with a clear Rust API
  for hardware acceleration backends.
- Each refactor slice has focused tests and does not require running the entire
  product to prove a local boundary.

## In Scope

- Cross-repo architecture docs and task routing.
- Narrowing touched server-side workflow ports.
- Persistence facade pressure relief where a workflow slice is touched.
- Library ingestion and Local Inference commit separation.
- Candidate/Acceptance path clarification across metadata, NFO, Addon writes,
  artwork, and future AI-like outputs.
- Official addon scrape runtime module split.
- Official addon provider adapter decomposition for TMDB, Bangumi, and Douban.
- Addon protected-write client/protocol alignment.
- Playback Runtime and transcode ownership clarification.
- Focused nextest gates for touched Rust packages.
- Reference-product analysis limited to architecture boundaries and product
  behavior.

## Out Of Scope

- Copying or translating Jellyfin, Plex, tinyMediaManager, MDCx, Kodi, or other
  reference implementation code.
- Jellyfin plugin compatibility.
- Native in-process plugin ABI.
- Adding new metadata providers only to increase provider count.
- New scraping selectors or live-network behavior unless a provider adapter
  split needs an opt-in drift test.
- Broad Admin Web redesign.
- Network Tunnel Provider implementation.
- Adaptive bitrate ladder implementation.
- AI model provider integration.
- Publishing or versioning a new protocol crate release unless a slice proves a
  public contract change is unavoidable.
- Large schema migrations before a workflow-shaped persistence contract exists.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Addon Sidecars remain the extension model for this phase. | High | ADR 0003, 0015, 0020, 0033 and completed Addon workstreams. | Reopen ADRs before changing plugin strategy. |
| One official metadata sidecar should contain multiple internal providers. | High | Official addon README and completed provider workstreams. | Split install artifacts only when trust, license, resource, or deployment boundaries differ. |
| The universal database facade is the highest-leverage server refactor risk. | High | `nako-db/src/facade.rs` delegates nearly every repository trait through one backend adapter. | If current callers are already narrow enough, focus on workflow ports instead of facade structure. |
| Provider module size reflects multiple change axes, not just file length. | High | TMDB/Bangumi modules combine search, direct lookup, enrichment, mapping, resilience, and tests. | Split by behavior only after tests characterize current semantics. |
| Candidate/Acceptance should be the common write boundary. | Medium | Nako already has Candidate, Selected Artwork, Generated Artifact, Addon Side Effect, and acceptance vocabulary. | If current product flows need direct commits, add an ADR exception before broadening writes. |
| Playback Runtime should own operator-visible transcode state. | High | Existing playback/transcode ops lanes and runtime diagnostics point in that direction. | Keep `nako-transcode` lower-level and avoid duplicating session state there. |
| Cross-repo edits will encounter dirty worktrees. | High | Both repositories already show unrelated modified/untracked files. | Each task must begin with status review and avoid reverting or formatting unrelated changes. |

## Architecture Direction

### 1. Interface-first refactoring

Do not begin by shuffling files. Begin by naming the real behavioral contract:
what operation is requested, what authority owns the decision, what side effects
are allowed, and what evidence proves it. File moves are only useful when they
make that contract easier to hold.

### 2. Workflow-shaped persistence ports

Keep repository traits where they are already useful, but avoid passing a
"database that can do everything" into feature code. Introduce or expose
workflow-shaped ports such as metadata refresh commit, library scan source
commit, protected side-effect apply, playback session store, or job lease
store. Backend adapters can still implement many traits internally, but the
caller should ask for the smallest authority it needs.

### 3. Host-owned acceptance

Providers, NFO parsers, Addon Sidecars, and future AI helpers should produce
evidence-backed candidates. Nako host code owns acceptance, canonical merge
policy, protected writes, audit records, and rollback/remediation semantics.

### 4. Official addon as one artifact with internal deep modules

Keep `nako-metadata-scraper` as one installable Addon Sidecar unless a provider
needs a separate trust, license, resource, or deployment boundary. Internally,
split scrape orchestration from provider details:

- request decoding and query normalization;
- provider fan-out and failure aggregation;
- candidate ranking/dedup/capping;
- writeback coordination and access-check handling;
- response/artifact shaping;
- provider-specific client/search/enrichment/mapper/parser modules.

### 5. Public protocol stays small and stable

Do not make `nako-addon-protocol` carry private server concepts. If the official
addon needs reusable host-client behavior, prefer a narrow public addon client
crate or generated client surface that preserves token redaction, safe errors,
and version tolerance.

### 6. Playback Runtime owns product behavior

`nako-transcode` should expose clear execution APIs for VAAPI, NVENC,
QuickSync, software fallback, probes, and ffmpeg/HLS process control. The
Playback Runtime should own which execution path is used, session lifecycle,
cancellation, durability, diagnostics, and operator-facing failure reasons.

## Closeout Condition

This lane can close when:

- `CRFBA-010` scope is accepted and the dirty-worktree constraints are
  recorded;
- at least one server-side workflow proves a narrower persistence/commit port;
- Library ingestion or Metadata Acceptance has one real deep-module proof;
- the official addon scrape runtime is split without changing public payload
  behavior;
- at least one large provider adapter is split with focused tests preserving
  search, direct lookup, enrichment, degraded candidate, and ranking behavior;
- Addon protected-write client/protocol ownership is clarified and implemented
  or explicitly split;
- Playback Runtime ownership is either deepened or split into a narrower
  follow-on with evidence;
- targeted package gates, relevant workspace gates, review, and fresh
  verification are recorded;
- remaining product breadth is split into named follow-ons instead of hidden in
  architecture work.
