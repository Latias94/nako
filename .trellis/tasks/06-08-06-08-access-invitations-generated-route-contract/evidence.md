# Evidence

## 2026-06-08 Access Invitation Generated Route Contract

Decision:

- Promote the existing Access Invitation list/create/revoke routes into the
  generated Admin API route inventory.
- Keep Admin Web on a bounded invitation-first operator projection instead of
  broad user/account management.
- Keep the one-time raw invitation token only in the create mutation result and
  never in list/read rows or mock summaries.

Reference boundary:

- `repo-ref/jellyfin` was used only to compare operator workflow shape around
  user access administration.
- No Jellyfin code, comments, schemas, assets, migrations, tests, or generated
  output were copied or translated.
- Nako keeps its own invitation redemption model rather than imitating
  Jellyfin direct user/password/policy mutation flows.

Implemented:

- Added generated Admin route keys for:
  - `accessInvitations`
  - `accessInvitationRevoke`
- Removed Access Invitation list/create/revoke routes from generated-contract
  exclusions.
- Refreshed generated Admin TypeScript contracts for both Admin Web copies.
- Added Admin Web client/data-source methods for invitation list, create, and
  revoke using generated routes and encoded `invitation_id`.
- Added safe `AccessInvitationRow` projection and bounded `/access`
  invitation panel with create form, one-time token display, revoke
  prepare/confirm, live-source mutation gating, i18n, mock data, and route
  tests.
- Recorded the reusable projection rule in
  `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.

Validation:

- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`
  - Result: passed, 173 tests.
- `npm run test --prefix apps/admin-web`
  - Result: passed, 201 tests.
- `npm run build --prefix apps/admin-web`
  - Result: passed; Vite reported a non-blocking chunk-size warning.
- `cargo check -p nako-api --tests`
  - Result: passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
  - Result: passed, 8/8 tests.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  - Result: passed, 1/1 test.
- `cargo fmt --all -- --check`
  - Result: passed.
- `git diff --check`
  - Result: passed with line-ending warnings only.
- `python ./.trellis/scripts/task.py validate 06-08-06-08-access-invitations-generated-route-contract`
  - Result: passed.

Final recheck before commit:

- `npm run check --prefix apps/admin-web`
  - Result: passed.
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts adminApi/dataSource.test.ts App.test.tsx`
  - Result: passed, 173 tests.
- `python ./.trellis/scripts/task.py validate 06-08-06-08-access-invitations-generated-route-contract`
  - Result: passed.
- `git diff --check`
  - Result: passed with line-ending warnings only.
