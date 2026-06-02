# Logging Guidelines

The CLI does not use structured logging today. It prints successful JSON to
stdout and errors to stderr.

## Required Patterns

- Keep stdout reserved for command output.
- Keep stderr reserved for `error: ...` messages.
- Redact Authorization headers in streaming command output.
- Prefer JSON fields over ad hoc logs for successful command facts.

## Forbidden Patterns

- Do not print raw tokens, login passwords, bearer headers, or playback tickets.
- Do not mix debug logs with JSON output on stdout.
- Do not print full reqwest/debug errors if they include sensitive URLs.

## Useful Fields

- `cli.command`
- `cli.method`
- `cli.url`
- `cli.error`
