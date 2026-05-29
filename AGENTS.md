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
- Current system maps live in `docs/ARCHITECTURE.md` and
  `docs/architecture/`.
- Workstream planning lives in `docs/workstreams/`.
- When a change crosses crate boundaries, changes public API shape, changes
  storage/schema behavior, or changes resource/concurrency policy, update the
  relevant ADR or workstream docs before considering the work complete.
- Before changing playback, storage/VFS, library intake, state/access,
  realtime/sync, operations/release, or control-plane behavior, read the
  matching `docs/architecture/*.md` map and the ADRs it references.
- Treat ADR 0053 as the control-plane baseline. New durable jobs, runtime
  supervision, tracing/diagnostics, remote access, addon lifecycle, or API
  cache/scale behavior should not be hidden inside a one-off feature helper.
- M27.0 is a design baseline only. Do not add schema migrations, provider
  features, runtime behavior, or public API changes under M27.0.

## Rust Workspace Rules

- Use `cargo fmt --all` for formatting when practical.
- Prefer `cargo nextest run` for tests. For narrow changes, run focused
  package tests first, then broaden only when risk requires it.
- Keep crate boundaries aligned with the workstream docs:
  - `nako-core`: domain records, IDs, repository traits.
  - `nako-db`: schema, migrations, SQLite/PostgreSQL adapters, repository
    contract tests.
  - `nako-vfs`: storage backend interfaces, local/WebDAV adapters, cache and
    staging primitives.
  - `nako-library`: scan, index, probe orchestration, local inference, and
    library ingestion workflow.
  - `nako-media-probe`: media technical fact extraction from VFS-backed
    sources.
  - `nako-naming`: file-name and path parsing helpers.
  - `nako-metadata`: provider runtimes, provider adapters, metadata strategy,
    mapping, and hierarchy confirmation.
  - `nako-nfo`: NFO codec, import/export, preview, sidecar workflow, and
    round-trip behavior.
  - `nako-catalog`: catalog graph hydration and search projection.
  - `nako-search`: search document and projection primitives.
  - `nako-playback`: pure playback decision planning, policy/capability
    matching, and selected playback requirements. Do not put FFmpeg process
    execution here.
  - `nako-transcode`: FFmpeg command planning, hardware capability inventory,
    transcode/remux/HLS request and artifact modeling, and transcode runtime
    policy.
  - `nako-streaming`: direct byte/range transport and streaming response
    mechanics.
  - `nako-events`: event/webhook envelopes, signing, delivery attempts, and
    event transport helpers.
  - `nako-automation`: automation provider configuration and automation job
    workflow.
  - `nako-addon-protocol`: permissive addon manifest, resource, event, task,
    health, hosted-surface, and permission contract.
  - `nako-addon-client`: client adapter for calling Addon Sidecars.
  - `nako-official-addon-catalog`: official addon catalog descriptors and
    manifest/package facts.
  - `nako-reference-addon`: local reference addon fixture.
  - `nako-client-protocol`: permissive Public Client wire contract and route
    inventory. Keep it dependency-light.
  - `nako-client-core`: transport-neutral client request builders and response
    mapping.
  - `nako-client`: async HTTP client adapter.
  - `nako-client-cli`: CLI client surface.
  - `nako-client-uniffi`: UniFFI client binding surface.
  - `nako-server`: composition, app orchestration, HTTP boundaries.
  - `nako-api`: explicit DTOs and public API shapes.
  - `nako`: package-level addon/protocol convenience crate.
  - `nako-uniffi-bindgen`: UniFFI binding generation helper.
- Prefer internal module deepening before adding a new crate. Add a crate only
  when multiple real callers or adapters prove the seam is useful.
- Production dependencies should flow toward `nako-core`, protocol crates, and
  implementation adapters. Dev-dependencies on `nako-db` for workflow tests do
  not by themselves mean the production seam is wrong.
- Long-running or important background work belongs behind durable job or
  runtime-supervisor interfaces. Do not add raw `tokio::spawn` work for scan,
  metadata, playback, addon, webhook, or artifact workflows without checking
  ADR 0053.

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
