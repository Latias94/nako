# nako-addon-protocol Backend Guidelines

`nako-addon-protocol` is the permissive wire-contract crate for Addon Sidecar
authors and integration tools. It must stay independent from Nako server
internals and expose explicit manifest, runtime, resource, task, event, health,
side-effect, and install-guide contracts.

## Current Evidence

- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-protocol/README.md`
- `CONTEXT.md`
- `AGENTS.md`

## Boundaries

- Define Addon Protocol Version and supported wire shapes.
- Validate manifests, install descriptors, resource/task/event/health response
  envelopes, scope grants, secret references, and runtime requirements.
- Keep Addon Sidecars out of process. Do not model native plugin execution here.
- Keep server persistence, HTTP routing, and grant storage outside this crate.
- Keep this crate suitable for addon authors through a permissive license.

## Executable Contract Summary

1. Scope / Trigger: any manifest, route, schema, scope, side-effect, task, event,
   or health wire shape change updates this crate first.
2. Signatures: public types such as `AddonManifest`, `AddonResourceRequest`,
   `AddonTaskRequest`, `AddonEventRequest`, `AddonHealthCheckRequest`, and
   validation helpers are the contract.
3. Contracts: current protocol version is `0.1.0-alpha.1`; runtime routes use
   `POST`; paths are constants under `/addon/v1/...`.
4. Validation & Error Matrix: invalid base URL, empty manifest surface,
   unsupported protocol version, duplicate resources, missing scopes, invalid
   runtime references, and secret-looking bindings return `AddonManifestError`.
5. Good/Base/Bad Cases: good manifests declare resources/tasks/events with
   scopes; base case uses HTTP sidecar runtime; bad cases include local binary
   paths, duplicate resources, and plaintext secret values.
6. Tests Required: serialization, validation, redaction-safe `Debug`, install
   guide, grant checks, and response-envelope matching.
7. Wrong vs Correct: do not infer server grants from manifest declarations;
   validate declarations here and enforce accepted grants in callers.

## Required Patterns

- Use `ADDON_PROTOCOL_VERSION` and `SUPPORTED_ADDON_PROTOCOL_VERSIONS` for
  compatibility checks.
- Add new runtime paths to `ADDON_RUNTIME_ROUTES`.
- Use `serde(rename_all = "snake_case")` for public enum wire values.
- Keep sensitive payloads out of custom `Debug` implementations.
- Validate response envelopes against manifest ID, protocol version, resource or
  task ID, job ID, event ID, and request ID.

## Forbidden Patterns

- Do not depend on `nako-server`, database crates, or storage adapters.
- Do not store plaintext secrets in install descriptors.
- Do not treat Addon Package or Addon Suite as the permission unit.
- Do not make breaking wire changes without a new Addon Protocol Version.

## Validation

- Focused:
  `cargo nextest run -p nako-addon-protocol --no-fail-fast`
- Cross-layer contract:
  `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api --tests`
