# Fearless Future Architecture Refactor

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

M61-M63 removed a lot of the obvious architecture debt, but the next wave of
growth will still harden a few wide modules if they are left alone. The largest
remaining hotspots are still visible in the workspace inventory:

| Hotspot | Approx. size | Why it matters |
| --- | --- | --- |
| `crates/nako-db/src/postgres.rs` | 10k+ lines | backend-specific persistence still concentrates too much behavior |
| `crates/nako-server/src/app/playback/mod.rs` | 1k+ lines | runtime orchestration, session policy, and hardware handling are still close together |
| `crates/nako-api/src/admin.rs` | 2k+ lines | admin DTOs still gather many unrelated surfaces in one file |
| `crates/nako-vfs/src/local.rs` | 2k+ lines | low-level path safety, writes, links, cleanup, and local authority are still coupled |
| `crates/nako-server/src/app/addons.rs` | 1k+ lines | addon runtime ownership is still broader than it should be |
| `crates/nako-server/src/app/metadata.rs` | 1k+ lines | refresh, local inference, and metadata policy still sit too close together |
| `crates/nako-library/src/lib.rs` | 1k+ lines | library orchestration still wants smaller workflow-shaped modules |

The problem is not that these files are large. The problem is that they still
mix orchestration, policy, and backend detail in places where future work needs
clear ownership boundaries.

## Target State

When this lane closes, Nako should look like a set of explicit control planes
with narrow leaf modules:

- `nako-server` owns orchestration, not hidden policy bundles.
- `nako-db` owns persistence families and backend modules, not one giant
  adapter file.
- `nako-api` owns explicit public/admin DTOs, not a convenience mirror of
  internal state.
- `nako-vfs` owns safe local primitives, while library-file-write policy lives
  one layer up.
- `nako-library`, `nako-naming`, and `nako-nfo` own inference and parsing, not
  server orchestration.
- Docker-backed validation is a normal part of refactor work, not a special
  release exercise.

## In Scope

- Splitting broad runtime modules in `nako-server`.
- Splitting broad persistence backend modules in `nako-db`.
- Splitting API DTO surfaces in `nako-api`.
- Deepening the VFS and library-file-write boundary.
- Deepening local inference and naming boundaries where they are still too
  shallow.
- Deleting redundant forwards, stale helpers, and compatibility shims once
  replacements are proven.
- Local Docker and PostgreSQL validation for touched seams.

## Out Of Scope

- New provider breadth for TMDB, Douban, Bangumi, or future addons.
- Plugin ABI work, native plugin loading, or Jellyfin compatibility.
- Network tunnel provider implementation.
- Adaptive bitrate ladder work.
- Broad UI feature work.
- Copying reference project source, comments, migrations, tests, or generated
  code.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako no longer has a production compatibility burden that blocks internal refactors. | High | M61-M63 are complete. | We would need to reintroduce compatibility gates and narrow deletion scope. |
| Local Docker is available for repeatable config and runtime validation. | Medium-High | `deploy/compose/` and `scripts/release-gate.ps1` already exist. | Validation would need a non-Docker fallback. |
| `repo-ref/jellyfin` is available as a behavior and layout reference only. | High | `repo-ref/jellyfin/` exists in the workspace. | We would lose a useful subsystem-layout comparison point. |
| Existing backend-neutral DB contracts and opt-in PostgreSQL harnesses remain valid. | High | `crates/nako-db/src/contract_tests.rs` and `scripts/postgres-contract-harness.ps1`. | The persistence split would need a new contract strategy. |
| Any cross-crate boundary or public/storage policy change will be documented before closeout. | High | Existing ADR and workstream policy already require this. | The lane would accumulate undocumented architecture drift. |

## Architecture Direction

Use the existing modular-monolith shape, but keep deepening it by ownership
boundary instead of by convenience layer.

The refactor order is deliberate:

1. Runtime control planes first. `nako-server` should keep the orchestration
   root thin and move large workflows into focused modules.
2. Persistence backend structure second. `nako-db` should become a clearer tree
   of backend-owned modules instead of a single large backend file.
3. API surfaces third. `nako-api` should split admin/public DTOs by surface and
   keep redaction local to the boundary.
4. File-authority and inference boundaries fourth. `nako-vfs`, `nako-library`,
   `nako-naming`, and `nako-nfo` should own the smallest sensible pieces of
   local parsing and write policy.
5. Validation and deletion last. Once a replacement is proven, the old path
   should go away immediately unless a named follow-on exists.

`repo-ref/jellyfin` is useful because it shows the same broad pattern at a
different scale: separate model, server, database, provider, and media-encoding
areas, rather than one monolithic server package. Nako should keep that
separation pattern but in Rust workspace terms.

## Closeout Condition

This lane can close when:

- the planned splits are implemented or explicitly carved into named follow-on
  lanes,
- the new module boundaries are documented,
- the required Docker and workspace gates pass,
- and the old redundant paths have been deleted or justified with a named
  expiry.

## Closeout Result

The lane met the target state on 2026-05-23. The broad server runtime,
PostgreSQL persistence, admin API, local VFS, and naming/local-inference
boundaries were split into focused modules or explicitly reviewed as acceptable
tails. The deletion sweep found no remaining replaced helper paths requiring
immediate removal; the remaining `local_inference.rs` width is a named
follow-up candidate rather than a lane blocker. The closeout gates passed:
workspace formatting, workspace test compilation, workspace nextest,
container release gate, PostgreSQL all-contract harness, and `git diff
--check`.
