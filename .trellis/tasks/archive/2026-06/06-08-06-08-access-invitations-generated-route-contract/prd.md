# Access Invitation Generated Route Contract

## Goal

Make the existing Admin Access Invitation lifecycle reachable through the
generated Admin API route contract and a bounded Admin Web operator panel. This
closes a hidden-route gap: the server already supports listing, creating, and
revoking invitations, but Admin Web cannot call those routes through
`NAKO_ADMIN_ROUTES`.

## What I Already Know

- The parent overnight campaign is comparing Nako with `repo-ref/jellyfin` and
  shipping independently verified fearless-refactor slices.
- Nako already has server handlers and tests for:
  - `GET /admin/v1/access/invitations`
  - `POST /admin/v1/access/invitations`
  - `POST /admin/v1/access/invitations/{invitation_id}/revoke`
- `crates/nako-api/src/admin_contract.rs` already contains invitation DTOs, but
  the two route suffixes are still explicitly excluded from generated route
  constants.
- Admin Web `/access` currently renders summary/readiness only and deliberately
  hides broader user/account mutation UX.
- Invitation creation returns a one-time raw invitation token. Invitation list
  responses must not expose token hashes or raw tokens.

## Reference-Code Boundary

- Jellyfin is used only for architecture and operator workflow comparison.
- Do not copy, translate, or import Jellyfin code, comments, tests, schemas, or
  assets.
- Nako should keep its Nako-native invitation model rather than imitating
  Jellyfin user creation or password reset flows directly.

## Requirements

- Add generated Admin route keys for invitation list/create and revoke.
- Remove the invitation routes from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Regenerate both Admin TypeScript contract copies:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add typed Admin Web client methods for list/create/revoke using generated
  routes and encoded `invitation_id`.
- Add route-local Admin Web data-source methods that:
  - safely map invitation records into UI rows,
  - preserve mock fallback for reads,
  - reject mutation failures instead of fabricating success,
  - keep raw creation token only in the create result.
- Extend `/access` with a bounded invitation panel:
  - list recent invitation facts,
  - create a Viewer/Admin invitation through a controlled form,
  - require explicit confirmation before revoke,
  - disable mutations when the invitation read source is not live.
- Do not expose raw tokens, token hashes, local paths, backend URLs,
  credentials, source URIs, fingerprints, or arbitrary raw payloads.

## Acceptance Criteria

- [ ] `nako-api` generated route inventory includes invitation route constants.
- [ ] Generated contract drift tests pass.
- [ ] Admin Web client and data source tests cover list/create/revoke routes,
      encoded parameters, fallback reads, and mutation failures.
- [ ] Access route tests cover live rendering, zh-Hans copy, create/revoke
      confirmation, mock mutation disabling, and redaction.
- [ ] Focused Rust and Admin Web gates pass before commit.

## Definition Of Done

- Code and generated artifacts are updated.
- Task evidence records commands run and results.
- Relevant spec memory is updated if this establishes a reusable pattern.
- Commit only this slice with a Conventional Commit message.

## Out Of Scope

- Full user management UI.
- Password reset, local password editing, lockout UX, or session management.
- Library Access policy editing.
- Persisting or re-displaying raw invitation tokens after the page state is
  cleared.
- Any schema migration.

## Technical Notes

- Backend evidence:
  `crates/nako-server/src/http/tests/system.rs::invitation_registration_redeems_once_and_does_not_list_raw_tokens`.
- API DTO evidence:
  `crates/nako-api/src/admin/access.rs` defines `AdminInvitationRecord`,
  `AdminInvitationListResponse`, `AdminCreateInvitationRequest`, and
  `AdminCreateInvitationResponse`.
- Existing Admin Web pattern:
  Addon token creation stores one-time raw token only in mutation result, while
  list/read models render safe summaries.
