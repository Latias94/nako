# Storage VFS OpenDAL Adapter Decision Spike

## Goal

Decide whether Apache OpenDAL should become a future implementation adapter
foundation behind Nako's `StorageBackend` boundary, and record the next safe
implementation slice for M2 storage/VFS backend breadth.

## What I Already Know

- Nako already owns the storage abstraction through `StorageUri`,
  `StorageBackend`, `StorageCapabilities`, `ByteRange`, cache repair
  diagnostics, storage health, source identity, and deterministic staging.
- ADR 0002 says Nako must keep an internal VFS abstraction instead of relying
  on OS mounts.
- ADR 0016 says remote storage work must not leak backend credentials, local
  paths, cache facts, or staging details into catalog or API surfaces.
- `docs/architecture/STORAGE_VFS.md` already lists
  `proposed:storage-opendal-adapter-decision-spike` as a next work lane.
- Apache OpenDAL 0.57.0 provides a unified Rust `Operator`, optional
  service/layer model, stat/read/range/list APIs, and Apache-2.0 licensing.
- This task is a decision spike only. It must not add an OpenDAL dependency,
  modify runtime behavior, or replace the current WebDAV/local implementations.

## Requirements

- Evaluate Apache OpenDAL against Nako's VFS boundary, not as a replacement for
  Nako domain concepts.
- Compare at least three feasible directions:
  - keep bespoke adapters only;
  - introduce OpenDAL as an optional adapter foundation;
  - replace `StorageBackend` with OpenDAL `Operator`.
- Record the decision in an ADR and update the storage/VFS architecture map.
- Preserve explicit non-goals around dependency introduction, schema changes,
  API changes, and runtime behavior changes.
- Identify the first safe proof-adapter slice if OpenDAL is accepted.

## Acceptance Criteria

- [x] OpenDAL official/current API and crate metadata are checked.
- [x] Existing `nako-vfs` boundary and storage architecture constraints are
      inspected.
- [x] ADR includes context, decision, consequences, alternatives, risks, and
      success metrics.
- [x] `docs/architecture/STORAGE_VFS.md` no longer leaves the decision spike as
      unresolved.
- [x] Task evidence records the exact verification commands.
- [x] Trellis task context validation passes.
- [x] Documentation diff passes `git diff --check`.

## Definition Of Done

- Docs, ADR, task PRD, research notes, and evidence are committed together.
- No Rust runtime/code dependency on OpenDAL is added by this task.
- The next implementation slice is narrow enough to prove adapter semantics
  without changing product behavior.

## Technical Approach

Use the spike as an ADR-style decision. Accept OpenDAL only as a future optional
adapter implementation behind `StorageBackend`, with a proof adapter that maps a
single service to Nako-owned URI, capability, error, range-read, list, and
redaction rules.

## Decision (ADR-lite)

**Context**: Nako wants broader backend coverage than hand-written WebDAV and
local adapters can comfortably sustain, but the current storage model already
encodes product-specific safety semantics that a generic storage library cannot
own.

**Decision**: Use Apache OpenDAL as an accepted candidate for future optional
adapter implementations behind `StorageBackend`. Do not replace
`StorageBackend`, `StorageUri`, source locator redaction, storage health, VFS
cache repair authority, or deterministic staging with OpenDAL primitives.

**Consequences**: The next slice can test whether an OpenDAL-backed adapter can
preserve Nako semantics with a low-risk in-memory or fs proof. Production S3,
WebDAV replacement, credential migration, and public configuration changes
remain separate follow-ons.

## Out Of Scope

- No `opendal` dependency in this task.
- No `Cargo.lock` update.
- No local/WebDAV adapter replacement.
- No S3/WebDAV production backend configuration.
- No schema migration.
- No Admin/Public API change.
- No runtime scheduler, cache repair, or storage health behavior change.

## Technical Notes

- Research notes: `research/opendal-adapter-decision.md`.
- New ADR: `docs/adr/0055-opendal-storage-adapter-foundation.md`.
- Existing authority:
  - `docs/adr/0002-internal-vfs-before-os-mounting.md`
  - `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `.trellis/spec/nako-vfs/backend/index.md`
  - `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
