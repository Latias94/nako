# Backend Media Product Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. BMPD-020 is complete. Discovery found that Nako already has:

- consolidated SQLite/PostgreSQL baseline migration files;
- durable users, roles, Library Access, local password credentials, login,
  current-user, logout, and bearer session resolution;
- direct/remux/HLS playback, browser playback tickets, transcode sessions,
  hardware diagnostics, support evidence, and admin playback session lists;
- Admin API routes for jobs, scan, metadata profile, playback runtime, support
  evidence, access management, artwork, catalog governance, and NFO actions.

The remaining backend gaps are not empty features. They are product seams:

- invitation onboarding, not open registration;
- Playback Session, not Transcode Session as the user-facing playback attempt;
- permission-gated Management Context Links, not frontend hard-coded admin
  route knowledge.

BMPD-020 flattened the SQLite/PostgreSQL migration baselines so they describe
the current schema directly. The baselines no longer contain old `-- From`
markers, `ALTER TABLE`, add-column replay, drop-index/constraint cleanup, or
fresh-database data cleanup. Regression tests now reject those fragments.

## Active Task

- Task ID: BMPD-030
- Status: ready
- Scope: controlled invitation-based registration and redemption.

## Decisions

- Nako should use mature media engines for decoding/transcoding heavy lifting.
  FFmpeg remains server-side. Desktop can later use mpv/libmpv, GStreamer, or
  platform media frameworks. Web clients can express WebCodecs/WASM/MSE
  capabilities.
- Nako backend owns authority, Library Access, source selection, tickets,
  playback session state, jobs, and redacted diagnostics.
- Public self-registration remains closed by default. Invitation redemption is
  the correct first registration model.
- Management Context Links are backend-computed actions, not shared frontend
  privileged state.

## Blockers

- None.

## Next Action

Run BMPD-030: add invitation creation/list/revoke for administrators and public
invitation redemption into local user credentials plus a session.
