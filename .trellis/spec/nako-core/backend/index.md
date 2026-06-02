# nako-core Backend Development Guidelines

These specs describe the current `crates/nako-core` boundary: domain records,
strong IDs, repository traits, and shared error types. `nako-core` is not a
runtime, database adapter, HTTP layer, or storage backend.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding a module,
  record, ID, enum, or repository trait.
- Read [Database Guidelines](./database-guidelines.md) before adding persistence
  contract fields or repository methods.
- Read [Error Handling](./error-handling.md) before adding validation,
  persisted enum parsing, stale-write handling, or repository contract errors.
- Read [Quality Guidelines](./quality-guidelines.md) before changing cross-crate
  domain contracts.
- Read [Logging Guidelines](./logging-guidelines.md) before adding diagnostics
  to core types.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Domain records, media submodules, repository traits, strong IDs | Filled from code and AGENTS |
| [Database Guidelines](./database-guidelines.md) | Persistence contracts without DB implementation leakage | Filled from code and ADRs |
| [Error Handling](./error-handling.md) | Shared `NakoError` usage, persisted enum parse behavior, stale writes | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Cross-crate contract rules and forbidden dependencies | Filled from code and repo gates |
| [Logging Guidelines](./logging-guidelines.md) | Core diagnostic constraints | Filled as no-runtime/logging boundary |

## Authority / Evidence

- `AGENTS.md` Rust workspace rules.
- `CONTEXT.md` project vocabulary.
- ADR 0001: modular monolith Rust workspace.
- ADR 0021: video-first media server domain model.
- ADR 0053: application control-plane boundary.
- `crates/nako-core/src/lib.rs`
- `crates/nako-core/src/id.rs`
- `crates/nako-core/src/media/*.rs`
- `crates/nako-core/src/repository/*.rs`
- `crates/nako-core/src/job.rs`
