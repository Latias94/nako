# Legacy principal normalization cleanup

## Goal

Remove the legacy HTTP `UserPrincipalId` request-extension compatibility path
after migrating the remaining server consumer to the canonical
`AuthenticatedPrincipal` request extension.

## Plan Anchor

This is the first code-bearing follow-up from
`.trellis/tasks/archive/2026-06/06-16-06-16-backend-readiness-control-plane-audit/audit.md`.
It advances U1 from
`docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
by deleting obsolete compatibility once replacement ownership is proven.

## Requirements

* `crates/nako-server/src/http/auth.rs` must insert only
  `AuthenticatedPrincipal` for authenticated, session-authenticated, and
  auth-disabled local-admin requests.
* `crates/nako-server/src/http/admin.rs` must not require
  `Extension<UserPrincipalId>` for admin access summary. It should derive the
  principal ID from `AuthenticatedPrincipal`.
* Keep Admin and Public API DTO shapes unchanged.
* Do not change auth token semantics, session-token semantics, playback-ticket
  bypass, admin guard behavior, or user/library access policy.
* Do not add schema migrations, generated contract changes, frontend changes,
  or new public routes.
* Tests must prove the canonical principal extension covers the previous
  legacy behavior.

## Acceptance Criteria

* [x] Production server code no longer inserts `UserPrincipalId` into request
      extensions from `require_auth`.
* [x] Production handlers no longer extract `Extension<UserPrincipalId>`.
* [x] Auth middleware tests assert `AuthenticatedPrincipal` is present for
      bearer-token and auth-disabled flows without relying on `UserPrincipalId`.
* [x] Admin access summary still works for the canonical principal extension.
* [x] `rg "Extension\\(principal\\): Extension<UserPrincipalId>" crates/nako-server/src`
      returns no production handler.
* [x] `rg "extensions\\(\\).*UserPrincipalId|get::<UserPrincipalId>" crates/nako-server/src/http`
      returns no remaining legacy compatibility assertion.
* [x] Focused `nako-server` tests pass for auth/admin access behavior.

## Definition of Done

* [x] Legacy principal request-extension insertion is removed.
* [x] Tests document the canonical principal path.
* [x] Trellis context validates.
* [x] `cargo fmt --all -- --check` or scoped format check passes.
* [x] Focused `cargo nextest run -p nako-server ... --no-fail-fast` passes.
* [x] `git diff --check` passes.
* [ ] Commit message is conventional.

## Out of Scope

* Changing identity domain models.
* Changing Admin/Public API response schemas.
* Changing session storage or token issuance.
* Changing library access or playback policy.
* Removing other compatibility paths such as legacy watch-folder source keys or
  legacy Addon task resource classes.

## Technical Notes

Primary files:

* `crates/nako-server/src/http/auth.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/tests/system.rs`

Likely focused verification:

* `cargo nextest run -p nako-server require_auth --no-fail-fast`
* `cargo nextest run -p nako-server admin_access --no-fail-fast`

Verification note: `admin_access` matched zero nextest cases in this workspace,
so the equivalent focused filters used were `admin_v1_access` and
`local_session_auth`.
