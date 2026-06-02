# nako-client-cli Backend Guidelines

`nako-client-cli` is the command-line wrapper around the Rust Public Client SDK.
It parses CLI arguments, resolves an optional token or token environment
variable, calls `nako-client`, and prints pretty JSON or safe streaming request
facts.

## Current Evidence

- `crates/nako-client-cli/src/lib.rs`
- `crates/nako-client-cli/src/main.rs`
- `crates/nako-client-cli/Cargo.toml`
- `crates/nako-client/src/lib.rs`

## Boundaries

- Own clap CLI shape.
- Own token resolution from `--token` or `--token-env`.
- Use `NakoClient` and `ClientTransport` for execution.
- Print pretty JSON for JSON API commands.
- Print redacted method/URL/header facts for streaming builder commands.
- Keep protocol DTOs and SDK transport behavior in lower crates.

## Executable Contract Summary

1. Scope / Trigger: new CLI command, argument, output shape, token resolution,
   streaming command, or CLI error update belongs here.
2. Signatures: `Cli`, `Command`, `PageArgs`, `PlaybackCapabilityArgs`,
   `StreamCommand`, `CliError`, `run`, and `run_with_transport`.
3. Contracts: default base URL is `http://127.0.0.1:3000`; explicit `--token`
   wins over `--token-env`; stream commands return safe request facts.
4. Validation & Error Matrix: missing token env becomes `MissingTokenEnv`;
   client errors pass through; serialization errors become `Serialize`.
5. Good/Base/Bad Cases: good stream output redacts Authorization; base health
   command does not send auth; bad output contains token strings.
6. Tests Required: clap parse, SDK transport requests, safe streaming output,
   token env behavior, and dependency boundary tests.
7. Wrong vs Correct: do not reimplement HTTP; construct `NakoClient` and use SDK
   methods or request builders.

## Required Patterns

- Route all execution through `run_with_transport` for testability.
- Use `ReqwestTransport::default()` only in `run`.
- Use `serde_json::to_string_pretty` for command output.
- Redact Authorization as `<redacted>` in streaming command output.
- Keep stream commands from sending transport requests.

## Forbidden Patterns

- Do not log or print raw tokens.
- Do not depend on server, API, core domain, streaming, or transcode crates.
- Do not add live network tests.
- Do not bypass `nako-client` SDK methods.

## Validation

- Focused:
  `cargo nextest run -p nako-client-cli --no-fail-fast`
- Client stack:
  `cargo nextest run -p nako-client -p nako-client-cli --no-fail-fast`
