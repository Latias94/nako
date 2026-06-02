# Database Guidelines

`nako-addon-client` does not own persistence. It returns protocol-validated
outcomes and failures so server/control-plane code can persist attempts, jobs,
audit records, or side effects.

## Required Patterns

- Accept manifests, granted scopes, request IDs, payloads, and tokens from
  callers.
- Return attempt counts and HTTP status in outcome wrappers.
- Return setup failures with `attempts: 0`.
- Let server workflows record durable task/event/resource attempts.
- Keep Addon Token lookup and rotation outside this crate.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not fetch Addon Tokens from storage.
- Do not write audit, side-effect, or generated-artifact rows here.
- Do not infer accepted grants from manifest declarations.

## Contract Rules

- Scope checks call `ensure_scope_grant`, `ensure_task_scope_grant`, or
  `ensure_event_subscription_scope_grant`.
- Resource declaration timeout and max attempts override manifest defaults.
- `NakoRuntimeClientConfig` carries base URL, addon token, and timeout for
  addon-to-Nako calls, but persistence remains in Nako runtime APIs.

## Tests Required

- Fake transport tests for attempts and headers.
- Setup-failure tests with `attempts: 0`.
- Server-side persistence tests must live in server/database crates.
