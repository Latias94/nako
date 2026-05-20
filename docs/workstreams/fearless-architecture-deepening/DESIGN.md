# Fearless Architecture Deepening

Status: Completed
Last updated: 2026-05-20

## Why This Lane Exists

Taru is past the point where the main risk is missing CRUD or missing crates.
M61 and M62 left a strong modular monolith baseline, explicit PostgreSQL
runtime selection, backend-neutral contract tests, a clearer public/admin/addon
API split, and mature workstream practice.

The next risk is different: a few high-leverage Modules are becoming shallow or
too broad while Taru is still early enough to refactor fearlessly. If these
Modules harden before deeper seams exist, future provider breadth, Addon
Sidecars, NFO/library-file writes, playback profiles, AI automation, and remote
access features will spread ordering, permissions, redaction, and persistence
rules across callers.

This lane records the 2026-05-20 architecture review findings and turns them
into an execution plan for the next fearless refactor pass.

## Relevant Authority

- Glossary and domain model:
  - `CONTEXT.md`
- ADRs:
  - `docs/adr/0001-modular-monolith-rust-workspace.md`
  - `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0029-postgresql-ready-persistence-boundary.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- Related workstreams:
  - `docs/workstreams/future-ready-architecture-refactor/`
  - `docs/workstreams/postgresql-production-readiness/`
  - `docs/workstreams/addon-protected-writes/`
  - `docs/workstreams/addon-library-file-write-policy/`
  - `docs/workstreams/addon-managed-artwork-artifacts/`
  - `docs/workstreams/metadata-catalog-commit-atomicity/`
  - `docs/workstreams/repository-seam-deepening/`
  - `docs/workstreams/transcode-runtime/`
  - `docs/workstreams/playback-source-selection-deepening/`
  - `docs/workstreams/managed-artwork-postgresql-parity/`

## Problem

The review found strong architecture direction, but also several Modules whose
Interfaces are not deep enough for the behavior now living behind them:

1. `crates/taru-server/src/app/addons.rs` now concentrates Addon registration,
   token/grant handling, Addon Principal resolution, Addon Side Effect intake,
   Canonical Metadata writes, Library File Write/NFO export, artwork candidate
   intake, storage access, catalog refresh, and search refresh. This is a
   locality warning: permissions, side effects, redaction, and persistence
   ordering are all changing in one Module.
2. Addon-initiated Canonical Metadata writes are still too close to an
   app-level sequence: update metadata, refresh catalog/search, then report the
   apply outcome. That ordering needs a deeper commit seam so rollback and
   projection consistency are proven by the Interface, not remembered by
   callers.
3. Library ingestion has good scanning and Local Inference separation, but
   `LibraryIndexService` still depends on a broad repository trait alias and
   owns scan snapshot, Source State, Library Item State, evidence, ingestion
   failure, and Search Projection orchestration. The scan commit path is
   proven, but the caller-facing Interface still exposes too much knowledge.
4. Playback/transcode has a useful profile identity foundation, but multi-profile
   HLS reuse and richer source selection need explicit request/cache identity
   semantics before adaptive ladders, subtitles, HDR/SDR variants, or
   device-specific profile reuse are added.
5. Hardware acceleration diagnostics still need a deeper runtime seam than
   encoder-list probing. Operators need evidence that VAAPI, NVENC, and
   QuickSync devices can actually initialize under the current host/container.
6. Search has a projection shape, but search quality and future AI/semantic
   integration need a measured query semantics lane before vector or AI features
   hide traditional search gaps.
7. Several rich test families live in very large files. The coverage is
   valuable, but fixture locality and failure navigation need improvement as
   the next refactors touch those Interfaces.

## Target State

When this lane closes:

- Addon Side Effect handling is split into deeper Modules with small caller
  Interfaces:
  - Addon Principal and grant resolution;
  - Addon Side Effect intake/idempotency;
  - Addon Side Effect apply routing;
  - domain-specific apply Adapters for Canonical Metadata, Library File Write,
    and artwork candidate behavior.
- Addon Canonical Metadata writes have a transactional commit seam that covers
  metadata mutation, Catalog Item Graph/Search Projection consistency, apply
  outcome recording, and rollback behavior.
- Library ingestion callers no longer need a broad repository trait alias to
  coordinate scan/source/evidence/search behavior. A workflow-shaped seam owns
  the commit ordering.
- Playback Source Selection identity and Transcode Profile identity produce a
  stable request/cache identity before multi-profile HLS reuse is widened.
- Hardware capability diagnostics distinguish static FFmpeg encoder discovery
  from device initialization and smoke-probe evidence.
- Search semantics have a small evaluation harness and projection-version
  discipline before AI/vector search is introduced.
- Tests around touched Interfaces are easier to navigate through domain-focused
  fixtures and smaller behavior families.

## In Scope

- Refactoring `taru-server` Addon Side Effect application Modules without
  changing the public Addon protocol shape unless a test proves the current
  shape is unsafe.
- Adding or deepening `taru-core` repository/workflow traits when they create
  real depth and are backed by at least two Adapters or a strong deletion-test
  argument.
- SQLite and PostgreSQL parity for any new persistence commit seam.
- Backend-neutral contract tests for commit behavior that crosses repository
  records.
- Focused playback/transcode identity and diagnostics refactors needed before
  richer playback profiles.
- Search semantics/evaluation scaffolding without provider breadth or AI
  runtime changes.
- Workstream, ADR, and API documentation updates when an Interface changes.

## Out Of Scope

- New TMDB, Douban, Bangumi, or AI provider breadth.
- Native plugin ABI or Jellyfin plugin compatibility.
- Network Tunnel Provider implementation.
- Adaptive bitrate ladder implementation.
- Moving artifact bytes into PostgreSQL.
- Managed Artwork PostgreSQL parity, except for routing/coordination with
  `docs/workstreams/managed-artwork-postgresql-parity/`.
- Broad UI work.
- Copying, translating, or importing Jellyfin/Plex source, tests, schemas, or
  assets.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| PostgreSQL runtime support for the M62-supported scope is committed and should be preserved. | High | `docs/workstreams/postgresql-production-readiness/`; commit `e45fa1a`. | Re-open M62 before adding new DB-dependent seams. |
| `AddonAppService` has real behavior, but its current Interface is too broad for future Addon Side Effect breadth. | High | `crates/taru-server/src/app/addons.rs`; Addon protected-write workstreams. | If the Module is already split locally in later changes, FAD-020 becomes an audit/cleanup task. |
| Addon metadata write projection consistency should be proven in one commit seam. | High | APW residual risk and review findings. | If existing metadata/catalog commit APIs can already cover it, prefer reuse over a new trait. |
| Library ingestion still needs a caller-facing workflow seam even though DB contracts prove scan commits. | Medium | `crates/taru-library/src/index.rs`; M62 scan commit contracts. | If a narrow seam adds no leverage by deletion test, split the task to test-layout and docs only. |
| Search should be measured before AI/vector integration. | High | `taru-search` is intentionally small; roadmap provider/search breadth remains future work. | If product priorities change, split search semantics into its own lane. |

## Architecture Direction

This lane uses Module-depth language deliberately:

- A **Module** earns its existence when its Interface gives callers leverage and
  keeps implementation knowledge local.
- A **Seam** is introduced only when behavior needs to vary, when ordering must
  be owned in one place, or when the deletion test shows callers would otherwise
  repeat domain knowledge.
- An **Adapter** should be concrete and testable. For persistence seams, SQLite
  plus PostgreSQL make the seam real; for runtime seams, fake Adapters are
  acceptable only when the Interface hides meaningful behavior in tests.
- Refactors should delete obsolete shims and caller-side ordering instead of
  adding pass-through layers.

The first execution slice is Addon Side Effect depth because it touches
permissions, redaction, storage, metadata authority, catalog/search projection,
and future plugin safety. It has the best leverage and locality payoff.

## Closeout Condition

This lane can close when:

- FAD-020 through FAD-090 are completed or explicitly split into narrower
  follow-on workstreams with evidence-backed rationale;
- final evidence includes formatting, workspace checks, focused nextest runs,
  and full workspace nextest when practical;
- any new persistence seam is covered by SQLite always-on and PostgreSQL
  opt-in contract evidence;
- docs and ADRs reflect shipped Interfaces;
- no public/admin/addon DTO leaks Source Locator, storage URI, local path, raw
  source URL, cache URI, content hash, secret, or raw database detail as part
  of the refactor.

Closeout result:

- FAD-020 through FAD-090 are complete.
- The final workspace gates passed on 2026-05-20.
- PostgreSQL opt-in contracts were skipped only because
  `TARU_TEST_POSTGRES_URL` was unset; the relevant contract pairs remain
  present for environments with a PostgreSQL test URL.
- No unowned architecture tail remains inside this lane. Existing independent
  tails stay in their named workstreams, especially
  `managed-artwork-postgresql-parity` and `admin-api-typescript-contract`.
