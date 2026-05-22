# Client CLI Entrypoint Milestones

Status: Completed
Last updated: 2026-05-17

## M37.0 Scope And Boundary Baseline

Outcome: M37 is opened with a clear client CLI scope and license boundary.
Status: completed.

Exit evidence:

- Workstream docs exist under `docs/workstreams/client-cli`.
- The design states that the CLI is Apache-2.0 and must not depend on AGPL
  server/internal crates.

## M37.1 CLI Crate Slice

Outcome: `crates/nako-client-cli` provides the first public-client CLI.
Status: completed.

Exit evidence:

- The crate manifest uses `license = "Apache-2.0"`.
- Commands go through `nako-client`.
- JSON output is available for health, library/item/search, source probe,
  playback decision, playback session, and streaming request builders.
- `cargo check -p nako-client-cli --tests` passes.

## M37.2 Test And Dependency Boundary Slice

Outcome: The CLI behavior and dependency boundary are test-visible.
Status: completed.

Exit evidence:

- Mocked transport tests prove JSON commands call the SDK with the expected
  method, path, query, and authorization behavior.
- Streaming request output redacts bearer token values.
- Manifest tests reject AGPL Nako server/internal dependencies.
- `cargo nextest run -p nako-client-cli --no-fail-fast` passes.

## M37.3 Docs And Closeout

Outcome: M37 is documented and ready to hand off to the next client slice.
Status: completed.

Exit evidence:

- Goal, roadmap, API, and workstream docs are updated.
- `cargo tree -p nako-client-cli` confirms the dependency boundary.
- Workspace check and nextest gates pass.
