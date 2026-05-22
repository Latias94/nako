# Future-Ready Architecture Refactor Design

Status: Completed
Last updated: 2026-05-20

## Why This Lane Exists

Nako's current architecture is already directionally strong: Rust workspace,
modular monolith, VFS-first storage, Addon Sidecars, explicit Public Client API,
separate Admin API, durable jobs, event outbox, NFO boundary, provider mapping,
and playback runtime ownership.

The next risk is not feature absence. The next risk is letting early MVP shapes
become permanent architecture:

- `nako-db::SqliteStore` has become a broad concrete adapter that implements
  almost every repository interface.
- Some transaction boundaries are workflow-specific methods rather than a
  consistent persistence model that can be implemented by SQLite and
  PostgreSQL.
- `nako-server::NakoApp` still knows too much construction detail for storage,
  metadata, playback, automation, jobs, Addons, webhooks, and admin surfaces.
- Local source discovery, Local Inference, provisional hierarchy creation,
  catalog/search projection, and scan failure resolution remain close enough
  that future anime/library breadth will be harder to reason about.
- Metadata providers can grow into direct Canonical Metadata writers unless
  provider output is normalized through a candidate/acceptance seam.
- `nako-search` is currently shallow; future multilingual, alias, provider-id,
  facet, and AI-assisted search needs a deeper module.
- Admin API and frontend contract work can accidentally mirror internals unless
  read models remain explicit.

The project is not live and has no compatibility burden. This lane therefore
uses a fearless refactor policy: remove compatibility shims, redundant code,
unused helpers, and shallow pass-through modules when a cleaner architecture is
available and covered by tests.

## Relevant Authority

- ADRs:
  - `docs/adr/0001-modular-monolith-rust-workspace.md`
  - `docs/adr/0005-bounded-async-pipelines-and-resource-budgets.md`
  - `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0011-normalized-catalog-graph-and-search-projection.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`
  - `docs/adr/0019-server-architecture-hardening-boundaries.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0021-video-first-media-server-domain-model.md`
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
  - `docs/adr/0029-postgresql-ready-persistence-boundary.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/development/REFACTORING_POLICY.md`
  - `docs/development/TEST_STRATEGY.md`
  - `docs/api/HTTP_API.md`
- Related completed workstreams:
  - `docs/workstreams/core-architecture-deepening/`
  - `docs/workstreams/repository-seam-deepening/`
  - `docs/workstreams/metadata-catalog-commit-atomicity/`
  - `docs/workstreams/metadata-merge-policy-unification/`
  - `docs/workstreams/catalog-hydration-lookup-deepening/`
  - `docs/workstreams/server-runtime-deepening/`
  - `docs/workstreams/admin-api-typescript-contract/`

## Problem

### P0 — Persistence Is SQLite-Shaped

`SqliteStore` is a concrete type with broad trait implementations. This made
the MVP fast, but it weakens locality:

- database connection, SQL codec helpers, migrations, repositories, and
  workflow commits are too close;
- adding PostgreSQL would either duplicate a large adapter or force every
  repository method to negotiate dialect differences;
- tests mostly prove SQLite behavior, not backend-neutral persistence
  contracts;
- transaction semantics are not yet expressed as a portable unit of work.

### P1 — Composition Root And App Services Still Carry Too Much Wiring

`NakoApp` remains readable, but it constructs too many runtime concerns
directly. Adding network tunnel providers, AI automation, Addon Manager
behavior, richer Admin API read models, or more playback workers would push
more knowledge into the root unless runtimes are grouped behind deeper modules.

### P2 — Local Inference Is Too Close To Scanning

`LibraryIndexService` still owns discovery commit planning, provisional item
creation, Local Inference Evidence, Source State, Library Item State, search
projection, and failure resolution. That is too much for the module that should
primarily coordinate Media Source discovery.

### P3 — Provider Output Needs A Candidate Graph

TMDB, Douban, Bangumi, NFO, Addons, and future Automation Providers do not
share the same native shape. Directly mapping each provider toward Canonical
Metadata increases the risk of provider-centric core models.

### P4 — Search Is Too Shallow

`nako-search` currently defines a small index trait and basic document/query
types. Future multilingual search, aliases, provider IDs, Browse Facets, Sort
Keys, and AI-assisted search need explicit semantics before the database schema
or UI expectations harden.

### P5 — Admin/API/Generated Contract Hygiene Needs A Stronger Boundary

Admin API read models should stay admin-owned and redacted. Frontend and SDK
generated artifacts should not introduce repository noise, stale generated
state, or accidental coupling to internal records.

## Target State

When this lane closes:

- Nako has a persistence architecture that can support both SQLite and
  PostgreSQL without duplicating a god adapter.
- Backend-neutral persistence contracts, transaction/unit-of-work semantics,
  and contract tests are explicit.
- SQLite implementation details live in SQLite-owned modules or crates; any
  future PostgreSQL adapter can implement the same contract without importing
  SQLite assumptions.
- `NakoApp` delegates construction to cohesive runtime modules instead of
  directly wiring every feature.
- Local Inference is a deep module that transforms discovered sources and
  evidence into provisional hierarchy plans.
- Metadata provider output can flow through provider-neutral candidates and an
  Acceptance Workflow rather than provider-specific canonical writes.
- Search semantics are explicit enough to choose SQLite FTS, Tantivy, Meilisearch,
  PostgreSQL full text search, or a hybrid later without rewriting callers.
- Admin API read models remain explicit, redacted, and separate from Public
  Client API DTOs.
- Old production paths, redundant helpers, and compatibility shims introduced
  by earlier MVP slices are deleted.

## In Scope

- Persistence and PostgreSQL-readiness design.
- Repository/transaction/unit-of-work deepening.
- Potential crate boundary changes such as splitting `nako-db` into a
  backend-neutral persistence contract and SQLite implementation.
- Contract tests for persistence behavior.
- Server runtime grouping and composition-root slimming.
- Local Inference Engine extraction.
- Metadata Candidate Graph design and first implementation slice.
- Search semantic module design and first implementation slice.
- Admin/API generated contract hygiene when it affects architecture boundaries.
- Deletion sweeps for obsolete helpers and duplicate paths.
- ADR updates when a hard-to-change contract is altered.

## Out Of Scope

- Full PostgreSQL production adapter unless the persistence deep dive chooses it
  as the smallest safe proof slice. This lane must make PostgreSQL possible;
  it does not have to finish every PostgreSQL operational detail.
- In-process plugin ABI or Jellyfin Plugin Compatibility.
- Copying Jellyfin, Plex, or other reference source, schemas, comments,
  migrations, tests, or generated code.
- New provider breadth for TMDB, Douban, Bangumi beyond what is necessary to
  prove the candidate seam.
- Network Tunnel Provider implementation.
- Full AI model runtime, vector database, or local model scheduling.
- New client UI features unless needed to protect Admin/Public API boundaries.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako has no production compatibility burden yet. | High | User direction and current project state. | If external users already depend on current APIs, deletion must be staged behind explicit migration notes. |
| PostgreSQL support is a future architecture requirement. | High | User explicitly requested PostgreSQL consideration. | If PostgreSQL is dropped, persistence still benefits from cleaner seams but some dialect work can defer. |
| `SqliteStore` breadth is the highest-leverage refactor target. | High | Architecture review and crate scan show `nako-db` owns most repository implementations. | If another bottleneck appears during audit, FRA-020 must update priority before implementation starts. |
| Crate renames/splits are acceptable. | Medium | User requested fearless refactor and no compatibility burden. | If package names must remain stable, use internal module splits first and defer crate renames. |
| Existing tests are sufficient to protect most behavior during refactor. | Medium | Workspace has focused tests across db/server/library/metadata/NFO/playback. | If gaps appear, add characterization tests before deleting old paths. |

## Architecture Direction

### Persistence First

The first deep dive must decide the clean persistence shape before code churn.
Preferred direction to validate:

- keep domain records and workflow concepts provider/database-neutral;
- define backend-neutral persistence contracts and contract tests;
- make transaction/unit-of-work behavior explicit for multi-record commits;
- isolate SQL dialect, migrations, row codecs, and pool types in backend-owned
  adapters;
- avoid generic CRUD repositories when a workflow-shaped commit gives better
  locality and leverage;
- remove old SQLite-shaped pass-through paths after the new persistence seam is
  tested.

The deep dive should consider whether the clean final state is:

- a slimmer `nako-db` facade over backend-specific implementation crates;
- `nako-sqlite` for the SQLite SQLx implementation;
- future `nako-postgres` for the PostgreSQL SQLx implementation;
- a staged internal `nako-db::sqlite` split first, followed by crate extraction
  when contract tests are in place.

FRA-020 accepted ADR 0029 and chose this facade-plus-adapters target. Do not
preserve the current `SqliteStore`-as-server-store shape merely to avoid edits.

### Runtime Modules

Server construction should move toward cohesive runtime modules:

- Storage Runtime;
- Metadata Runtime;
- Playback Runtime;
- Automation/Addons Runtime;
- Admin Operations Runtime;
- Job Runtime.

`NakoApp` should remain a thin composition root that exposes application
capabilities, not the place where every concrete service is assembled.

### Local Inference Engine

Scanning should discover Media Sources. Local Inference should interpret paths,
filenames, nearby local files, NFO hints, and probe facts into Local Inference
Evidence and provisional hierarchy plans. Catalog/search commits should persist
the result. This gives anime, series, unknown items, extras, and franchise
relationships a deeper seam.

### Metadata Candidate Graph

Provider adapters should map native provider payloads to provider-neutral
candidates. The Acceptance Workflow should decide what becomes Canonical
Metadata, Provider Mapping, Artwork Candidates, or Generated Artifacts. This
protects Nako's domain from TMDB/Douban/Bangumi-specific shapes.

### Search Semantics

`nako-search` should own explicit search semantics: normalized text, aliases,
facets, sort keys, provider identifiers, index versioning, and projection
contracts. Storage-specific implementation can remain replaceable.

### Deletion Rules

- No production old/new parallel path may remain after a task closes unless the
  next task names the deletion owner and expiry gate.
- Compatibility shims need a named migration reason. "Easier for now" is not a
  valid reason in this lane.
- Generated files, frontend build outputs, logs, and dependency folders should
  be ignored or regenerated through documented commands, not hand-maintained.

## Priority Table

| Priority | Area | Why first/why later | First task |
| --- | --- | --- | --- |
| P0 | Persistence/PostgreSQL readiness | All future metadata, search, jobs, and admin read models depend on it. | FRA-020 |
| P1 | Server runtime composition | Keeps later workers from expanding `NakoApp`. | FRA-070 |
| P2 | Local Inference Engine | Needed before serious anime/series/provider breadth. | FRA-080 |
| P3 | Metadata Candidate Graph | Needed before Douban/Bangumi/TMDB coexistence and AI artifacts deepen. | FRA-090 |
| P4 | Search semantics | Important for product quality, but can follow persistence shape. | FRA-100 |
| P5 | Admin/API/generated hygiene | Protects clients and repo cleanliness during UI/admin growth. | FRA-110/FRA-120 |
| P6 | Final deletion/closeout | Must happen after each slice and again at lane close. | FRA-130 |

## Closeout Condition

This lane can close when:

- P0 persistence architecture is implemented or split into a narrower active
  persistence execution lane with ADR-backed decisions and fresh evidence;
- at least one downstream deepening slice proves the new persistence/runtime
  direction through code and tests;
- Local Inference, Metadata Candidate Graph, Search, and Admin/API hygiene are
  either implemented or split into focused follow-on workstreams with owners;
- obsolete code paths introduced by this lane are deleted;
- docs and evidence reflect the shipped architecture;
- final verification gates pass.

## Closeout Summary

M61 closed with the target state implemented rather than split:

- `nako-db` now presents a `NakoDatabase` facade while SQLite implementation
  details live under SQLite-owned modules.
- Backend-neutral job lease contract tests run against SQLite, with an ignored
  PostgreSQL proof harness gated by `NAKO_TEST_POSTGRES_URL`.
- ADR 0029 and ADR 0030 record the PostgreSQL-ready persistence boundary,
  SQL dialect, migration, row-codec, and fixture policies.
- `NakoApp` delegates construction to the `app::composition` module.
- Local Inference, Metadata Candidate Graph, and Search semantic projection
  have explicit domain seams instead of being provider-, scanner-, or
  database-shaped.
- Admin/Public API boundaries remain explicit and redacted, generated frontend
  and SDK artifacts are reproducible, and the `nako-api` root compatibility
  re-export shim was deleted.

Residual broad work is intentionally future product scope, not closeout debt:
production PostgreSQL operations, optional backend crate extraction, richer
search backends, network tunneling, AI automation runtime, and further Admin UI
feature work should be opened as narrower workstreams when prioritized.
