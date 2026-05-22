# OpenAPI And Public Client SDK Contract Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M32 is closed. Nako now has a first machine-readable Public Client API
OpenAPI v1 generator and checker.

## Completed Scope

- `nako-client-protocol` owns public playback session response DTOs.
- Public playback session responses no longer include server-local
  `output_path`.
- `nako-api` generates OpenAPI v1 JSON and owns schema aggregation.
- `nako-server` exposes `GET /libraries/{library_id}` so the public library
  surface has list and detail routes.
- OpenAPI checker covers public route inventory, bearer auth, API version
  header, error envelope, pagination parameters, and leakage rejection.

## Decisions Since Last Update

- OpenAPI aggregation belongs in `nako-api`, not `nako-server`.
- Public wire DTOs belong in `nako-client-protocol`.
- Server-admin/internal routes are excluded from the first public spec.
- Playback session `output_path` must not be part of the public client schema.
- The first OpenAPI artifact is emitted by
  `cargo run -p nako-api --example emit-openapi`.

## Blockers

- None.

## Follow-Ons

- Add an optional server route for serving the OpenAPI JSON if operators need
  runtime discovery.
- Add SDK generation jobs for Dart/Flutter, TypeScript, or CLI clients.
- Add a separate admin/internal OpenAPI contract if admin tooling needs one.
- Keep user accounts, sessions, OAuth/OIDC, RBAC, and concrete client apps as
  separate future goals.
