# Backend Media Product Deepening - Milestones

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- Scope, non-goals, risks, and reference research are documented.
- Task ledger has vertical slices with gates.
- `WORKSTREAM.json` is valid.

## M1 - Clean Baseline

Exit criteria:

- SQLite and PostgreSQL baselines describe the final schema directly.
- Historical migration replay artifacts are removed where safe.
- Database migration and repository tests pass.
- Identity, addon, job, and playback schema parity is preserved.

## M2 - Invitation Registration

Exit criteria:

- Admin API can create/list/revoke redacted invitations.
- Public Client API can redeem an invitation into a local user/session.
- Raw invitation tokens and hashes never appear in list/detail responses.
- Redemption is atomic and expiry/reuse-disabled behavior is tested.

## M3 - Playback Session Runtime

Exit criteria:

- Playback Session is distinct from Transcode Session in domain/API language.
- Direct Play, Remux, and HLS all create or expose coherent playback attempts.
- Optional transcode artifacts remain inspectable without leaking paths.
- Admin current-connections/playback diagnostics can report direct and
  transcode-backed playback.

## M4 - Management Context Links

Exit criteria:

- Context links are computed by the backend from principal, role, Library
  Access, and target context.
- Media contexts can discover scan, metadata refresh, job, playback diagnostic,
  and settings/admin actions without embedding Admin API data.
- Viewer and no-access users do not receive enabled admin actions.
- Tests cover administrator, library manager, viewer, disabled user, and
  bootstrap admin cases.

## M5 - Closeout

Exit criteria:

- Evidence is fresh and recorded.
- Follow-ons are split rather than hidden in TODO comments.
- Workstream status and handoff match the actual implementation state.
