# Finish Notes

## Result

- Promoted `POST /admin/v1/jobs/{job_id}/cancel` into generated Admin Web route constants.
- Added `AdminApiClient.cancelJob` and `AdminDataSource.cancelJob`.
- Added queued/running Job cancel action to the Jobs page while preserving VFS cache repair execute/retry actions.
- Rendered backend-returned cancellation facts only: job id, returned status, terminal flag, and cancellation response state.

## Verification

- `cargo check -p nako-api --tests`
- `cargo check -p nako-server --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-08-admin-jobs-cancel-command-ui`

## Spec Update Decision

No `.trellis/spec/` update is needed for this slice. The existing specs already
cover the executable contract:

- generated Admin routes must be promoted through `nako-api` and regenerated;
- Admin Web mutations must use `AdminDataSource` and generated route constants;
- mock fallback must not fabricate mutation success;
- job diagnostics and command responses must stay redaction-safe.

## Redaction Check

The UI and test fixtures do not render raw durable `input_json`, `summary_json`,
raw errors, storage locators, local paths, backend URLs, credentials, etags,
fingerprints, URI digests, or cache payload material.
