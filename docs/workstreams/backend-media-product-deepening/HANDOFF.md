# Backend Media Product Deepening - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is open. BMPD-020, BMPD-030, and BMPD-040 are complete. Discovery found that
Nako already has:

- consolidated SQLite/PostgreSQL baseline migration files;
- durable users, roles, Library Access, local password credentials, login,
  current-user, logout, and bearer session resolution;
- direct/remux/HLS playback, browser playback tickets, transcode sessions,
  hardware diagnostics, support evidence, and admin playback session lists;
- Admin API routes for jobs, scan, metadata profile, playback runtime, support
  evidence, access management, artwork, catalog governance, and NFO actions.

The remaining backend gaps are not empty features. They are product seams:

- Playback Session, not Transcode Session as the user-facing playback attempt;
- permission-gated Management Context Links, not frontend hard-coded admin
  route knowledge.

BMPD-020 flattened the SQLite/PostgreSQL migration baselines so they describe
the current schema directly. The baselines no longer contain old `-- From`
markers, `ALTER TABLE`, add-column replay, drop-index/constraint cleanup, or
fresh-database data cleanup. Regression tests now reject those fragments.

BMPD-030 added controlled invitation onboarding. Administrators can create,
list, and revoke invitations. Public clients can redeem a one-time invitation
into a local user credential and session. Invitation tokens are only returned
at creation time; stored state uses token hashes and list responses are
redacted.

BMPD-040 added durable Playback Sessions. Playback Session is now the
user/client playback attempt; Transcode Session is an optional remux/HLS
artifact. Direct play records sessions without fake transcode rows. Remux/HLS
headers expose Playback Session IDs, cancellation acts on the linked artifact
when present, HLS segment routes accept Playback Session IDs while preserving
legacy transcode-id segment compatibility, and public/admin/generated contracts
use Playback Session DTOs for user-facing state. The public route inventory now
also includes invitation redemption and playback heartbeat.

## Active Task

- Task ID: BMPD-050
- Status: ready
- Scope: Management Context Links between media/library/playback contexts and
  permission-gated admin operations.

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

Run BMPD-050: add backend-computed Management Context Links so media clients
can discover safe scan, metadata refresh, job, playback diagnostic, runtime
settings, and access actions without embedding Admin Web route knowledge.
