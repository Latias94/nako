# Addon Token Grants Side Effects

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

The M5 Addon Protocol workstream lets Taru register and call HTTP Addon
Sidecars, but it intentionally stops before allowing addons to mutate Taru
state. ADR 0020 says addons may eventually perform strong side effects, but
only through Taru-owned APIs, Addon Tokens, accepted permissions, library
grants, audit, and resource boundaries.

This lane is the focused ARF-006 follow-up. It replaces the broad Post-M5 TODO
entry in `addons-automation` with a durable execution lane.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/addons-automation/`
- `docs/workstreams/access-boundary-auth/`
- `docs/workstreams/public-api-contract/`
- `docs/workstreams/public-client-source-locator-redaction/`
- `crates/taru-addon-protocol/`
- `crates/taru-core/src/addon.rs`
- `crates/taru-db/src/addons.rs`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http/addons.rs`

## Problem

Current addon registration stores manifests, enablement, base URLs, auth mode,
and granted scope strings. That is enough for Taru-to-addon calls, but not
enough for addon-to-Taru calls or protected writes.

The missing boundary has several risks:

- Addon Token issuance, revocation, and rotation are not modeled.
- Manifest-requested scopes and user/admin accepted grants are not clearly
  separated from runtime credentials.
- Addon grants are not narrowed by Media Library, so an addon could become
  accidentally global.
- Protected writes have no Taru-owned Addon Side Effect envelope, idempotency
  model, review/acceptance policy, audit trail, or safe error surface.
- Metadata, artwork, subtitle, and Library File Write behavior could grow
  inconsistent one API at a time.

## Target State

- Addon Tokens are issued by Taru, stored safely, revocable, and rotatable
  without changing addon registration identity.
- Runtime token checks bind the caller to one addon registration, accepted
  Addon Permissions, and optional Library-Scoped Addon Grants.
- Addon manifests can request permissions, but Taru stores accepted grants as
  the authority for execution.
- Addon Side Effects enter through a Taru-owned intake model that records actor,
  target, permission, library scope, idempotency key, provenance, validation
  result, and audit state.
- The first implementation slice can prove one safe side-effect path without
  opening direct database or filesystem mutation.
- Docs and tests make it hard to reintroduce admin-token, raw path, or
  unmediated storage shortcuts.

## In Scope

- Audit current addon registration, manifest auth, scope validation, and HTTP
  route behavior.
- Design Addon Token record shape, issuance response, redaction policy,
  revocation, rotation, and hash/secret storage boundary.
- Design accepted Addon Permissions and Library-Scoped Addon Grants.
- Design the Addon Side Effect intake envelope and audit/event expectations.
- Define the first protected-write proof slice and its gates.
- Update ADRs or split a new ADR if the Addon Protocol or access boundary
  contract changes materially.

## Out Of Scope

- OAuth for remote multi-tenant addon services.
- Addon Manager lifecycle automation.
- Native Plugin or Jellyfin Plugin Compatibility.
- Frontend Addon Hosted Page execution trust.
- Full metadata, artwork, subtitle, or NFO write feature breadth.
- Library Access/RBAC for human users beyond what token checks need to avoid
  erasing the future model.
- Public Client API changes unless needed to document a redaction boundary.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Long-lived revocable Addon Tokens are enough for the first self-hosted sidecar phase. | High | ADR 0020 explicitly defers OAuth-first authorization. | Need an OAuth or device-flow ADR before issuing credentials. |
| Addon permissions must be accepted grants, not just manifest declarations. | High | ADR 0015 denies all addon access by default. | Manifest validation could become implicit authorization. |
| Library-Scoped Addon Grants should be first-class before protected writes. | High | `CONTEXT.md` defines Library-Scoped Addon Grant and Addon Side Effect. | A metadata addon could accidentally write every library. |
| Side effects need a generic intake model before concrete write APIs multiply. | Medium | Metadata, artwork, subtitle, and Library File Write share audit and idempotency needs. | A too-generic model may delay useful vertical slices; split by effect type if needed. |
| The current registration table can be extended, but token secrets likely need a separate table. | Medium | Current DB stores addon registrations and granted scopes JSON. | Migration design may choose a different aggregate boundary after audit. |

## Architecture Direction

Keep the trust boundary asymmetric:

- Taru can call an Addon Sidecar through the existing bounded addon HTTP caller.
- An Addon Sidecar can call Taru only with an Addon Token issued for one
  registered addon.
- The token grants no admin identity. It resolves to an addon principal with
  accepted Addon Permissions and optional Media Library constraints.
- Protected writes become Addon Side Effects. Taru validates target identity,
  library scope, permission, idempotency, and storage/resource policy before
  committing any canonical metadata, Managed Artwork, subtitle, or Library File
  Write.

Prefer one narrow proof slice after design: a token-authenticated side-effect
intake that records or rejects a proposed metadata/artifact change without
performing broad canonical mutation. That proves the identity, grant, audit,
and idempotency seam before adding richer write handlers.

## Closeout Condition

This lane can close when:

- Addon Token lifecycle and secret storage policy are implemented or explicitly
  deferred with an ADR-backed reason;
- accepted Addon Permissions and Library-Scoped Addon Grants are enforced by a
  runtime access check;
- at least one Addon Side Effect intake path is implemented with tests for
  allowed, denied, revoked, wrong-library, duplicate-idempotency, and redacted
  response cases;
- docs describe how addons perform protected writes without receiving admin
  tokens or raw storage authority;
- targeted Rust gates and `git diff --check` pass;
- and broader metadata/artwork/subtitle write feature breadth is completed or
  split into follow-on lanes.

