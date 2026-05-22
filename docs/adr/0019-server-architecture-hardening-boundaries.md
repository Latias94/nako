# 0019: Use a Thin Server Composition Root and Explicit Runtime Supervisors

## Status

Accepted.

## Context

Nako has grown from an MVP backend into a modular monolith with playback,
metadata, NFO, VFS, webhook, automation, addon, staging, and transcode paths.
That growth validated the product direction, but several early shortcuts are
now working against long-term architecture quality:

- `nako-server::app::NakoApp` is too wide and owns composition, feature
  orchestration, repository access, storage resolution, runtime tasks, and
  compatibility helpers in one surface.
- Feature modules have started to split out, but high-level services still
  reach into the concrete `SqliteStore` shape instead of consistently using
  narrow ports or focused service handles.
- Background work is launched from multiple feature paths through ad hoc
  `tokio::spawn` calls, making lifecycle, shutdown, error reporting, and
  resource ownership harder to reason about.
- Some domain operations still rely on helper functions that made sense in the
  single-library MVP, such as deriving a library from a source by scanning
  configured roots.
- Catalog, metadata, NFO, staging, and scan operations need clearer atomic
  boundaries before more providers, clients, and networked storage backends are
  added.
- NFO handling still contains hand-written XML walking that should be replaced
  by a structured parser before format breadth increases.

Nako has not shipped a stable compatibility contract yet. This means the right
move is fearless cleanup: remove obsolete helper code, collapse duplicate paths,
and make the architecture correct instead of preserving MVP shapes.

## Decision

`nako-server` remains the server composition crate, but its root application
type must become a thin composition root. It should assemble configuration,
repositories, storage backends, runtime supervisors, and application services;
it should not keep growing feature orchestration methods.

Feature orchestration belongs in focused application services. A service should
own one bounded workflow, such as playback planning, metadata refresh, NFO
import/export, library scan/probe, staging lifecycle, catalog hydration, or
extension dispatch. HTTP handlers should depend on these services and translate
requests/responses only.

Background work must be registered through an explicit runtime supervisor or
worker registry. The supervisor owns task handles, cancellation, startup order,
shutdown, bounded concurrency, and task-level diagnostics. Feature services may
request work, but they should not directly detach long-running runtime tasks.

High-level server code should depend on narrow ports or service handles rather
than the full concrete `SqliteStore` whenever a workflow does not need
SQLite-specific behavior. Concrete transaction and SQL details stay inside
`nako-db`. When an operation must update several persistence records together,
the transaction boundary belongs in the repository implementation or an
explicit unit-of-work boundary, not in scattered app-level write calls.

MVP compatibility helpers should be deleted once their replacement boundary is
in place. This includes helpers that infer library identity from paths after
`MediaSource.library_id` is available, duplicate route wiring, obsolete config
translation paths, and temporary service methods that exist only because the
old root app was too broad.

## Consequences

- `NakoApp` becomes easier to review because it composes the server instead of
  acting as the server.
- Feature workflows can evolve independently without forcing every change
  through one large application surface.
- Background task behavior becomes observable and shutdown-safe.
- Repository interfaces become clearer, and transaction boundaries become
  explicit enough for provider expansion, multi-library correctness, and
  future client contracts.
- Some existing code will be deleted or moved aggressively. This is acceptable
  while Nako is pre-compatibility, but every deletion needs focused validation
  evidence.
- The first M24 slice is allowed to be architecture-only. It should not add new
  product features while reshaping the server boundary.

## Alternatives Considered

- Keep adding methods to `NakoApp`: rejected because it preserves the current
  bottleneck and makes every future feature harder to reason about.
- Split into multiple deployable services now: rejected because Nako still
  benefits from a modular monolith. The problem is internal ownership, not
  process boundaries.
- Introduce a generic dependency-injection framework: rejected because Rust
  module boundaries, explicit constructors, traits, and service handles are
  enough for the current scale.
- Delay cleanup until after more metadata providers and clients: rejected
  because those features would deepen the existing coupling and make the
  eventual refactor more expensive.

## Related Workstreams

- `docs/workstreams/server-architecture-hardening/README.md`
- `docs/workstreams/runtime-foundation/README.md`
- `docs/workstreams/metadata-operations/README.md`
- `docs/workstreams/server-foundation/PHASE20_0_SERVER_SURFACE_DECOMPOSITION.md`
- `docs/workstreams/server-foundation/PHASE23_0_API_HTTP_DB_BOUNDARY_CLEANUP.md`
- `docs/workstreams/runtime-foundation/PHASE19_0_DATABASE_BOUNDARY_HARDENING.md`
