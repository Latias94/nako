# Admin contract brace path parameters

## Goal

Normalize generated Admin API route templates to use brace-style path parameters
(`{addon_id}`) instead of mixing Axum-style `:addon_id` with generated-contract
placeholders. This removes Admin Web special-case route substitution and keeps
all generated Admin client commands on one parameter encoding rule.

## Requirements

- Replace `:addon_id` suffixes in `crates/nako-api/src/admin_contract.rs` with
  `{addon_id}` for generated Admin route constants.
- Regenerate both generated TypeScript Admin contract artifacts from
  `nako-api`.
- Update Admin Web client code so Addon routes use the same `routeWithParam`
  helper as other generated route templates.
- Remove the dedicated `addonPath` helper once no route uses colon parameters.
- Update client tests to assert brace-style Addon route templates and encoded
  path parameters.
- Add a contract-level regression check that generated Admin route templates do
  not contain colon-style path parameters.

## Acceptance Criteria

- [ ] `rg ':addon_id'` finds no generated contract, client, or client test
      residue.
- [ ] Admin Web route construction still URL-encodes unsafe Addon IDs.
- [ ] `cargo nextest run -p nako-api admin_contract --no-fail-fast` passes.
- [ ] `npm run test --prefix apps/admin-web -- adminApi/client.test.ts` passes.
- [ ] `npm run check --prefix apps/admin-web` passes.
- [ ] `cargo fmt --all -- --check` and `git diff --check` pass.

## Definition of Done

- Focused tests and checks pass.
- Trellis task context is validated.
- Any new contract convention is reflected in the relevant code-spec.
- Changes are committed with a Conventional Commit message.

## Technical Approach

`nako-api` is the source of truth for generated Admin route templates. The
implementation changes the generator inventory first, refreshes the generated
TypeScript contract artifacts, then simplifies the Admin Web client to route all
path substitution through `routeWithParam`.

## Decision (ADR-lite)

**Context**: The Admin route inventory currently normalizes both Axum-style
`:addon_id` and generated-contract `{addon_id}` templates for comparison, but
the generated TypeScript contract leaks the mixed style to Admin Web. That
forces a dedicated `addonPath` helper and makes future route coverage audits
harder.

**Decision**: Generated Admin API contract templates use brace-style
placeholders only. Server Axum route syntax can remain independent; normalization
still accepts Axum-style paths when comparing implemented routes.

**Consequences**: Admin Web route construction becomes uniform. Any future
generated route with path parameters should use `{param}` and `routeWithParam`
in the client.

## Out of Scope

- No server route behavior change.
- No new Admin Web UI controls.
- No public client or OpenAPI route inventory change.
- No redesign of route inventory generation beyond placeholder normalization.

## Research References

- [`research/jellyfin-route-template-comparison.md`](research/jellyfin-route-template-comparison.md)
  - Jellyfin controllers consistently expose route parameters as `{id}` in
    controller attributes, supporting brace-style templates as the generated
    contract convention.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/nako-api/backend/quality-guidelines.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Generated contract refresh commands:
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
