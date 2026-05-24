# Addon Outbound Task Dispatch Credentials - Handoff

Status: Active
Last updated: 2026-05-24

## Current State

This lane has been opened to close the outbound auth gap left by the host-owned
Addon Task runtime. The direct dispatch path already exists, but it currently
passes `None` to the Addon client, so authenticated sidecars cannot be called
through the host-owned dispatch path without a credential-management layer.

`crates/nako-addon-client/src/lib.rs` already knows how to emit `Authorization:
Bearer` and `x-nako-addon-secret` headers from a supplied token. The host-side
storage/resolution boundary now exists as `outbound_task_dispatch_secret_env`
on addon registrations plus `resolve_outbound_task_dispatch_secret` in
`crates/nako-server/src/app/addons.rs`. Direct dispatch now wires that helper
into `crates/nako-server/src/app/addons/task_runtime.rs` and injects the
resolved outbound credential when the manifest declares bearer or shared
secret auth.

## Next Task

- AOTDC-060

## Known Risks

- Storing raw secret material would leak into the wrong boundary; prefer
  environment-variable references and redaction-safe diagnostics.
- If env-backed references are too weak for real deployments, split a
  follow-on vault/provider lane instead of widening this one.
- Direct dispatch must stay host-owned; do not move scheduling, retry, or
  result handling into the sidecar.
