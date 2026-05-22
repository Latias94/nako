# 2026-05-22 AWACG-040 Closeout

Closed `admin-web-addon-credential-grant-onboarding`.

Outcome:

- Generated Admin API TypeScript contract includes explicit one-time token
  issue/rotation response DTOs, revoke response DTO, and grant replacement
  request DTOs.
- Admin Web client/data-source supports issue, rotate, revoke, and replace
  grants through Admin-only routes.
- Admin Web renders Addon Credentials & Grants with:
  - token label input;
  - issue token;
  - rotate first token;
  - revoke first token;
  - accepted grant replacement;
  - one-time raw token copy-now notice;
  - enable readiness checklist.
- Raw tokens remain action-only and are not part of load data.
- Docs explain one-time raw token handling and grant/manifest scope separation.

Closeout evidence:

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`
- `git diff --check`

Note: `assets/brand/nako-app-icon-1024.png` is an untracked file in the working
tree and was intentionally left unstaged because it is not part of this
workstream.
