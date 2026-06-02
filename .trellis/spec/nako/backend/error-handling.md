# Error Handling

The current `nako` facade does not define fallible behavior. Error semantics
belong to the underlying public crates it re-exports.

## Required Patterns

- Let `nako-addon-protocol` own protocol validation errors.
- Let `nako-addon-client` own transport, retry, schema, and redaction errors.
- Keep facade tests focused on export visibility and feature-gate wiring.
- If a fallible helper is added, return the underlying crate error instead of
  inventing a facade-only error taxonomy.

## Forbidden Patterns

- Do not swallow or stringify underlying SDK errors.
- Do not introduce server error types into the facade.
- Do not expose error messages that reveal bearer tokens, shared secrets, local
  paths, source locators, or server internals.
- Do not add catch-all `anyhow` style APIs to the public facade.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| `addon_protocol` export changes | Test a representative protocol symbol |
| `addon-client` feature changes | Check and test with the feature enabled |
| Underlying SDK error changes | Keep error type in the owning crate |
| New fallible facade helper | Preserve underlying typed error |

## Evidence

- `crates/nako/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
