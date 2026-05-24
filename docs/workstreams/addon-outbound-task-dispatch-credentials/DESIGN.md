# Addon Outbound Task Dispatch Credentials

Status: Active
Last updated: 2026-05-24

## Problem

`call_addon_task_with_outcome` already knows how to send `Authorization:
Bearer <token>` or `x-nako-addon-secret`, but the server-side direct-dispatch
path currently passes `None` for the outbound credential.

That means authenticated sidecars cannot be reached through the host-owned
task runtime without a safe storage and resolution boundary for the dispatch
credential.

## Target State

Nako should be able to:

- persist an outbound task-dispatch credential env reference for an addon
  registration;
- inject the resolved credential into direct task dispatch when the manifest
  declares `AddonAuth::Bearer` or `AddonAuth::SharedSecret`;
- keep the raw secret out of public API responses, logs, and install guides;
- return a safe, actionable diagnostic when the credential env reference is
  missing or empty;
- leave inbound Addon Token auth, Addon Manager lifecycle, marketplace,
  package signing, provider breadth, and process supervision outside the lane.

## Scope

- Outbound credential env-reference model and storage shape.
- Host-side resolution for direct Addon task dispatch.
- Header injection for bearer and shared-secret auth.
- Redaction-safe diagnostics and tests.
- Documentation and closeout evidence.

## Non-Goals

- Inbound Addon Token lifecycle or grant management.
- Addon Manager discovery/install/update/rollback.
- Marketplace/source catalog behavior.
- Package signing/trust roots.
- Provider breadth.
- Official-addon task-path smoke.
- Direct process/container supervision.
- A general-purpose vault or OAuth-first auth platform in the first slice.

## Assumptions

- The first slice stores an environment-variable name in
  `outbound_task_dispatch_secret_env` and resolves it at dispatch time.
- The host remains the caller that owns task dispatch and credential
  resolution.
- If the first credential store proves too narrow, a separate secret-provider
  lane can be split later.

## Architecture Direction

Keep `AddonAuth` as the protocol contract and add the smallest host-owned
credential layer necessary to resolve it. Prefer storing environment-variable
references and resolving them at dispatch time over persisting raw secret
material.

The current implementation stores `outbound_task_dispatch_secret_env` on the
addon registration record and resolves it from the host environment immediately
before direct dispatch. Missing or empty env values fail with a redaction-safe
configuration error.

The direct-dispatch path should stay responsible for scheduling, retries, and
result handling. This lane only fills the outbound auth gap required by
authenticated sidecars.

## Related Docs

- `docs/workstreams/addon-task-runtime-contract/DESIGN.md`
- `docs/workstreams/addon-task-runtime-contract/HANDOFF.md`
- `docs/workstreams/addon-source-catalog-marketplace/DESIGN.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
