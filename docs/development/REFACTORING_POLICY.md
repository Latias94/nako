# Refactoring Policy

Taru should be refactored early and deliberately while the server architecture
is still cheap to reshape. The goal is not churn; the goal is to keep module
boundaries honest before storage, metadata, playback, transcode, search, and
addons become harder to separate.

## Principles

- Prefer correct boundaries over preserving accidental early structure.
- Refactor when a goal exposes the wrong abstraction, not after every feature.
- Keep behavior covered before moving code across crates.
- Delete obsolete code when a replacement is complete and no compatibility
  promise depends on it.
- Keep public HTTP and storage contracts more stable than internal Rust APIs.
- Document architecture changes in ADRs or workstream phase notes.

## Crate Boundary Rules

`taru-core`:

- owns shared domain IDs, common errors, and cross-crate domain primitives;
- must not depend on infrastructure crates;
- should not become a dumping ground for feature-specific logic.

Infrastructure and adapters:

- `taru-db` owns SQLite schema, migrations, repositories, and transaction
  boundaries.
- `taru-vfs` owns storage backend contracts and local/remote file access.
- `taru-search` owns search adapter contracts and fallback implementations.
- `taru-media-probe` owns probe execution and normalized probe output.
- `taru-transcode` owns FFmpeg command planning, process running, session
  lifecycle primitives, and hardware policy.

Domain services:

- `taru-library`, `taru-catalog`, `taru-metadata`, `taru-nfo`,
  `taru-streaming`, `taru-events`, `taru-automation`, and
  `taru-addon-protocol` own domain-specific orchestration and contracts.
- Domain crates can depend on lower-level infrastructure only when the boundary
  is already explicit and tested.

Composition:

- `taru-server` owns binary bootstrap, runtime configuration, dependency
  assembly, HTTP handler wiring, and application services that coordinate
  multiple domain crates.
- `taru-api` owns HTTP DTOs and response envelopes. It should not contain
  process orchestration or database transactions.

## Dependency Direction

Default direction:

```text
taru-server
  -> taru-api
  -> domain service crates
  -> infrastructure adapter crates
  -> taru-core
```

Allowed shortcuts must be intentional. For example, a domain crate may use a
repository from `taru-db` while the project is still a modular monolith, but
the repository trait or service boundary must remain visible enough that it can
be inverted later.

Avoid:

- `taru-core` depending on feature crates;
- HTTP handlers calling FFmpeg or SQLite directly;
- metadata providers mutating catalog/search state without a service boundary;
- VFS users falling back to raw `std::fs` paths except inside local backend
  implementations;
- transcode code assuming local files when a staged VFS path is required.

## When To Refactor

Refactor before or during a goal when:

- a new feature would duplicate orchestration logic already present elsewhere;
- a handler, repository, or provider starts owning policy from another domain;
- a crate must depend on a higher-level crate to finish the feature;
- test setup becomes a signal that the boundary is too implicit;
- cancellation, retry, timeout, or resource budgeting cannot be expressed at
  the current layer;
- the implementation would make remote storage, HLS, addons, or future clients
  harder to add.

Defer refactoring when:

- the boundary is speculative and no near-term goal needs it;
- the change only renames files without clarifying behavior;
- a smaller compatibility shim can isolate the risk until the next milestone;
- validation would become weaker because behavior is not yet covered.

## Internal API Stability

Internal Rust APIs may change aggressively before a stable release. The rules
are:

- keep changes scoped to the active goal;
- update all call sites in the same commit;
- remove obsolete APIs once call sites are migrated;
- add tests around the behavior being moved;
- update docs when the boundary or ownership changes.

External API surfaces are stricter:

- HTTP routes, response envelopes, config keys, database migrations, provider
  manifest formats, and storage URI schemes should only change with explicit
  documentation.
- Breaking changes before the first release are allowed, but they must be
  visible in phase notes or ADRs.

## Validation Gates

For meaningful Rust changes, run:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
git diff --check
```

For docs-only changes, run at least:

```powershell
git diff --check
```

When a refactor changes dependency direction, also inspect crate manifests and
update the relevant roadmap, ADR, or workstream document.

## Review Checklist

Before committing a refactor:

- Can the new boundary be explained in one sentence?
- Did any crate gain an upward dependency?
- Did HTTP code stay thin?
- Did database code stay behind repository or service boundaries?
- Did VFS abstractions remain the path for file access?
- Are cancellation, timeout, retry, and resource limits still explicit?
- Are tests covering behavior rather than only type compilation?
- Are obsolete code paths deleted?
