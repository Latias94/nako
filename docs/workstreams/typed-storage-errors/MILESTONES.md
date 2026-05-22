# Typed Storage Errors Milestones

Status: Completed
Last updated: 2026-05-17

## M45.0 - Workstream Baseline

Exit criteria:

- M45 is recorded as the active goal.
- Scope excludes new storage backends, public API expansion, NFO, playback
  source selection, and DB schema changes.

Status: Completed.

## M45.1 - Typed HTTP Mapping

Exit criteria:

- `NakoError::Storage` carries a typed storage error kind.
- HTTP status/code/message mapping uses the typed kind.
- Existing public storage error code behavior remains stable.

Status: Completed.

## M45.2 - Source Classification

Exit criteria:

- WebDAV request failures and HTTP status failures are categorized.
- Staging budget and validation failures are categorized.
- Local/playback filesystem failures have generic IO/security/resource-budget
  categories where applicable.

Status: Completed.

## M45.3 - Closeout

Exit criteria:

- Workstream evidence is complete.
- Workspace gates pass.
- Follow-ons are recorded without expanding M45.

Status: Completed.
