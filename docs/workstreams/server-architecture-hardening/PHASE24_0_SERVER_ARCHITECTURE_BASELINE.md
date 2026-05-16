# Phase 24.0: Server Architecture Baseline

## Summary

M24 starts with documentation because the next implementation work is a
cross-cutting refactor, not a feature patch. The target is a cleaner server
architecture: `TaruApp` composes runtime pieces, focused services own
workflows, supervised workers own background execution, repositories own
transaction details, and obsolete MVP helpers are removed instead of preserved.

## Starting Point

Important current surfaces to re-audit during the first code pass:

- `crates/taru-server/src/app.rs`: root application type, service composition,
  library resolution helpers, startup work, and broad convenience methods.
- `crates/taru-server/src/app/metadata.rs`: metadata refresh, provider runtime
  construction, raw cache, and job orchestration.
- `crates/taru-server/src/app/playback.rs`: direct play, remux, HLS, staging,
  resource permits, and playback session orchestration.
- `crates/taru-server/src/app/storage.rs`: backend resolution, staging wrapper,
  and configured library storage behavior.
- `crates/taru-nfo/src/lib.rs`: NFO parse/export policy and current XML
  handling boundary.
- `crates/taru-catalog/src/lib.rs`: catalog graph hydration and search
  projection write boundaries.
- `docs/workstreams/server-foundation/PHASE20_0_SERVER_SURFACE_DECOMPOSITION.md`:
  earlier test-surface decomposition evidence.
- `docs/workstreams/server-foundation/PHASE23_0_API_HTTP_DB_BOUNDARY_CLEANUP.md`:
  API, HTTP router, and DB cleanup rules that M24 should preserve.
- `docs/workstreams/runtime-foundation/PHASE19_0_DATABASE_BOUNDARY_HARDENING.md`:
  database transaction and repository boundary evidence that M24 should build
  on rather than duplicate.

## Decisions

- Treat M24 as an architecture workstream, not as metadata provider feature
  expansion.
- Keep the modular monolith. Do not split server processes.
- Make `TaruApp` thin enough that new feature work naturally lands in focused
  services.
- Introduce one lifecycle owner for background tasks before adding more
  scheduled or provider-driven work.
- Prefer deleting obsolete code over carrying compatibility shims while Taru is
  pre-compatibility.
- Use existing later phase notes as evidence and follow-up context, but keep
  M24 scoped to server architecture cleanup.

## Suggested Implementation Order

1. Freeze the intended `TaruApp` public surface and identify methods that
   should move to services.
2. Introduce service handles and migrate one low-risk workflow first to prove
   the constructor and test pattern.
3. Move remaining workflow orchestration out of `TaruApp` by bounded context.
4. Add the runtime supervisor boundary and migrate detached task launch sites.
5. Tighten repository and transaction boundaries where app services still
   coordinate multi-record writes manually.
6. Delete obsolete helpers and compatibility paths after tests cover the new
   invariants.
7. Close with a stabilization audit and full workspace validation.

## Risks

- Moving too many service boundaries at once can make review noisy. Keep each
  phase focused around one ownership boundary.
- A supervisor abstraction can become too generic. It should solve concrete
  Taru lifecycle needs: startup, shutdown, cancellation, task failure, and
  diagnostics.
- Repository cleanup should not leak SQLite details into `taru-core` traits.
  Keep SQL-specific concerns inside `taru-db`.
- Deleting compatibility helpers is correct, but deletion should follow a
  proven replacement path and focused tests.

## Validation

Baseline validation for this docs-only phase:

```powershell
git diff --check
```
