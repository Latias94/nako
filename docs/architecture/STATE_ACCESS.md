# State, Database, And Access Architecture

Last updated: 2026-05-29

This document maps Nako's persistence, playback state, identity, and access
boundaries.

## Target Chain

```text
Authenticated Principal
  -> Library Access / Playback Policy
  -> Playback or Admin operation
  -> bounded repository transaction
  -> public/admin DTO with redaction
  -> event/outbox/update where applicable
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| SQLite default | Shipped | deployment docs; DB workstreams | Playback write pressure tests. |
| PostgreSQL-ready boundary | Shipped foundation | `docs/adr/0029-postgresql-ready-persistence-boundary.md`; `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md` | Parity for newer feature tables. |
| Repository traits | Shipped foundation | `docs/adr/0001-modular-monolith-rust-workspace.md` | Keep domain traits out of adapters. |
| Local credential auth | Shipped | `docs/adr/0037-local-credential-and-session-auth.md` | Account recovery/SSO follow-ons. |
| Library access | Shipped foundation | identity/access lanes | Fine-grained content policies. |
| Playback policy | Partial | `docs/adr/0039-playback-policy-and-renderer-target-boundary.md` | Remote bitrate, transcode, session-limit policies. |
| User playback progress | Shipped foundation | `docs/adr/0028-user-playback-state-principal-and-public-contract.md` | Heartbeat buffering and conflict semantics. |
| Transcode/playback sessions | Shipped foundation | playback runtime lanes | Active-session limits and write pressure tests. |
| Search projection | Shipped foundation | catalog/search lanes | FTS/filter scale-up. |
| Event outbox | Shipped | `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md` | Realtime client gateway bridge. |

## Workstream Evidence

Use
`docs/architecture/WORKSTREAM_LINKS.md#state-database-identity-and-access` as
the consolidated index for persistence, identity, access, and state workstreams.
Keep this document focused on state and access capability pressure.

## Next Work Lanes

### playback-db-write-pressure-and-wal-policy

Goal: Prove playback heartbeat, transcode metrics, scan jobs, and user state do
not cause unacceptable SQLite lock contention.

Scope:

- SQLite WAL/busy-timeout deployment policy;
- connection pool sizing;
- heartbeat write coalescing;
- transcode metric update frequency;
- pressure tests with concurrent playback and scan writes.

Exit criteria:

- focused tests exercise concurrent heartbeat/session writes;
- docs explain SQLite and PostgreSQL operational expectations;
- public API remains responsive under simulated write pressure.

### playback-access-policy-and-session-limits

Goal: Let self-hosted operators control playback cost and access by user.

Scope:

- per-user remote playback permission;
- max remote bitrate;
- allow/deny transcode;
- active session count or resource limit;
- idle session termination policy.

## Risk Register

### SQLite Is Good But Not Infinite

SQLite is the right default for many self-hosted installs, but playback creates
frequent writes. Heartbeat and metrics writes should be bounded and coalesced
where practical.

### Public DTOs Must Stay Redacted

Playback and transcode internals include paths, request keys, FFmpeg commands,
stderr, and storage locators. Public/Admin DTOs must expose safe status and
diagnostics without leaking host details.

### Policy Must Run Before Expensive Work

Library access, playback permission, remote access, and transcode permission
must be checked before staging or FFmpeg startup.

## Agent Notes

When changing persistence behavior, update SQLite and PostgreSQL tests together
where a repository contract is shared. When changing public playback state,
verify Public Client redaction tests and Admin diagnostics tests.
