# Server Architecture Hardening Workstream

## Purpose

This workstream turns the server from an MVP-shaped application surface into a
clean modular-monolith composition boundary. It owns the refactor that makes
`taru-server` thin, moves orchestration into focused services, gives background
work explicit lifecycle ownership, tightens repository and transaction
boundaries, and removes obsolete helper paths.

## Status

Completed as M24.

M24 was intentionally architecture-first. It prepared the server for
metadata provider expansion, future clients, stronger NFO handling, network
storage growth, and AI/automation surfaces without adding new product features
in the same slice.

Top-level tracking:

- [Goal map](../../GOALS.md)
- [Roadmap](../../ROADMAP.md)
- [ADR 0019: server architecture hardening boundaries](../../adr/0019-server-architecture-hardening-boundaries.md)
- [Milestones](MILESTONES.md)
- [TODO](TODO.md)
- [Phase 24.0 baseline](PHASE24_0_SERVER_ARCHITECTURE_BASELINE.md)
- [Phase 24.1 implementation slice](PHASE24_1_IMPLEMENTATION_SLICE.md)

## Goals

- Make `taru-server::app::TaruApp` a thin composition root instead of a
  feature orchestration object.
- Move workflow logic into focused application services with explicit
  constructors and narrow dependencies.
- Introduce an explicit runtime supervisor or worker registry for background
  jobs, cleanup loops, scheduled maintenance, and detached task ownership.
- Keep high-level server services behind repository traits, ports, or focused
  service handles instead of passing around broad concrete stores.
- Move multi-record atomicity into repository or unit-of-work boundaries.
- Delete obsolete MVP helpers after their replacement invariants are in place.
- Keep HTTP route modules thin: extraction, validation, app-service call,
  response/error translation.

## Non-Goals

- No new metadata provider feature work.
- No Flutter, web, or mobile client implementation.
- No split into multiple deployable services.
- No in-process plugin ABI design.
- No adaptive bitrate playback ladder.
- No compatibility shims for deprecated config or route shapes unless they are
  explicitly justified by a testable migration need.

## Boundary Rules

- `taru-server` composes runtime pieces and owns HTTP integration.
- Application services own workflows and depend on narrow ports.
- `taru-db` owns SQLite, SQL mapping, and transaction details.
- `taru-core` owns stable domain types and repository contracts.
- `taru-api` owns public request/response DTOs.
- `taru-vfs`, `taru-metadata`, `taru-nfo`, `taru-catalog`, and
  `taru-transcode` should expose domain-specific capabilities instead of
  relying on server internals.
- Background workers must be registered and supervised; direct detached
  `tokio::spawn` calls are not allowed in feature code without an explicit
  supervisor boundary.

## Refactor Policy

Prefer the clean target architecture over preserving early shortcuts. If a
helper exists only because the MVP had one library, one store, one route file,
or no lifecycle manager, remove it after the replacement path is covered.

Do not stage a partial refactor that leaves both old and new orchestration
paths alive unless the next patch deletes one of them. Temporary duplication
must be visible in `TODO.md` with an owner phase.

## Related Workstreams

- [runtime-foundation](../runtime-foundation/README.md): shared runtime,
  database, secret, resource, and lifecycle boundaries.
- [metadata-operations](../metadata-operations/README.md): provider runtime,
  maintenance scheduling, diagnostics, and raw cache lifecycle.
- [playback-streaming](../playback-streaming/README.md): playback app and HTTP
  service split, staging, and resource budgets.
- [server-foundation](../server-foundation/README.md): historical foundation
  notes and earlier decomposition phase records.
