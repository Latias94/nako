# Finish Notes

## Result

- Promoted Addon token and grant Admin routes into generated Admin Web route constants:
  - `addonTokens`
  - `addonTokenRotate`
  - `addonTokenRevoke`
  - `addonGrants`
- Removed the four matching route exclusions from `admin_contract_route_exclusions()`.
- Regenerated both Admin TypeScript contract artifacts from `nako-api`.
- Replaced Admin Web credential and grant path construction with generated route keys.
- Updated Admin API client/data-source tests so credential paths are asserted through generated route constants.

## Verification

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-08-06-08-addon-credential-generated-route-contract`

## Spec Update Decision

No `.trellis/spec/` update is needed for this slice. Existing specs already
cover the rule being enforced:

- Admin Web-facing routes should live in the generated Admin contract;
- generated TypeScript contract files must come from `nako-api`;
- Admin Web calls must use `AdminApiClient`, `AdminDataSource`, and
  `NAKO_ADMIN_ROUTES`;
- sensitive credential material must remain redaction-safe.

## Redaction Check

This slice only promotes route templates and replaces path construction. It does
not change token issue, rotation, revocation, hashing, grant replacement, or
response semantics. Admin Web load data still does not render raw tokens, token
hashes, backend URLs, local paths, or Addon Sidecar secrets.
