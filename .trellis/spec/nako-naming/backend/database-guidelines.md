# Database Guidelines

`nako-naming` is a pure parsing crate. It has no database layer and should not
learn about persisted catalog state.

## Required Patterns

- Return serializable parsing evidence to callers.
- Let library ingestion or metadata workflows decide whether to persist parser
  output.
- Keep parser version explicit so stored inference evidence can be interpreted
  later.
- Keep source paths as input strings; do not resolve or query source records.

## Forbidden Patterns

- Do not depend on `nako-db`, repository traits, metadata providers, catalog
  graph code, or server state.
- Do not perform lookups to confirm movie, series, season, or episode identity.
- Do not mutate catalog records, ingestion state, or source observations.
- Do not introduce parser caches that affect deterministic output.

## Review Checklist

- Is this still a pure function of path text and parser version?
- Does a persistence concern belong in `nako-library` instead?
- Does a metadata concern belong in `nako-metadata` instead?
- Can unit tests exercise the behavior without repositories or fixtures?

## Evidence

- `crates/nako-naming/Cargo.toml`
- `crates/nako-naming/src/lib.rs`
