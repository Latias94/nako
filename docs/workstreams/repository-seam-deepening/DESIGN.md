# Repository Seam Deepening And Workflow Port Extraction

Status: Completed
Last updated: 2026-05-17

## Why This Lane Exists

M28 split several large modules, and M38 moved startup/runtime orchestration
behind deeper server interfaces. The next source of friction is that some
workflow crates still depend on broad repository trait combinations. A caller
that wants catalog hydration or metadata refresh often has to satisfy several
low-level repository traits, even when it only wants one workflow.

The problem is not that Taru has too many traits. The problem is that some
traits are shallow from a workflow caller's point of view: the interface exposes
many persistence details instead of one use-case-shaped port.

## Target State

- Workflow crates use narrow ports named after real workflows.
- The first slice extracts `CatalogHydrationPort` for catalog hydration and
  search projection.
- `taru-metadata` and `taru-nfo` call catalog hydration through the workflow
  port instead of requiring the full `CatalogRepository + MediaRepository +
  SearchIndex` combination at each caller.
- SQLite integration remains in `taru-db`; schema stays unchanged.
- Tests distinguish pure workflow behavior from SQLite adapter behavior.

## In Scope

- Open a durable M39 workstream with design, task, milestone, evidence, and
  handoff docs.
- Add a use-case-level catalog hydration port.
- Keep the existing broad repository traits for lower-level browse/admin/query
  surfaces while letting hydration callers depend on the narrower port.
- Add focused tests that can exercise catalog hydration through a fake port.
- Keep existing SQLite behavior covered by `SqliteStore` adapter tests.
- Update GOALS, ROADMAP, and the workstream index.

## Out Of Scope

- Playback source selection, transcode plan, client profile, and runtime
  execution semantics.
- NFO Round Trip preservation, unknown XML field retention, partial XML
  updates, and soft/hard-link policy.
- Public Client API, OpenAPI, Rust SDK, TypeScript SDK, CLI, and license
  boundary changes.
- Database schema changes unless the first slice proves an existing behavior
  cannot be expressed.
- Broad mechanical splitting of every repository trait.
- Webhook, automation, addon, and playback runner runtime migration.

## Architecture Direction

Use workflow ports where the interface can hide meaningful behavior. For M39.1,
`CatalogHydrationPort` should describe what catalog hydration needs as a
workflow:

- load the current **Media Item** plus the existing graph/search context needed
  to hydrate it;
- persist the replacement **Catalog Item Graph** and search projection through
  one use-case method.

The existing low-level repository traits still matter for query surfaces,
diagnostics, and SQLite integration. The new port should not replace them
mechanically; it should reduce what metadata and NFO workflows need to know.

## Closeout Condition

M39 can close when:

- the workstream is documented and indexed;
- catalog hydration has a narrow workflow port;
- at least metadata refresh and NFO import/confirmation callers use that port;
- focused tests prove the port contract and existing SQLite behavior;
- workspace validation gates pass;
- follow-on repository seam work is split clearly instead of left implicit.
