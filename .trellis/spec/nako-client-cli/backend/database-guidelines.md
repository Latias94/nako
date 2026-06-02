# Database Guidelines

`nako-client-cli` has no persistence. It executes public client commands and
prints results.

## Required Patterns

- Treat IDs and tokens as CLI inputs.
- Let `nako-client` build and send requests.
- Let server-side APIs enforce persistence, auth, access, and pagination.
- Keep command outputs as JSON values from SDK responses or safe request facts.

## Forbidden Patterns

- Do not import repository traits, SQL adapters, database pools, or migrations.
- Do not persist tokens, sessions, playback state, or command history.
- Do not read local configuration files for secrets unless that behavior is
  explicitly added and tested.
- Do not infer server database behavior from CLI output.

## Contract Rules

- `--token` takes precedence over `--token-env`.
- Missing `--token-env` value returns `CliError::MissingTokenEnv`.
- Stream commands construct request facts and do not use transport.

## Tests Required

- Token resolution tests when behavior changes.
- Mock transport tests for SDK-backed commands.
- Safe output tests for streaming commands.
