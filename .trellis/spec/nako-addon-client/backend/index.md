# nako-addon-client Backend Guidelines

`nako-addon-client` is the Rust HTTP caller helper for Addon Sidecars. It builds
on `nako-addon-protocol`, keeps transport mockable, validates protocol
envelopes, and redacts unsafe request/transport details.

## Current Evidence

- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-addon-client/README.md`
- `crates/nako-addon-protocol/src/lib.rs`

## Boundaries

- Build outbound addon HTTP requests and parse addon responses.
- Expose `AddonTransport` for fake and reqwest-backed transports.
- Handle resource, resource-search, resource-link-check, subtitle, task, event,
  health, and Nako runtime side-effect calls.
- Keep durable job persistence, addon registration, and permission storage
  outside this crate.
- Keep protocol wire definitions in `nako-addon-protocol`.

## Executable Contract Summary

1. Scope / Trigger: any new addon runtime call, specialized resource helper, or
   retry/redaction behavior update belongs here.
2. Signatures: public calls such as `call_addon_resource_with_outcome`,
   `call_addon_task_with_outcome`, `call_addon_event_with_outcome`,
   `check_addon_health`, and `NakoRuntimeClient`.
3. Contracts: requests include protocol, addon ID, resource/task/event facts,
   request ID, `x-nako-attempt`, and auth headers based on `AddonAuth`.
4. Validation & Error Matrix: invalid manifest, missing scope, missing token,
   invalid schema, non-2xx HTTP, invalid response, and unsafe request body map to
   `AddonClientError` or outcome failures.
5. Good/Base/Bad Cases: good calls return outcome with status and attempts; base
   calls use manifest or declaration timeout/defaults; bad calls never expose
   token material in errors.
6. Tests Required: mock transport request assertions, retry attempts, schema
   validation, unsafe body rejection, health checks, and safe error messages.
7. Wrong vs Correct: do not call reqwest directly from server workflows; call
   through `AddonTransport` so protocol checks and redaction run.

## Required Patterns

- Validate manifests and scope grants before dispatch.
- Resolve timeout and max attempts from declaration first, then manifest
  defaults, then safe crate defaults.
- Retry only transport errors and retryable statuses: 408, 429, and 5xx.
- Validate specialized request/response schemas for resource-search,
  resource-link-check, subtitle, and external acquisition materialization.
- Use `safe_code`, `kind`, and outcome attempt counts for caller diagnostics.

## Forbidden Patterns

- Do not expose bearer tokens, shared secrets, request URLs, query tokens, or
  unsafe bodies in errors.
- Do not bypass `validate_resource_response`, `validate_task_response`,
  `validate_event_response`, or `validate_health_check_response`.
- Do not persist jobs or attempts here.
- Do not make reqwest the only testable transport path.

## Validation

- Focused:
  `cargo nextest run -p nako-addon-client --no-fail-fast`
- Protocol contract:
  `cargo check -p nako-addon-client -p nako-addon-protocol --tests`
