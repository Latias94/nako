# Admin Playback Policy Management API

## Problem

Nako already has a backend `PlaybackPolicy` domain model, SQLite/PostgreSQL
persistence, effective-policy resolution, and playback/runtime enforcement.
The product gap is that a self-hosted operator has no Admin API surface to
create, list, or remove those policies. That makes the feature effectively
test-only even though playback paths already respect it.

## Goals

- Add an Admin-only playback policy management route under the existing access
  control surface.
- Let operators list, upsert, and delete per-user or per-role playback
  policies scoped to one Media Library.
- Keep the wire contract redaction-safe: expose only scope IDs/roles, library
  ID, permission booleans, bitrate caps, and timestamps.
- Reuse the existing `PlaybackPolicyRepository`; do not add migrations or a
  second policy model.
- Refresh generated Admin TypeScript contracts from `nako-api`.

## Non-Goals

- No Admin Web UI in this slice.
- No new Public Client DTO or OpenAPI change.
- No schema migration; persistence already exists.
- No change to playback planner behavior or permission semantics.
- No active session count, idle timeout, or household profile system yet.

## API Contract

- `GET /admin/v1/access/playback-policies`
  - Query filters: optional `user_id`, optional `role`, optional
    `library_id`, plus existing bounded `limit` and `offset`.
  - Response: `AdminPlaybackPolicyListResponse`.

- `PUT /admin/v1/access/playback-policies`
  - Body: `AdminUpsertPlaybackPolicyRequest`.
  - Idempotently creates or replaces the policy for `(scope, library_id)`.
  - Response: `AdminPlaybackPolicyResponse`.

- `DELETE /admin/v1/access/playback-policies`
  - Query identifies exactly one scope and one library:
    `user_id + library_id` or `role + library_id`.
  - Response: `AdminPlaybackPolicyDeleteResponse { deleted: true }`.

## Acceptance Criteria

- Admin DTOs exist for playback policy scope, permission policy, record, list
  response, upsert request, response, and delete response.
- Server handlers reuse `PlaybackPolicyRepository` through `NakoApp`, not raw
  SQL or playback runtime internals.
- Query parsing rejects missing or ambiguous scope filters using existing
  `NakoError::InvalidInput` behavior.
- Route inherits the existing Admin auth guard.
- Route inventory and generated Admin contracts include
  `accessPlaybackPolicies`.
- Focused server tests prove list/upsert/delete behavior and policy redaction.
- Existing playback policy enforcement tests remain valid.

## Validation

- `cargo check -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server admin_playback_policy --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `npm run generate:admin-api --prefix apps/admin-web`
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
- `git diff --check`
