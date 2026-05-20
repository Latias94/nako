# Fearless Architecture Deepening — Handoff

Status: Active
Last updated: 2026-05-20

## Current State

M62 PostgreSQL Production Readiness has been committed as
`e45fa1a refactor: complete postgresql production readiness`.

This workstream is now the active architecture-deepening lane for the next
fearless refactor pass. It records the 2026-05-20 architecture review findings
and prioritizes the Modules most likely to hurt future Taru evolution if they
harden as-is.

Completed task:

- FAD-020 — Addon Side Effect Module depth.

Current executable task:

- FAD-030 — Addon metadata commit atomicity.

Why FAD-020 comes first:

- Addon Side Effects touch permission, grants, redaction, idempotency, storage,
  Canonical Metadata authority, Catalog Item Graph/Search Projection refresh,
  NFO/Library File Write policy, artwork candidate intake, and future plugin
  safety.
- `crates/taru-server/src/app/addons.rs` currently concentrates too many of
  those concerns in one Module.
- A behavior-preserving split can improve locality before semantic changes.

## Decisions So Far

- Keep the lane architecture-first. Do not add provider breadth, network
  traversal, native plugin ABI, adaptive bitrate, or AI runtime features here.
- Managed Artwork PostgreSQL parity remains a separate proposed follow-on:
  `docs/workstreams/managed-artwork-postgresql-parity/`.
- Prefer deep workflow seams over mechanical trait splits.
- New persistence commit seams must prove SQLite and PostgreSQL behavior
  through backend-neutral contracts.
- Addon metadata write atomicity is the first semantic refactor after the
  behavior-preserving Addon Module split.

## FAD-020 Summary

FAD-020 split the Addon Side Effect implementation into focused server Modules:

- `principal.rs` for Addon Principal resolution, grant authorization, token
  label normalization, and grant normalization.
- `intake.rs` for side-effect idempotency, validation, safe validation error
  codes, and accepted/rejected intake persistence.
- `side_effect_apply.rs` for apply routing and common apply-outcome recording.
- `metadata_write.rs` for Canonical Metadata patch/merge plus the existing
  catalog/search refresh behavior.
- `library_file_write.rs` for NFO Library File Write apply through Taru's NFO
  service, VFS backend, write policy, and backup policy.
- `artwork_write.rs` for Addon Artwork Candidate proposal.
- `target.rs` for shared Media Item resolution from side-effect targets.

Validation passed:

- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- `cargo nextest run -p taru-server addon --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

FAD-020 intentionally preserved behavior. It did not fix Addon metadata write
atomicity; that remains the purpose of FAD-030.

## Blockers

- None for FAD-030.

## Next Recommended Action

1. Start FAD-030.
2. Inspect the current metadata write Adapter and existing metadata/catalog
   commit contracts.
3. Design the transactional commit seam so the Interface expresses the Addon
   Canonical Metadata write domain action rather than exposing caller-ordered
   repository calls.
4. Add SQLite always-on and PostgreSQL opt-in evidence if the seam touches
   persistence.
5. Preserve current Addon DTO redaction guarantees.
