# Addon Architecture Deepening

Status: Completed
Last updated: 2026-05-21

## Why This Lane Exists

The 2026-05-21 Addon architecture review found that Taru has made the correct
macro decision: an **Addon** is an HTTP **Addon Sidecar** that follows the
**Addon Protocol**, not a **Native Plugin** and not **Jellyfin Plugin
Compatibility**. The implementation already has Addon registration, Addon
Tokens, Library-Scoped Addon Grants, Addon Side Effects, Canonical Metadata
Protected Writes, Addon Artwork Candidate proposals, and a narrow NFO **Library
File Write** path.

The remaining risk is not direction; it is depth. The current Addon Modules are
good enough for the first slices, but several Interfaces are shallow enough that
future Addon breadth would spread permissions, target validation, idempotency,
payload schemas, redaction, storage policy, and apply outcome rules across
callers.

This lane performs a fearless refactor before that breadth hardens.

## Relevant Authority

- Domain language:
  - `CONTEXT.md`
- Repository guidance:
  - `AGENTS.md`
- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0029-postgresql-ready-persistence-boundary.md`
  - `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`
- Related workstreams:
  - `docs/workstreams/addons-automation/`
  - `docs/workstreams/addon-token-grants-side-effects/`
  - `docs/workstreams/addon-protected-writes/`
  - `docs/workstreams/addon-library-file-write-policy/`
  - `docs/workstreams/addon-managed-artwork-artifacts/`
  - `docs/workstreams/managed-artwork-ingest-selection/`
  - `docs/workstreams/managed-artwork-fetch-artifact-storage/`
  - `docs/workstreams/managed-artwork-public-serving-selection/`
  - `docs/workstreams/fearless-architecture-deepening/`

## Problem

Current Addon behavior is implemented and tested, but not yet deep enough for
the full Addon product surface:

1. **Addon Side Effect lifecycle is distributed.** Intake creates validation
   state and safe error codes; the apply router owns permission dispatch and
   some failure taxonomy; metadata writes commit their own apply outcome through
   a repository seam; artwork and Library File Write Adapters return summaries
   that the router turns into outcomes. Deleting the router would not remove
   complexity; it would scatter lifecycle rules back into callers.
2. **Idempotency replay is key-only.** `addon_id + idempotency_key` returns an
   existing Addon Side Effect even if permission, target, provenance, or payload
   differ. True replay and conflicting key reuse are not distinct.
3. **Protected Write payload contracts are private implementation details.**
   `metadata_write`, `artwork_write`, and `library_file_write` parse private
   server structs from JSON. Addon authors and future SDKs must infer the
   Interface from tests and implementation rather than a protocol-owned Module.
4. **Addon Manifest is still a first-slice Module.** It has resources, scopes,
   auth, timeout, and retry, but not Addon Entry Points, Addon Hosted Pages,
   Addon Configuration Schema, Secret References, Addon Event Subscriptions, or
   Addon Task declarations.
5. **Library File Write is narrower than its name.** The domain Interface should
   own target derivation, storage writability, atomic replace, backup,
   preservation, idempotency, and redacted reporting for multiple file roles.
   The current Addon Adapter implements only MediaSource-targeted NFO Export.
6. **Admin Addon Interface needs one Admin API boundary.** Token and grant
   administration already uses `/admin/v1/addons/*`; registration and listing
   should live there too, with no root `/addons` management compatibility seam,
   and Admin responses must not expose core persistence records directly.
7. **Documentation state is stale.** Several implemented ADRs still say
   `Proposed`, which makes future reviewers unsure whether they are constraints
   or exploratory ideas.

## Target State

When this lane closes:

- Addon Side Effect submission, authorization, target validation, journaling,
  apply dispatch, apply outcome persistence, failure taxonomy, and replay
  behavior live behind one deep runtime Interface.
- Addon Side Effect idempotency records a stable request fingerprint and
  returns conflict for same-key different-request reuse.
- Protected Write payloads have explicit protocol or admin-owned DTO Modules
  for the shipped behaviors:
  - Canonical Metadata patch;
  - Addon Artwork Candidate proposal;
  - Library File Write command.
- Addon Manifest validation is deepened through a validated manifest Module
  and first-class declaration types for the next Addon Protocol concepts, even
  where runtime execution is deferred.
- Library File Write behavior has a Taru-owned runtime seam. NFO Export is one
  Adapter behind that seam, not the seam itself.
- Addon administration has a clear Admin API v1 Interface with DTO shielding.
  Root `/addons` management is removed instead of wrapped for compatibility.
- ADRs and workstream docs distinguish accepted Addon architecture constraints
  from deferred Addon Manager, Addon Task, and Event Subscription breadth.
- SQLite and PostgreSQL stay semantically aligned for any schema or repository
  changes made in this lane.

## In Scope

- Refactoring Addon Side Effect runtime Modules in `taru-server`.
- Adding request fingerprinting to Addon Side Effect idempotency.
- Adding or moving Addon-facing payload DTOs to the correct Interface seam.
- Deepening `taru-addon-protocol` while preserving its permissive crate
  boundary and avoiding server-internal dependencies.
- Adding Addon Manifest declaration types and validation for concepts already
  named in `CONTEXT.md` and ADR 0020.
- Introducing a Library File Write runtime seam for Taru-owned file writes.
- Admin Addon API route and DTO shielding work.
- SQLite/PostgreSQL migration and repository parity for touched Addon state.
- Updating ADR/workstream/API docs when an Interface changes.
- Deleting obsolete pass-through helpers when replacements are proven.

## Out Of Scope

- Addon Manager discovery, installation, update, marketplace, package signing,
  process supervision, log collection, or rollback.
- OAuth or remote multi-tenant Addon authorization.
- Native Plugin ABI or Jellyfin Plugin Compatibility.
- Embedded JavaScript runtime or WASI runtime.
- Full Addon Task scheduler implementation unless a minimal protocol
  declaration is needed for manifest depth.
- Full Addon Event Subscription delivery bridge unless a minimal declaration is
  needed for manifest depth.
- Broad subtitle model implementation.
- Broad Managed Artwork workflow decomposition beyond what Addon runtime
  changes require. Split a follow-on if that becomes its own problem.
- Public Client API write routes.
- Provider breadth, AI/vector search, network tunnel, or playback feature
  expansion.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| HTTP Addon Sidecars remain the correct extension model. | High | ADR 0003 and ADR 0020; existing `taru-addon-protocol`. | Reopen extension-model ADRs before code changes. |
| Addon Tokens must stay separate from administrator bearer tokens. | High | ADR 0020 and ADR 0024; runtime `/addon/v1/*` routes. | Redesign inbound identity before runtime refactors. |
| Addon Side Effect is the right shared journal for strong Addon side effects. | High | ATGSE/APW/ALFW/AMAA lanes and current tests. | Split per-effect journals only with a stronger deletion-test argument. |
| Request fingerprinting is required for safe idempotency. | High | Current replay uses key-only lookup. | If existing clients depend on key-only replay, add compatibility handling and document migration. |
| Protected Write payload contracts belong at a reusable seam, not private server structs. | High | `taru-addon-protocol` is intended for Addon authors; current payloads are private. | If license/dependency constraints block this, create a smaller permissive protocol submodule. |
| Library File Write should become a runtime seam before subtitle/artwork sidecar breadth. | High | `CONTEXT.md` and ALFW both reject direct Addon path writes. | Keep future file roles blocked until the seam exists. |
| Admin Addon registration is admin-only behavior. | Medium | ADR 0027 says new admin-only surfaces use `/admin/v1/*`; current `/addons` predates it. | Remove root `/addons` rather than documenting it as public. |

## Architecture Direction

Use Module-depth rules:

- A **Module** earns its existence when its Interface gives callers leverage and
  keeps implementation knowledge local.
- Do not add pass-through Modules. Apply the deletion test before introducing
  a new seam.
- Add a seam when behavior varies, when ordering is unsafe for callers to
  remember, or when tests need to hit the same Interface that production
  callers use.
- Keep `taru-addon-protocol` dependency-light and free of server internals. If
  HTTP client helpers make the protocol crate too heavy, split a follow-on
  `taru-addon-client` crate rather than weakening the protocol seam.
- Prefer vertical slices that keep behavior testable after each task.
- Update ADRs before declaring an Interface complete when the refactor changes
  a public, admin, protocol, storage, or security contract.

## Closeout Condition

This lane can close when:

- AAD-010 through AAD-100 are complete or split into named follow-ons with
  evidence-backed rationale;
- Addon Side Effect runtime has one deep lifecycle Interface and focused tests;
- idempotency distinguishes replay from conflict;
- shipped Protected Write payload contracts are explicit and documented;
- Addon Manifest supports the next declaration concepts or records why a
  concept is deferred;
- Library File Write has a deeper runtime seam than the current NFO-only
  Adapter;
- Admin Addon API has a clear `/admin/v1` path and DTO shielding plan;
- SQLite and PostgreSQL Addon state remain aligned for touched behavior;
- final evidence includes `cargo fmt --all -- --check`, workspace checks,
  focused nextest gates, `git diff --check`, and PostgreSQL opt-in evidence
  when `TARU_TEST_POSTGRES_URL` is available.
