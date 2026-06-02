# Directory Structure

`nako-client-cli` keeps command parsing and execution in `src/lib.rs`, with a
small binary entry point in `src/main.rs`.

## Current Layout

- `src/main.rs`: parses `Cli`, calls `run`, prints output, maps errors to
  process exit code.
- `src/lib.rs`: clap command definitions, token resolution, client construction,
  command execution, streaming safe output, and tests.

## Command Families

- `health`
- `libraries`
- `items`
- `search`
- `source probe`
- `playback decision/session/cancel`
- `stream direct/head/remux/hls-playlist/hls-segment`

## Module Rules

- Keep CLI argument definitions close to execution code while the crate is small.
- Split command families only if each gets enough options to justify it.
- Keep `SafeRequestOutput` private.
- Keep tests in this crate using mock transport.

## Naming Rules

- Use `*Args` for flattened clap arguments.
- Use `*Command` for clap subcommands.
- Use `run_with_transport` for test execution.

## Anti-Patterns

- Do not add SDK transport code here.
- Do not add public client DTO definitions here.
- Do not make `main.rs` contain command logic.
