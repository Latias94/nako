# nako-official-addon-catalog Backend Guidelines

`nako-official-addon-catalog` is the shared source of truth for official addon
manifest and install descriptor facts. It prevents drift between Nako's addon
catalog, admin surfaces, and official sidecar runtimes.

## Current Evidence

- `crates/nako-official-addon-catalog/src/lib.rs`
- `crates/nako-official-addon-catalog/README.md`
- `docs/addons/OFFICIAL_ADDON_CATALOG.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `CONTEXT.md`

## Boundaries

- Build official addon manifests and install descriptors.
- Build the operator-visible official addon catalog artifact from the same
  manifest and install descriptor builders.
- Keep official addon IDs, versions, base URLs, container base URLs, runtime
  image names, binary names, resource paths, scopes, tasks, events, hosted pages,
  configuration schema facts, health-check path, trust tiers, smoke status, and
  install reference paths in one place.
- Validate facts through `nako-addon-protocol`.
- Keep actual sidecar implementation outside this crate.
- Keep operator lifecycle automation outside this crate.

## Executable Contract Summary

1. Scope / Trigger: any official addon fact, resource, scope, schema, runtime
   image, base URL, task, event, or install descriptor change updates this crate.
2. Signatures: module functions such as `default_manifest`,
   `container_manifest`, `manifest_with_version`,
   `binary_install_descriptor`, and `container_install_descriptor`; catalog
   functions such as `official_addon_catalog()` and
   `render_official_addon_catalog_markdown()`.
3. Contracts: official modules declare metadata scraper, resource search,
   external acquisition runner, subtitle provider, DLNA renderer, Chromecast
   renderer, and notification bridge facts. The catalog excludes helper
   surfaces such as `browser-worker`.
4. Validation & Error Matrix: every manifest and descriptor must pass
   `validate_manifest` or `validate_install_descriptor`; catalog artifact drift,
   missing compatible Nako version, missing health-check path, missing trust
   tier, missing smoke status, missing install reference, missing scopes, or
   invalid secret references are errors.
5. Good/Base/Bad Cases: good facts match constants and protocol schemas; base
   descriptors use binary or container runtime references; catalog rows
   distinguish Addon Version, Addon Protocol Version, compatible Nako version,
   and `POST /health`; bad cases include schema drift, hand-maintained artifact
   drift, helper surfaces listed as addons, missing scopes, or raw secret
   values.
6. Tests Required: one manifest shape test per official addon plus descriptor
   tests for binary/container install guides; catalog tests validate every entry
   and assert the markdown artifact exactly matches
   `render_official_addon_catalog_markdown()`.
7. Wrong vs Correct: do not duplicate official addon manifest facts in server or
   docs; import these builders and test against protocol validation. Do not
   hand-edit `docs/addons/OFFICIAL_ADDON_CATALOG.md` without updating the
   renderer and passing the artifact match test.

## Required Patterns

- Keep addon module constants stable and explicit.
- Use `ADDON_PROTOCOL_VERSION` from `nako-addon-protocol`.
- Use protocol schema constants for resource search, link check, external
  acquisition, and subtitle resources.
- Build diagnostics entry points and hosted pages through protocol declaration
  types.
- Use `AddonInstallDescriptor` for binary and container install paths.
- Use `OfficialAddonCatalogEntry` for catalog metadata that is not part of the
  Addon Protocol manifest itself, such as compatible Nako version, health-check
  path, trust tier, smoke status, and install/smoke document references.
- Keep `docs/addons/OFFICIAL_ADDON_CATALOG.md` generated from
  `render_official_addon_catalog_markdown()` and fixed to LF line endings so
  byte-level artifact tests are stable across Windows checkouts.

## Forbidden Patterns

- Do not copy official addon facts into server code.
- Do not include plaintext secret values in install descriptors.
- Do not treat official addon catalog as an Addon Manager.
- Do not change official resource/scope shape without protocol validation tests.
- Do not list browser/helper processes that do not expose an Addon manifest as
  catalog entries.
- Do not put lifecycle actions such as install, update, start, stop, remove,
  log streaming, or supervision into the catalog artifact or renderer.

## Validation

- Focused:
  `cargo nextest run -p nako-official-addon-catalog --no-fail-fast`
- Protocol contract:
  `cargo nextest run -p nako-addon-protocol -p nako-official-addon-catalog --no-fail-fast`
