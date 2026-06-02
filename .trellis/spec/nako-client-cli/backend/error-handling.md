# Error Handling

CLI errors should be concise for users and safe for terminals.

## Required Patterns

- Use `CliError::MissingTokenEnv` when a named token environment variable is not
  set.
- Pass through `NakoClientError` with `CliError::Client`.
- Use `CliError::Serialize` for JSON output serialization failures.
- In `main`, print `error: {err}` to stderr and return failure exit code.
- Keep successful output on stdout.

## Forbidden Patterns

- Do not print raw tokens or Authorization headers.
- Do not panic for missing environment variables.
- Do not unwrap SDK responses outside tests.
- Do not hide SDK errors behind generic CLI failure strings.

## Examples

- `health` with a token still sends no Authorization header.
- `stream remux` prints method, URL, and redacted headers without sending.
- `stream hls-segment` encodes session and segment path names and uses no
  transport call.

## Review Checklist

- Is the user-facing error actionable?
- Could it leak a token?
- Does a mock transport test cover the path?
