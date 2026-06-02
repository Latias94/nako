# Database Guidelines

`nako-playback` has no database ownership.

## Rules

- Do not import `nako-db`, `sqlx`, or repository traits into this crate.
- Playback sessions, transcode sessions, renderer sessions, and user playback
  state are persisted through `nako-core` repository traits and `nako-db`
  adapters, then orchestrated in `nako-server`.
- Planner outputs may be persisted by callers, but this crate should remain
  deterministic over its input facts.

## Review Checklist

- Is the new field a planner fact or persisted runtime state?
- Should it live in `nako-core` playback/session records instead?
- Does `nako-server` own the transition from plan to runtime/session write?
