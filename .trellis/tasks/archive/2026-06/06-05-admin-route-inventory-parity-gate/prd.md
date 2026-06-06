# Admin route inventory parity gate

## Goal

Add a bounded Admin route inventory parity gate so parallel lanes can safely
change Admin API routes, generated Admin TypeScript contracts, and Admin Web
clients without silent route drift.

## Requirements

- Compare implemented `/admin/v1/*` routes with generated Admin route
  constants.
- Normalize route parameter syntax such as `{job_id}` and `:job_id`.
- Classify implemented routes as generated, intentionally excluded, or missing.
- Keep this as a contract/test gate; do not change endpoint behavior.
- Preserve Admin/Public API separation and generated artifact workflow.
- Keep generated contracts synchronized only through the existing generator.

## Acceptance Criteria

- [x] Generated Admin route constants map to implemented server routes.
- [x] Implemented Admin routes that are not generated are explicitly excluded
      with a reason.
- [x] Placeholder syntax differences do not produce false drift failures.
- [x] Existing generated Admin TypeScript contract checks still pass.
- [x] The gate covers routes from both Admin HTTP and Addon Admin route modules.
- [x] No Public Client route is added to the Admin route inventory.

## Definition of Done

- Focused `nako-api` route/contract tests pass.
- Focused server route inventory or HTTP tests pass if touched.
- `cargo fmt --all -- --check` and `git diff --check` pass.

## Out of Scope

- No new Admin endpoint behavior.
- No Admin Web page implementation.
- No generated contract hand editing.
- No broad `admin_contract.rs` generator refactor in this task.

## Technical Notes

- Parent audit: `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/`
- Key research:
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/admin-api-web-contracts.md`
  - `.trellis/tasks/06-05-06-05-cross-lane-architecture-audit/research/synthesis.md`
- Likely files:
  - `crates/nako-api/src/admin_contract.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/addons.rs`
  - generated Admin TypeScript contracts if the generator output changes
