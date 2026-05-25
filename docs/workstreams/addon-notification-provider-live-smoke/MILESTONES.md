# Addon Notification Provider Live Smoke — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Live Smoke Contract

Exit criteria:

- Opt-in env vars and skip behavior are explicit.
- CI and package gates do not require live secrets.

## M1 — Opt-In Script

Exit criteria:

- Script skips by default.
- Script parser check passes.
- Safe output assertions are documented.

## M2 — Docs And Release Notes

Exit criteria:

- Operator docs explain local-only live smoke.
- No example secret or live endpoint is committed.

## M3 — Closeout

Exit criteria:

- Final gates pass and platform-specific live smoke is deferred or split.

Result: complete. Parser and default-skip gates passed; platform-specific live
smoke remains operator-provided through env.
