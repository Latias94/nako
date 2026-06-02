# Error Handling

Official catalog builders mostly return concrete values. Validation failures
should be caught in tests through `nako-addon-protocol` helpers.

## Required Patterns

- Validate each official manifest in tests with `validate_manifest`.
- Validate install descriptors in tests with `validate_install_descriptor`.
- Keep invalid official facts out of runtime by failing focused tests.
- Use protocol errors rather than custom catalog error types unless runtime
  lookup behavior is added.

## Forbidden Patterns

- Do not leave official catalog drift to be discovered by server integration
  tests only.
- Do not unwrap optional schema facts in production callers without knowing the
  addon module guarantees them.
- Do not hide invalid secret references in install notes.
- Do not silently accept local runtime paths rejected by protocol validation.

## Examples

- `resource_search` must declare both `ResourceSearch` and `ResourceLinkCheck`
  resources with matching schema constants.
- `external_acquisition_runner` must declare the action task and optional
  Transmission password secret reference.
- Renderer addons must declare renderer-adapter scopes and resources.

## Review Checklist

- Does the module test assert every fact changed?
- Does protocol validation still pass?
- Are install notes free of secrets?
