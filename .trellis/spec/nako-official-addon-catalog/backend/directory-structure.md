# Directory Structure

`nako-official-addon-catalog` currently keeps official addon facts in
`src/lib.rs`, grouped by addon module.

## Current Layout

- `metadata_scraper`: metadata suggestion, bulk metadata scrape task,
  library-scanned event subscription, diagnostics hosted page, provider toggles.
- `resource_search`: external resource search and link-check resources,
  diagnostics hosted page, provider/source configuration.
- `external_acquisition_runner`: action task, runner profile configuration, and
  optional Transmission password secret reference.
- `subtitle_provider`: subtitle search resource and read-only configuration.
- `dlna_renderer`: renderer-adapter manifest with plan-only configuration.
- `chromecast_renderer`: renderer-adapter manifest for Chromecast targets.
- `notification_bridge`: event subscription and acknowledgement resource.

## Module Rules

- Keep one official addon module per addon ID.
- Keep constants, manifest builders, install descriptor builders, and tests in
  the same module area.
- Use private `configuration_schema` helpers for JSON schema assembly.
- Add a new official addon module only when it has a real manifest contract.

## Naming Rules

- Use `ADDON_ID`, `ADDON_NAME`, `ADDON_VERSION`, `DEFAULT_BASE_URL`, and
  `DEFAULT_CONTAINER_BASE_URL` in each module.
- Use `RUNTIME_BINARY` and `RUNTIME_IMAGE` for install descriptors.
- Use `*_RESOURCE_PATH`, `*_TASK_ID`, `*_EVENT_KIND`, and `CONFIG_SCHEMA_ID`
  constants for facts that consumers may assert.

## Anti-Patterns

- Do not place official sidecar runtime code here.
- Do not mix multiple official addon IDs into one module.
- Do not generate constants dynamically when tests can assert stable facts.
