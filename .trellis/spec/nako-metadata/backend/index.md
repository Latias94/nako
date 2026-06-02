# nako-metadata Backend Development Guidelines

These specs document metadata provider, strategy, mapping, hierarchy
confirmation, and Candidate Review behavior in `crates/nako-metadata`.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding provider,
  mapping, strategy, or review modules.
- Read [Database Guidelines](./database-guidelines.md) before adding repository
  interactions or persistence-facing metadata behavior.
- Read [Error Handling](./error-handling.md) before changing provider failures,
  stale review operations, or status transitions.
- Read [Quality Guidelines](./quality-guidelines.md) before adding metadata
  application, Provider Mapping, or hierarchy confirmation logic.
- Read [Logging Guidelines](./logging-guidelines.md) before adding provider
  diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Providers, mappings, strategy, runtime, Candidate Review services | Filled from code |
| [Database Guidelines](./database-guidelines.md) | Repository-trait usage without direct DB adapter dependency | Filled from code |
| [Error Handling](./error-handling.md) | Provider errors, stale review conflicts, NotFound behavior | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Plan-before-apply metadata governance and tests | Filled from code and ADRs |
| [Logging Guidelines](./logging-guidelines.md) | Provider diagnostic redaction | Filled as constrained boundary |

## Authority / Evidence

- `CONTEXT.md` metadata vocabulary.
- ADR 0007: metadata merge policy and local authority.
- ADR 0018: metadata provider runtime and diagnostics.
- `crates/nako-metadata/src/candidate_review.rs`
- `crates/nako-metadata/src/confirmation.rs`
- `crates/nako-metadata/src/providers/*.rs`
- `crates/nako-metadata/src/mapping/*.rs`
- `crates/nako-metadata/src/strategy.rs`
