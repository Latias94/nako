# AGENTS.md

This file gives repository-local guidance for agents working on Nako.

## Project Language

- Read `CONTEXT.md` before changing domain model, API, metadata, catalog,
  addon, playback, or storage behavior.
- Use the terms from `CONTEXT.md` in docs and code discussions. Do not drift
  back to provider-centric or file-centric names when a Nako term exists.
- Keep `CONTEXT.md` as a glossary only. Do not put implementation plans,
  schemas, workstream notes, or ADR content in it.

## Architecture Records and Workstreams

- Durable architecture decisions live in `docs/adr/`.
- Workstream planning lives in `docs/workstreams/`.
- When a change crosses crate boundaries, changes public API shape, changes
  storage/schema behavior, or changes resource/concurrency policy, update the
  relevant ADR or workstream docs before considering the work complete.
- M27.0 is a design baseline only. Do not add schema migrations, provider
  features, runtime behavior, or public API changes under M27.0.

## Rust Workspace Rules

- Use `cargo fmt --all` for formatting when practical.
- Prefer `cargo nextest run` for tests. For narrow changes, run focused
  package tests first, then broaden only when risk requires it.
- Keep crate boundaries aligned with the workstream docs:
  - `nako-core`: domain records, IDs, repository traits.
  - `nako-db`: schema, migrations, repository adapters.
  - `nako-metadata`: provider adapters and provider payload mapping.
  - `nako-nfo`: NFO parsing/export and round-trip behavior.
  - `nako-catalog`: graph hydration and search projection.
  - `nako-server`: composition, app orchestration, HTTP boundaries.
  - `nako-api`: explicit DTOs and public API shapes.

## Reference Code and Licensing

- Nako server-side code is AGPL-3.0-or-later unless a crate or file says
  otherwise.
- `nako-addon-protocol` is intended to remain permissive for addon authors.
- Repositories under `repo-ref/` are reference material only. Do not copy,
  translate line by line, or import source, comments, migrations, tests,
  schemas, assets, or generated code from Jellyfin or other reference projects.
- Use reference projects to study behavior, architecture boundaries, and user
  workflows, then write original Nako implementations against Nako's own
  domain model and tests.
