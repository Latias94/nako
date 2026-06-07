# Quality Guidelines

Use these rules when changing shared domain contracts.

## Required Patterns

- Prefer plain, serializable domain records with explicit fields. Public records
  commonly derive `Clone`, `Debug`, `Serialize`, `Deserialize`, `Eq`, and
  `PartialEq` when the type is part of a persisted or wire-adjacent contract.
- Use `#[serde(rename_all = "snake_case")]` for enum wire/persistence names
  that are serialized.
- Pair persisted enums with `as_str` and `parse` helpers, or equivalent score
  helpers when the database representation is numeric.
- Keep resource policy generic at the core layer. For example, durable job
  priority is a generic scheduler policy, not a metadata-review shortcut.
- Keep source fingerprint escalation as a pure domain decision. The decision can
  expose action, reason, evidence kind, confidence, stale state, and candidate
  count, but must not expose raw locators, etags, fingerprints, paths, or
  backend URLs.
- `JobKind::SourceFingerprintHash` is the persisted queue kind for future
  source fingerprint hash work only. Keep its redaction-safe input contract in
  `nako-library::source_hash` and its runtime budget mapping in `nako-server`;
  do not move VFS execution, source lookup, or operator diagnostics into
  `nako-core`.
- `JobKind::VfsCacheRepair` is the persisted queue kind for future VFS cache
  repair work only. Keep its redaction-safe input contract in
  `nako-core::vfs_cache` because the input is derived from core cache failure
  records, but keep backend execution, target lookup, operator routes, and
  scheduler policy in `nako-server` / `nako-vfs`.
- Durable VFS cache repair input must be digest- and authority-based: it may
  carry action, source scheme, cache operation, failed-at timestamp, failure
  count, URI digest, and `VfsCacheFailureAuthority`, but not raw `StorageUri`,
  local path, backend URL, credential, raw backend error, etag, fingerprint, or
  cache payload material.
- When a core change crosses crate boundaries, update the relevant Trellis spec,
  ADR, architecture map, or task context before considering the work complete.

## Forbidden Patterns

- Do not add feature-specific shortcuts to shared scheduler, repository,
  playback, or access records.
- Do not expose raw local paths, tokens, secret values, or provider cache
  payloads as general-purpose core diagnostics.
- Do not add a new crate just to avoid deepening an existing core module. AGENTS
  prefers internal module deepening until multiple real callers prove a new
  crate boundary.
- Do not make `nako-core` depend on implementation crates such as `nako-db`,
  `nako-server`, `nako-vfs`, `nako-transcode`, or `nako-api`.

## Tests Required

- Pure domain behavior should use unit tests in the owning crate.
- Source fingerprint escalation policy tests should cover no escalation,
  partial-hash recommendation, and full-hash recommendation without requiring
  storage, VFS, repository, or runtime work.
- New persisted `JobKind` values must cover `as_str` / `parse` round trips in
  `nako-core`.
- New durable job input records in `nako-core` must cover serialization
  round-trip, validation rejection, and redaction assertions proving unsafe
  storage/provider material is absent from serialized JSON and fixed error
  messages.
- Repository contract changes must be covered in `nako-db/src/contract_tests.rs`
  or the focused adapter tests.
- Cross-crate contract changes should run at least:
  `cargo check -p nako-core --tests` and the downstream package checks that use
  the changed record or trait.

## Review Checklist

- Does the name match `CONTEXT.md`?
- Is this a domain contract rather than app/runtime/adapter behavior?
- Are list APIs paginated?
- Are unknown persisted values handled explicitly?
- Did every affected adapter, DTO, and test get updated?
