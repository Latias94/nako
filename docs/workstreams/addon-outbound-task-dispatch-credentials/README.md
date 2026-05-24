# Addon Outbound Task Dispatch Credentials

Status: Completed
Last updated: 2026-05-25

This workstream added storage and resolution for outbound task-dispatch
credentials for addon sidecars that declare `AddonAuth::Bearer` or
`AddonAuth::SharedSecret`.

The host-owned task runtime already exists. Direct dispatch now resolves an
environment-variable credential reference stored on the addon registration and
passes the resolved credential to the Addon client at dispatch time.

## Problem

Nako could dispatch Addon Tasks directly, but the direct-dispatch path passed no
outbound credential. That left `Bearer` and `SharedSecret` sidecars without a
host-owned way to receive authenticated task calls.

## Target State

This lane closed after Nako could:

- store or resolve an outbound task-dispatch credential env reference for an
  addon without exposing raw secret values in public APIs or logs;
- inject the correct outbound auth headers or secret tokens when direct task
  dispatch calls a `Bearer` or `SharedSecret` sidecar;
- keep the host-owned task runtime boundary intact;
- return redaction-safe diagnostics when a required outbound credential is
  missing or cannot be resolved;
- keep inbound Addon Token auth, Addon Manager lifecycle, marketplace, package
  signing, provider breadth, and process supervision as separate lanes.

## Scope

- Outbound credential storage and resolution for direct Addon task dispatch.
- Host-side dispatch injection for `AddonAuth::Bearer` and
  `AddonAuth::SharedSecret`.
- Redaction-safe diagnostics and tests for missing or unresolved credentials.
- Documentation and validation for the new outbound auth boundary.

## Non-Goals

- Inbound Addon Token authorization.
- Addon Manager discovery, install, update, or rollback.
- Marketplace, package signing, trust roots, or provider breadth.
- Official-addon task-path smoke.
- Direct process/container supervision.
- OAuth-first or external vault integration in the first slice unless the
  storage model forces a separate split.

## Architecture Direction

Keep `AddonAuth` as the protocol-level outbound auth contract. This lane should
not invent a new auth vocabulary; it should define how Nako stores, resolves,
and injects the credential that a manifest already requires. The current slice
uses `outbound_task_dispatch_secret_env` as the host-owned reference.

The shipped slice stores `outbound_task_dispatch_secret_env` on the addon
registration and resolves it on the host side at dispatch time. Diagnostics say
whether a credential reference is present or missing without echoing the secret
itself.

## Closeout

Closed on 2026-05-25 after fresh compile, formatting, Addon client, Addon
server, focused direct-dispatch, and diff gates passed. A richer vault or
secret-provider abstraction remains a separate follow-on if env-backed
references prove too narrow.

## Related Docs

- `docs/workstreams/addon-task-runtime-contract/`
- `docs/workstreams/addon-source-catalog-marketplace/`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
