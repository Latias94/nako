# Addon Install Health Guide Evidence

Date: 2026-06-02
Selected slice: bounded Addon Install Guide and Addon runtime-readiness surfaces.

## Selection

Chose the existing bounded Addon onboarding slice because it is the smallest
useful step that improves operator onboarding without crossing into Addon
Manager, Docker socket control, process supervision, OAuth, or cross-repo
official addon implementation scope.

The selected slice is already present in the current branch baseline and is
bounded to:

- typed `AddonInstallDescriptor` and `AddonInstallGuide` protocol contracts;
- redaction-safe Addon install guide preview and registered Addon install guide
  Admin surfaces;
- redaction-safe Addon Health Check and runtime-readiness Admin surfaces;
- official addon catalog resolve/install-guide behavior that preserves Addon
  Protocol compatibility and lifecycle boundaries.

## Audit Summary

- `crates/nako-addon-protocol` already owns additive install-guide wire types,
  install descriptor validation, runtime reference validation, and redaction-safe
  tests for secret values and local runtime paths.
- `crates/nako-addon-client` already owns bounded Addon Health Check calling and
  transport error redaction without leaking request URLs, query tokens, or auth
  material.
- `crates/nako-official-addon-catalog` already resolves official Addon Package
  facts through validated install descriptors without taking lifecycle control.
- `crates/nako-server` already exposes Admin install-guide preview, official
  catalog resolve, Addon Health Check, and Addon runtime-readiness routes.
- `crates/nako-api` already exposes typed Admin DTOs and generated contract
  coverage for these surfaces.

## Boundaries Preserved

- No Addon Protocol breaking change; compatibility remains additive and the
  current Addon Protocol Version is unchanged.
- No Docker socket control.
- No process supervision or Addon Manager lifecycle ownership.
- No OAuth flow.
- No cross-repo `nako-official-addons` implementation work.
- No native in-process plugin ABI.
- No addon token, endpoint credential, local runtime path, or resolved Secret
  Reference value is exposed in install guides, health output, or runtime
  readiness diagnostics.

## Validation

- `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-official-addon-catalog --no-fail-fast`
  passed: 78 tests.
- `cargo nextest run -p nako-server -E 'test(admin_addon_install_guide_preview_redacts_package_and_secret_material) or test(admin_addon_install_guide_preview_rejects_raw_secret_and_local_runtime_paths) or test(admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work) or test(admin_addon_source_catalog_resolves_notification_bridge_event_manifest) or test(admin_addon_source_catalog_resolves_chromecast_renderer_adapter_manifest) or test(admin_addon_source_catalog_resolves_resource_search_link_check_manifest) or test(admin_addon_source_catalog_resolves_external_acquisition_runner_action_manifest) or test(admin_addon_source_catalog_resolves_subtitle_provider_manifest) or test(admin_addon_source_catalog_resolves_dlna_renderer_manifest) or test(admin_addon_runtime_readiness_reports_ready_sidecar_without_token_or_payload_echo) or test(admin_addon_runtime_readiness_preserves_sidecar_degraded_status) or test(admin_addon_runtime_readiness_classifies_local_gaps_without_sidecar_call) or test(admin_addon_runtime_readiness_classifies_network_policy_blockers_without_echoing_url) or test(admin_addon_runtime_readiness_classifies_protocol_manifest_and_unsafe_responses_safely)' --no-fail-fast`
  passed: 14 tests.
- `cargo nextest run -p nako-api admin_contract_excludes_generated_fetch_runtime_and_raw_sensitive_fields --no-fail-fast`
  passed: 1 test.
- `cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output --no-fail-fast`
  passed: 1 test.
- `cargo fmt --all -- --check` passed.
- `python ./.trellis/scripts/task.py validate 06-02-04d-addon-install-health-guide`
  passed.

## Follow-ons

- Addon Manager package inventory, install/update/remove lifecycle, and host-side
  process policy remain a separate follow-on.
- Hosted settings depth and stronger hosted-surface policy remain separate
  follow-ons.
- Addon Token rotation UX and operator workflows remain separate follow-ons even
  though the bounded slice preserves current token redaction boundaries.
- Official provider breadth across more addon packages remains separate from this
  onboarding slice.
- Any cross-repo official addon implementation work remains out of scope until a
  planner explicitly assigns related repository coordination.

## Session Outcome

- No product-code patch was required in this session because the requested slice
  is already present in the current branch baseline.
- This session closed the Trellis task with fresh validation and evidence so the
  task can be reported as done from the current baseline.

## Fresh Integration Evidence

Date: 2026-06-03

- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-official-addon-catalog --no-fail-fast`
  passed: 78 tests.
- `cargo nextest run -p nako-server -E 'test(admin_addon_install_guide_preview_redacts_package_and_secret_material) or test(admin_addon_install_guide_preview_rejects_raw_secret_and_local_runtime_paths) or test(admin_addon_source_catalog_browses_and_resolves_without_hidden_lifecycle_work) or test(admin_addon_source_catalog_resolves_notification_bridge_event_manifest) or test(admin_addon_source_catalog_resolves_chromecast_renderer_adapter_manifest) or test(admin_addon_source_catalog_resolves_resource_search_link_check_manifest) or test(admin_addon_source_catalog_resolves_external_acquisition_runner_action_manifest) or test(admin_addon_source_catalog_resolves_subtitle_provider_manifest) or test(admin_addon_source_catalog_resolves_dlna_renderer_manifest) or test(admin_addon_runtime_readiness_reports_ready_sidecar_without_token_or_payload_echo) or test(admin_addon_runtime_readiness_preserves_sidecar_degraded_status) or test(admin_addon_runtime_readiness_classifies_local_gaps_without_sidecar_call) or test(admin_addon_runtime_readiness_classifies_network_policy_blockers_without_echoing_url) or test(admin_addon_runtime_readiness_classifies_protocol_manifest_and_unsafe_responses_safely)' --no-fail-fast`
  passed: 14 tests.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed:
  6 tests.
- `python ./.trellis/scripts/task.py validate 06-02-04d-addon-install-health-guide`
  passed.
