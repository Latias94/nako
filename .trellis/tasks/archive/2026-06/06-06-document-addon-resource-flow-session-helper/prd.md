# Document Addon Resource Flow Session Helper

## Goal

Record the server-local Addon resource-flow `SelectionSessionStore` convention
in the Trellis spec after extracting the shared helper. Future Addon resource
flows should reuse the helper instead of copying TTL, max-count, and
selection-lookup mechanics.

## Requirements

- Update `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`.
- Document the `resource_flow::SelectionSessionStore<TSelection, TContext>`
  helper as the default server-local selection session mechanism.
- Preserve the rule that resource-specific behavior stays in the resource
  module.
- Document that missing selection/addon mismatch returns resource-specific
  not-found, while manifest mismatch remains a conflict.
- Record required tests for helper behavior and resource-flow regressions.

## Acceptance Criteria

- [ ] The spec names the server-local helper and its scope.
- [ ] The spec lists concrete helper signatures and responsibilities.
- [ ] The spec clearly forbids new bespoke session-store copies for Addon
      resource flows.
- [ ] The spec remains actionable with validation/error behavior and test
      expectations.
- [ ] Trellis context validation and `git diff --check` pass.

## Definition Of Done

- Trellis task context validates.
- `git diff --check` passes.
- Task is archived with implementation/check context.
- Commit the spec-only change with a Conventional Commit message.

## Out Of Scope

- No Rust code changes.
- No public/Admin API, DTO, route, or generated contract changes.
- No Addon Protocol changes.
- No additional resource-flow extraction.

## Technical Notes

- Implementation evidence:
  commit `22106c8e refactor(addons): share resource flow sessions`.
- Relevant spec:
  `.trellis/spec/nako-server/backend/addon-resource-flow-patterns.md`.
- Helper implementation:
  `crates/nako-server/src/app/addons/resource_flow.rs`.
