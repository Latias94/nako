# Database Guidelines

`nako-transcode` does not own database persistence.

## Rules

- Transcode session records and repository traits live in `nako-core`.
- SQLite/Postgres adapters live in `nako-db`.
- Server app services decide when to persist transcode sessions, artifacts, and
  runtime outcomes.
- `nako-transcode` may produce typed session IDs, manifests, plans, and runtime
  summaries that callers persist.

## Review Checklist

- Is this a plan/artifact/runtime value or durable session state?
- Should a new durable field be added to `nako-core` and `nako-db` instead?
- Does the caller own resource admission and session lifecycle?
