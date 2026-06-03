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
