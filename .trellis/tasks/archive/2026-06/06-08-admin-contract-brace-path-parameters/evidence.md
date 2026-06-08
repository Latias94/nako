# Evidence

## Changes

- Unified generated Admin Addon route templates from `:addon_id` to
  `{addon_id}` in `nako-api`.
- Regenerated both Admin TypeScript contract artifacts.
- Removed the Admin Web `addonPath` colon-template helper and routed Addon
  client path substitution through the shared brace-parameter helper.
- Added API contract guidance that generated client-facing route templates must
  reject `/:param` placeholders.
- Recorded the matching Admin Web client convention in the frontend spec.

## Verification

- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-admin-contract-brace-path-parameters`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `npm run test --prefix apps/admin-web -- adminApi/client.test.ts`
- `npm run check --prefix apps/admin-web`

## Notes

- `rg -n ':addon_id|addonPath\('` has no runtime/generated-client hits after
  the change. The remaining `:addon_id` text is a Trellis spec bad-case example
  documenting the forbidden generated suffix style.
- `rg -n '/:' apps/admin-web/src/adminApi/generated/contract.ts web/src/api/admin/generated/contract.ts`
  returns no generated contract matches.
