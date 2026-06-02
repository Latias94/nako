# Quality Guidelines

CLI changes must remain testable without a server and must keep output safe.

## Required Patterns

- Use `Cli::parse_from` in tests for command shape.
- Use mock `ClientTransport` for SDK-backed commands.
- Assert exact request method, URL, headers, and output JSON for commands.
- Assert stream commands do not call transport.
- Assert token strings are absent from output.

## Forbidden Patterns

- Do not add live server tests.
- Do not add non-JSON success output.
- Do not leak token values in safe request output.
- Do not add dependencies on server-side crates.

## Tests Required

- Health command auth behavior.
- Search command query and pagination behavior.
- Stream direct/head/remux/HLS command safe-output behavior.
- Token env resolution tests when touched.
- Cargo manifest dependency boundary test.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-client-cli --no-fail-fast`
- Client stack:
  `cargo check -p nako-client-cli -p nako-client --tests`
