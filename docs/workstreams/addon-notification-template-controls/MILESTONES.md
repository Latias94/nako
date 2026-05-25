# Addon Notification Template Controls — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Safe Template Contract

Exit criteria:

- Allowed tokens and rejected behavior are explicit.
- Raw payload values are out of scope.

## M1 — Renderer Proof

Exit criteria:

- Renderer substitutes only whitelisted fields.
- Unknown tokens fail safely.
- Tests prove raw event payload values are unavailable.

## M2 — Provider Wiring

Exit criteria:

- Providers can use safe rendered summaries.
- Defaults preserve existing behavior.
- ACK and error output stay redaction-safe.

## M3 — Docs And Smoke

Exit criteria:

- Operator docs explain safe tokens and ownership.
- Default smoke remains secret-free.

## M4 — Closeout

Exit criteria:

- Final gates pass and follow-ons are named.

Result: complete. Final gates passed; Admin UI/API template management remains
out of scope.
