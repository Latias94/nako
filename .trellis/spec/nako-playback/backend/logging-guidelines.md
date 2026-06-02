# Logging Guidelines

`nako-playback` should not emit runtime logs.

## Rules

- Use `PlaybackDecisionReport` for planner diagnostics.
- Runtime tracing belongs in `nako-server` playback services and transport
  layers.
- Do not add logging side effects to pure compatibility evaluation.

## Evidence

- `crates/nako-playback/src/capability.rs`
- `crates/nako-server/src/app/playback/*`
