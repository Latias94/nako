# Addon Notification Platform Adapters — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Scope And Adapter Freeze

Exit criteria:

- `discord_webhook` is confirmed or replaced before implementation starts.
- Credential, template, retry, and redaction ownership are explicit.

## M1 — Platform Configuration And Diagnostics

Exit criteria:

- Provider is disabled by default.
- Health and diagnostics expose only safe booleans/status strings.
- No raw URL, secret, message body, or platform token is echoed.

## M2 — Fixture-Backed Platform Send

Exit criteria:

- Local fixture receives exactly one platform-shaped request when configured.
- ACK output remains safe.
- Retryable and non-retryable provider failures map to safe sidecar failures.

## M3 — Docs And Smoke Alignment

Exit criteria:

- Operator docs describe env vars and default-disabled behavior.
- Default smoke does not require live provider credentials.

## M4 — Closeout

Exit criteria:

- Final gates pass.
- Remaining platform breadth is split or deferred by name.

Result: complete. Final gates passed; additional platform breadth is deferred
to future named adapter lanes.
