# Quality Guidelines

API contract work must keep generated artifacts and route inventories honest.

## Required Patterns

- Update DTO source first, generator second, generated artifacts last.
- Regenerate generated Admin Web contract files from `nako-api`; do not edit
  generated TypeScript directly.
- Keep Admin `/admin/v1/*` routes out of Public Client/OpenAPI/SDK outputs
  unless the task explicitly changes the public contract.
- Add tests for route inventory, DTO generation, redaction, and Public/Admin
  separation.
- Use snake_case serde wire fields to match existing contracts.

## Gate Selection

- Focused admin contract:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- Public/OpenAPI/SDK contract:
  `cargo nextest run -p nako-api --no-fail-fast`
- Admin Web contract refresh:
  `npm run generate:admin-api --prefix apps/admin-web`
- Cross-crate API/server:
  `cargo check -p nako-api -p nako-server --tests`

## Forbidden Patterns

- Do not expose internal database/domain records directly as wire responses.
- Do not add route strings in Admin Web instead of the generated contract.
- Do not make Public Client contracts depend on Admin-only concepts.
- Do not add a DTO field before deciding redaction and audience.

## Review Checklist

- Does this route belong to Admin API or Public Client API?
- Is the generated output updated by the generator?
- Are route inventory tests updated?
- Are sensitive fields redacted or omitted?
