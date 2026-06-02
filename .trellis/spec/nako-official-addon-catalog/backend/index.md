# nako-official-addon-catalog Backend Guidelines

`nako-official-addon-catalog` is the shared source of truth for official addon
manifest and install descriptor facts. It prevents drift between Nako's addon
catalog, admin surfaces, and official sidecar runtimes.

## Current Evidence

- `crates/nako-official-addon-catalog/src/lib.rs`
- `crates/nako-official-addon-catalog/README.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `CONTEXT.md`

## Boundaries

- Build official addon manifests and install descriptors.
- Keep official addon IDs, versions, base URLs, container base URLs, runtime
  image names, binary names, resource paths, scopes, tasks, events, hosted pages,
  and configuration schema facts in one place.
- Validate facts through `nako-addon-protocol`.
- Keep actual sidecar implementation outside this crate.
- Keep operator lifecycle automation outside this crate.

## Executable Contract Summary

1. Scope / Trigger: any official addon fact, resource, scope, schema, runtime
   image, base URL, task, event, or install descriptor change updates this crate.
2. Signatures: module functions such as `default_manifest`,
   `container_manifest`, `manifest_with_version`,
   `binary_install_descriptor`, and `container_install_descriptor`.
3. Contracts: official modules declare metadata scraper, resource search,
   external acquisition runner, subtitle provider, DLNA renderer, Chromecast
   renderer, and notification bridge facts.
4. Validation & Error Matrix: every manifest and descriptor must pass
   `validate_manifest` or `validate_install_descriptor`; missing scopes or
   invalid secret references are protocol errors.
5. Good/Base/Bad Cases: good facts match constants and protocol schemas; base
   descriptors use binary or container runtime references; bad cases include
   schema drift, missing scopes, or raw secret values.
6. Tests Required: one manifest shape test per official addon plus descriptor
   tests for binary/container install guides.
7. Wrong vs Correct: do not duplicate official addon manifest facts in server or
   docs; import these builders and test against protocol validation.

## Required Patterns

- Keep addon module constants stable and explicit.
- Use `ADDON_PROTOCOL_VERSION` from `nako-addon-protocol`.
- Use protocol schema constants for resource search, link check, external
  acquisition, and subtitle resources.
- Build diagnostics entry points and hosted pages through protocol declaration
  types.
- Use `AddonInstallDescriptor` for binary and container install paths.

## Forbidden Patterns

- Do not copy official addon facts into server code.
- Do not include plaintext secret values in install descriptors.
- Do not treat official addon catalog as an Addon Manager.
- Do not change official resource/scope shape without protocol validation tests.

## Validation

- Focused:
  `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`
- Protocol contract:
  `cargo nextest run -p nako-addon-protocol -p nako-official-addon-catalog --no-fail-fast`
