# Admin Web V2 Settings Mutation Authority

Status: Closed
Last updated: 2026-05-26

## Why This Lane Exists

Admin Web V2 now has a route-first `/settings` page, but it is intentionally
read-only. The broader Admin Web V2 goal requires settings editing. That cannot
be achieved by adding local form state or a mock save button, because the
current Admin API exposes only redacted diagnostics for system configuration.

This lane exists to define the mutation authority, persistence semantics,
redaction rules, and first safe editable slice before any settings UI claims to
change server behavior.

## Relevant Authority

- Glossary:
  - `CONTEXT.md`
- ADRs:
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0009-resolve-provider-secrets-from-environment.md`
- Existing workstreams:
  - `docs/workstreams/admin-web-v2-system-settings-route/`
  - `docs/workstreams/admin-web-v2-library-management-and-localization/`
  - `docs/workstreams/admin-library-metadata-profile-configuration/`
  - `docs/workstreams/metadata-profile-configuration-authority/`
  - `docs/workstreams/network-access-boundary/`
- Current code:
  - `apps/admin-web/src/features/settings/SettingsPage.tsx`
  - `apps/admin-web/src/adminApi/client.ts`
  - `apps/admin-web/src/adminApi/dataSource.ts`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/config.rs`
  - `docs/api/HTTP_API.md`

## Problem

The settings route displays safe summaries from
`GET /admin/v1/system/config`. Operators still cannot edit global settings such
as network exposure policy, trusted proxy behavior, worker budgets, or runtime
policy from Admin Web V2.

The risky part is not the form. The risky part is deciding what owns a setting
after startup:

- `NakoServerConfig` is loaded from TOML at process start.
- `crates/nako-server/src/config.rs` has `load_config()` and
  `example_config()`, but no accepted save/update authority for running system
  config.
- Media Library Metadata Profile updates already use a narrower persisted
  library-options authority, but that does not generalize to raw system config.
- The existing `/settings` page is deliberately redaction-safe and must not
  expose URLs, host names, filesystem paths, roots, env var names, credentials,
  tokens, provider secrets, or raw config text.

Without a bounded authority model, a settings form would either mutate only
frontend state, silently drift from TOML on restart, or expose sensitive config
material.

## Target State

When this lane closes:

- The current settings mutation surface is audited and documented in
  `ROUTE_API_READINESS.md`.
- Nako has an explicit decision for the first editable settings slice:
  - which fields are editable;
  - whether changes are runtime-only, persisted, restart-required, or rejected;
  - how current values are rendered without leaking sensitive config;
  - how validation, idempotency, auditability, and error states work.
- Admin API exposes only the accepted first mutation slice under
  `/admin/v1/*`, or the lane splits a required backend configuration-authority
  follow-on before UI mutation work proceeds.
- Admin Web V2 renders mutation controls only for fields backed by a real
  Admin API mutation path.
- Mock fallback cannot pretend a settings mutation succeeded.
- Tests, docs, browser smoke, and closeout evidence prove the shipped behavior.

Closeout result:

- First shipped slice: metadata raw cache retention and startup cleanup.
- Backend authority: `GET|PUT /admin/v1/settings/metadata/raw-cache`.
- Source of truth: configured startup value until an Admin override is
  persisted, then persisted Admin desired state.
- Runtime semantics: PUT validates and persists desired state; the response
  reports `effect = active|requires_restart` so Admin Web can tell the
  operator whether the running process already matches the desired state.
- Admin Web exposes save controls only when the raw-cache route is live-backed;
  mock fallback stays read-only and cannot report fake mutation success.
- Broader settings groups remain out of scope and require their own authority
  lanes.

## In Scope

- Route/API readiness audit for current settings diagnostics and mutation gaps.
- First editable settings slice selection.
- Admin API design for a narrow settings mutation or review-plan endpoint.
- Admin Web V2 route-owned mutation controls for the accepted slice, if and
  only if a real mutation path exists.
- Redaction tests for sensitive settings data.
- HTTP API and generated Admin Web contract updates when routes are added.
- Browser smoke for the edited settings workflow.

## Out Of Scope

- Raw TOML editor UI.
- Arbitrary full-config import/export.
- Secret value editing, secret display, or credential rotation.
- Exposing raw URLs, host names, filesystem paths, roots, env var names,
  provider secret references, tokens, or raw config.
- Public Client API, public OpenAPI, generated public SDK, or
  `nako-client-protocol` changes.
- User, Role, or Library Access management.
- Built-in NAT traversal or Network Tunnel Provider lifecycle automation.
- Addon Configuration Schema editing.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `/admin/v1/system/config` is diagnostic-only today. | High | `crates/nako-server/src/http/admin.rs` routes only `GET /admin/v1/system/config`; Admin Web client has only `getSystemConfig()`. | The first task must update readiness evidence and reuse any discovered route instead of designing a new one. |
| Full `NakoServerConfig` mutation is not accepted yet. | High | `config.rs` has `load_config()` and `example_config()`, no save/update authority. | The lane can implement a narrower persisted or runtime authority directly. |
| Network settings are the most user-visible first candidate but carry high redaction risk. | Medium | `network-access-boundary` added readiness diagnostics and left mutation as follow-on. | Pick a lower-risk slice such as worker budget if network editability requires a broader ADR. |
| Metadata Profile editability is already handled at the Media Library boundary. | High | `admin-library-metadata-profile-configuration` and AWL closeout cover library-scoped profile edits. | This lane must avoid duplicating library metadata-profile controls in global settings. |

## Architecture Direction

Use a two-step settings mutation model:

1. Establish a route/API readiness baseline before implementation.
2. Implement only a field group with an explicit source of truth and validation
   story.

Preferred first slice, pending ASM-020 evidence, is a non-secret policy group
with stable enum/boolean/numeric fields and no raw path or URL display. If
network exposure is selected, the API must preserve the existing diagnostics
redaction boundary and return review/validation facts rather than raw endpoint
values. If no field group can be safely persisted or applied at runtime, split a
backend configuration-authority workstream and keep Admin Web read-only.

Admin DTOs remain in `nako-api` under the Admin API boundary. Public client
contracts remain unchanged. Admin Web must call mutation routes only through
`AdminApiClient` and `AdminDataSource`; deterministic mock data may demonstrate
disabled/read-only states but must not report fake success.

## Closeout Condition

This lane can close when:

- route/API readiness is documented;
- either the first real settings mutation slice is implemented and verified, or
  a backend configuration-authority follow-on is split with evidence explaining
  why UI mutation is blocked;
- Admin Web does not present fake saves;
- redaction-sensitive tests pass;
- frontend, Rust, docs, generated contract, and browser smoke gates are
  recorded; and
- `WORKSTREAM.json`, `TODO.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md`
  reflect the final state.
