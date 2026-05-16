# Runtime Foundation Workstream

## Purpose

This workstream owns cross-cutting runtime correctness, safety, and operational
boundaries that do not belong to a single product feature. It covers database
runtime behavior, migrations, secret handling, resource limits, capability
selection, lifecycle tasks, and diagnostics.

Runtime foundation work should reduce long-term coupling across
`taru-server`, `taru-db`, `taru-metadata`, `taru-transcode`, storage, and
future clients. It is the place to make fearless refactors when an MVP shortcut
has become a shared system risk.

## Status

Active for M16 storage backend registry and lease lifecycle hardening after
M15 runtime foundation implementation.

## Goals

- Make SQLite behavior explicit under concurrent scans, playback, metadata
  maintenance, automation, and webhook workers.
- Replace fragile migration execution with a runner that can safely handle
  future complex SQL.
- Ensure resolved secrets cannot leak through `Debug`, config diagnostics, job
  inputs, provider structs, or API responses.
- Make hardware acceleration selection a server runtime decision based on
  capability detection, policy, fallback, and resource budgets.
- Keep app composition thin by moving reusable runtime policies into focused
  modules or crates.
- Prefer explicit API DTOs for new public surfaces instead of expanding direct
  `taru-core` exposure.

## Non-Goals

- Distributed multi-process coordination for SQLite or lifecycle workers.
- Production-grade observability stacks, metrics backends, or tracing
  exporters.
- In-process plugin runtime design.
- Client UI protocol stabilization.
- Keeping deprecated config or API shapes only for backward compatibility.

## Refactor Policy

Taru has not shipped a compatibility contract yet. Runtime foundation phases may
remove old code paths, rename configuration fields, tighten public DTOs, and
delete legacy helpers when doing so produces a cleaner boundary.

Compatibility is only preserved when it lowers risk without keeping a bad
design alive. Otherwise, prefer the smallest complete design that is correct for
the next several milestones.

## Active Phases

- [Phase 15.0: Runtime Hardening Baseline](PHASE15_0_RUNTIME_HARDENING_BASELINE.md)
- [Phase 15.1: Runtime Hardening Implementation](PHASE15_1_RUNTIME_HARDENING_IMPLEMENTATION.md)
- [Phase 16: Storage Backend Registry And Lease Lifecycle](PHASE16_STORAGE_BACKEND_REGISTRY_AND_LEASE_LIFECYCLE.md)

## Related Workstreams

- [metadata-operations](../metadata-operations/README.md): provider runtime,
  maintenance scheduling, raw cache lifecycle, and metadata diagnostics.
- [playback-streaming](../playback-streaming/README.md): direct play, staging,
  HLS/remux playback, and remote playback resource budgets.
- [storage-vfs](../storage-vfs/README.md): VFS backend behavior, remote storage
  caching, and backend capability boundaries.
- [server-foundation](../server-foundation/README.md): historical foundation
  milestones and earlier runtime planning.
