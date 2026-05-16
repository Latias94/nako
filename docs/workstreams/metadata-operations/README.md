# Metadata Operations Workstream

## Purpose

This workstream turns metadata refresh from an item-level provider runtime into
an operable maintenance boundary for libraries. It covers batch refresh jobs,
provider diagnostics, raw cache retention, provider health visibility, and
multi-library profile correctness.

## Status

Active in M13.

## Goals

- Refresh metadata for one item set or one library through a durable job.
- Keep job input, summaries, events, and diagnostics free of resolved secrets.
- Make provider attempts queryable by provider and status.
- Give raw provider cache a retention policy and cleanup entry point.
- Surface provider runtime health as process-local state.
- Keep library metadata profile selection consistent across scan, refresh,
  diagnostics, catalog hydration, and search hydration.

## Non-Goals

- A distributed provider health store.
- A multi-process raw cache cleanup scheduler.
- Full OpenAPI generation.
- In-process plugin metadata writeback.

## Active Phases

- [Phase 13.0: Maintenance Job Boundary](PHASE13_0_MAINTENANCE_JOB_BOUNDARY.md)
- [Phase 14.0: Scheduling And Lifecycle](PHASE14_0_SCHEDULING_AND_LIFECYCLE.md)

## Related ADRs

- [ADR 0018: Metadata Provider Runtime and Diagnostics](../../adr/0018-metadata-provider-runtime-and-diagnostics.md)
