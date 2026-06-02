# Directory Structure

`nako-addon-client` currently keeps client orchestration in `src/lib.rs`. Split
only when transport, resource helpers, task/event helpers, or Nako runtime calls
grow enough to need private modules.

## Current Layout

- HTTP request/response DTOs used by `AddonTransport`.
- `AddonClientError`, result aliases, and safe diagnostics.
- Outcome/failure structs for resource, specialized resource, task, and event
  calls.
- `AddonTransport` and `ReqwestAddonTransport`.
- Resource, task, event, health, and `NakoRuntimeClient` call helpers.
- Private schema, retry, URL, and unsafe-body helpers.
- Mock transport tests.

## Module Split Rules

- Keep `AddonTransport` close to request/response structs.
- Move reqwest-only code to a transport module before adding another transport.
- Keep specialized resource helpers next to schema constants they enforce.
- Keep Nako runtime side-effect calls separate from sidecar resource calls if
  the file is split.

## Naming Rules

- Use `call_addon_*_with_outcome` when attempts and HTTP status matter.
- Use `Addon*CallOutcome` and `Addon*CallFailure` for delivery result wrappers.
- Use `NakoRuntimeClient` only for Addon Sidecar calls back to Nako runtime APIs.

## Anti-Patterns

- Do not add server job scheduler code here.
- Do not put manifest builder facts here.
- Do not create hidden global reqwest clients in helper functions.
