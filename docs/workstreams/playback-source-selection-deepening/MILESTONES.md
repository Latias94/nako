# Playback Source Selection Deepening Milestones

Status: Completed
Last updated: 2026-05-17

## M43.0 - Workstream Baseline

Exit criteria:

- M43 is recorded as the active goal.
- The workstream records problem, target state, non-goals, and validation
  gates.
- Follow-on concerns are ranked so M43 does not absorb unrelated refactors.

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`
- Completed.

## M43.1 - Deep Selection Model

Exit criteria:

- `nako-streaming` exposes a workflow-shaped **Playback Source Selection**
  Interface.
- Existing container/codec/direct-play behavior is tested through the new
  Interface.
- The decision model has explicit placeholders or extension points for future
  client profile facts without forcing a full implementation.

Expected gates:

- `cargo fmt --all -- --check`
- `cargo check -p nako-streaming --tests`
- `cargo nextest run -p nako-streaming --no-fail-fast`

Evidence:

- Completed with 8 `nako-streaming` tests passing.

## M43.2 - Server Playback Migration

Exit criteria:

- `nako-server` playback app loads facts, calls `nako-streaming`, and executes
  the returned direct/remux/transcode intent.
- Mode-choice reasoning is not duplicated in HTTP or app orchestration.
- Existing playback route behavior remains compatible.

Expected gates:

- `cargo check -p nako-server --tests`
- focused `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`

Evidence:

- Completed with 16 focused server playback route tests passing.

## M43.3 - Public Contract Check

Exit criteria:

- Public playback DTO mapping is explicit.
- Existing TypeScript/Rust SDK compile-contract expectations remain valid, or
  additive changes are documented and tested.
- Server-only playback planning types do not enter permissive protocol crates
  unless they are stable **Public Client API** wire types.

Expected gates:

- `cargo check -p nako-api --tests`
- `npm run check --prefix sdk/typescript` if generated SDK output changes

Evidence:

- Completed with 12 `nako-api` tests passing.
- No public DTO shape changed, so TypeScript SDK regeneration was not required.

## M43.4 - Closeout

Exit criteria:

- Workstream evidence is complete.
- `docs/GOALS.md` marks M43 completed and recommends the next goal.
- Follow-on architecture risks are carried forward without bloating M43.

Expected gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

Evidence:

- Completed with workspace check passing and 292 workspace tests passing.
