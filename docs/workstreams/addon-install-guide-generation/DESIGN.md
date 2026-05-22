# Addon Install Guide Generation Design

Status: Completed
Last updated: 2026-05-22

## Why This Lane Exists

Nako now has a safe Admin Addon Operations surface: operators can register an
**Addon Sidecar**, enable or disable it, run an **Addon Health Check**, inspect
manifest surfaces, and diagnose resource calls. The next product gap is not
process automation; it is helping an operator correctly run the Addon Sidecar
outside Nako.

Without a first-class **Addon Install Guide**, operators must infer deployment
shape from a manifest and docs. That encourages ad-hoc Docker Compose snippets,
plaintext secrets, confusing health-check commands, and accidental expectation
that Nako should manage containers.

## Relevant Authority

- `CONTEXT.md`: defines **Addon**, **Addon Sidecar**, **Addon Install Guide**,
  **Addon Manager**, **Addon Health Check**, and **Secret Reference**.
- `docs/workstreams/admin-addon-operations-mvp/`: backend lifecycle, health,
  surfaces, diagnostics, token, and grant boundaries.
- `docs/workstreams/admin-web-addon-operations/`: Admin Web Addon Operations
  read model and UI seam.
- `docs/guides/ADDON_AUTHOR_GUIDE.md`: Addon author protocol guidance.
- `docs/api/HTTP_API.md`: Admin Addon API route documentation.

## Problem

Nako can operate registered **Addon Sidecars**, but it does not yet produce a
canonical, redaction-safe deployment guide from the stored manifest snapshot.
The missing boundary has three consequences:

- operators cannot quickly see the expected sidecar URL, health endpoint,
  required scopes, secret references, and verification commands in one place;
- frontend-only guide text would drift from server-side Addon semantics;
- any future install experience may accidentally become an **Addon Manager**
  unless we freeze the non-goal now.

## Target State

When this lane closes:

- `nako-api` exposes a generated Admin API TypeScript contract for
  `GET /admin/v1/addons/{addon_id}/install-guide`;
- `nako-server` generates an `AdminAddonInstallGuideResponse` from the stored
  manifest without network calls, secret resolution, Docker socket access, or
  process lifecycle operations;
- the guide includes Docker Compose, systemd, Secret Reference checklist,
  health-check verification, and Admin registration verification sections;
- `apps/admin-web` renders the guide as a safety-bounded preview in the Addon
  Operations panel with safe mock fallback;
- docs and tests prove the guide does not leak raw tokens, resolved secrets,
  local filesystem paths, Source Locators, storage URIs, or process-control
  promises.

## In Scope

- Admin-only read route:
  `GET /admin/v1/addons/{addon_id}/install-guide`.
- Redaction-safe DTOs in `nako-api::extension` and generated Admin Web
  TypeScript contract.
- Pure server-side guide composition from registered Addon manifest facts.
- Docker Compose snippet, systemd unit snippet, Secret Reference checklist,
  Addon Health Check verification, and registration verification steps.
- Admin Web client/data-source/mock/UI integration.
- API docs, Addon author guide updates, workstream closeout evidence.

## Out Of Scope

- No Addon discovery, marketplace, package signing, install, update, remove,
  start, stop, restart, log, or process supervision.
- No Docker socket, Kubernetes, systemd D-Bus, SSH, or host-agent adapter.
- No storage of resolved secret values and no secret-manager integration.
- No Addon Protocol version bump unless validation proves it is unavoidable.
- No Public Client API exposure.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The install guide can be derived from the stored manifest snapshot and Addon registration summary. | High | Addon Operations already renders manifest, surfaces, tokens, and grants from safe DTOs. | If more data is required, split follow-on rather than adding manager state in this lane. |
| Docker Compose and systemd snippets are useful as text previews even if Nako does not execute them. | High | `CONTEXT.md` explicitly defines **Addon Install Guide** as generated deployment snippets. | If operators need real lifecycle automation, open a separate **Addon Manager** lane. |
| Secret Reference fields are declarations only and must remain unresolved in the guide. | High | Existing Admin surface exposes counts/labels only and rejects raw secrets. | If secrets must be resolved, create a separate secret-runtime design with stronger policy. |
| Admin Web should consume a server-owned guide instead of re-deriving snippets in TypeScript. | High | Generated Admin API contract is already the anti-drift seam for Admin Web Addons. | Frontend-only generation would likely diverge from server-side route semantics. |

## Architecture Direction

The correct boundary is a server-owned Admin read model:

- `nako-api` owns the response DTO shape and generated TypeScript contract.
- `nako-server::app::addons` owns guide composition because it already owns
  stored manifest parsing and Addon status policy.
- `crates/nako-server/src/http/addons.rs` stays thin and only exposes the
  route.
- `apps/admin-web/src/adminApi` maps the wire response into a UI read model.
- `App.tsx` renders snippets as inert text. It does not execute commands,
  write files, or talk to Docker/systemd.

This keeps future **Addon Manager** work honest: the guide is a read-only
operator aid, while lifecycle automation remains a separately reviewed
product/architecture lane.

## Closeout Condition

This lane can close when:

- the Admin API route and DTO are implemented and documented;
- Admin Web can render live/mock guide previews safely;
- focused Rust and Admin Web tests pass;
- final evidence is recorded in `EVIDENCE_AND_GATES.md`;
- `docs/GOALS.md`, `docs/ROADMAP.md`, and `docs/workstreams/README.md` reflect
  the shipped behavior and next recommended follow-on.

Closeout status: completed on 2026-05-22. The shipped route and Admin Web
surface satisfy the target state without adding Addon Manager lifecycle
automation.
