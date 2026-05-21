# Phase 5.5: Reference Addon and Stabilization

Status: completed.

## Goal

Prove the M5 extension and automation surface end to end with a minimal local
reference addon, user-facing protocol documentation, and a final M5 limitations
audit.

## Completed Shape

- Added `taru-reference-addon`, a minimal HTTP addon fixture crate.
- The reference addon exports `reference_manifest(base_url)` and
  `build_router()`.
- The reference addon implements a `metadata` resource that accepts the Taru
  addon request envelope and returns a matching response envelope with a
  metadata suggestion artifact.
- Added a server end-to-end test that starts the local reference addon,
  registers it through the Addon management HTTP surface, queries it by Addon
  ID, and calls its metadata resource through `ReqwestAddonTransport`.
  Architecture deepening later moved the management HTTP surface from the
  original root `/addons` routes to `/admin/v1/addons`.
- Added addon author, webhook receiver, and automation provider guides.
- Updated HTTP API, roadmap, goal map, workstream milestones, and TODO state.

## Known M5 Limitations

- Addon resource calls are available as protocol/application library behavior;
  no public server route invokes addon resources yet.
- The reference addon is a fixture and protocol example, not a real metadata
  provider.
- Addon runtime secret resolution is modeled at the caller boundary, but there
  is no persisted addon secret-reference field yet.
- There is no addon SDK, generated JSON schema package, compatibility layer for
  other addon ecosystems, embedded JavaScript runtime, or native plugin ABI.
- Automation artifacts remain proposed outputs. Canonical metadata writeback
  still requires a future acceptance policy.
- Webhook dispatch is explicit and inspectable, but there is no always-on
  background retry scheduler yet.

## Validation

Coverage:

- `taru-reference-addon` validates its reference manifest.
- `taru-server` starts the reference addon in a local TCP listener and exercises
  registration, query, and real HTTP resource call behavior.
- Existing webhook and automation tests continue to cover secret omission,
  persistence, delivery attempts, job enqueue, artifact persistence, and safe
  error mapping.
- Workspace gates pass when run after this phase: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo nextest run --workspace`, and
  `git diff --check`.
