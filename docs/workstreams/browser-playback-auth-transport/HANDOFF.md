# Browser Playback Auth Transport - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

This lane was opened after Media Web Client Foundation closed. Media Web has a
safe watch shell, but no real browser player because bearer-only `<video src>`
cannot attach Authorization headers.

The recommended default direction is short-lived playback tickets, pending
BPAT-010 decision review.

## Active Task

- Task ID: BPAT-010
- Owner: planner
- Files: `docs/workstreams/browser-playback-auth-transport`, optional
  `docs/adr`
- Validation: `python -m json.tool docs/workstreams/browser-playback-auth-transport/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/browser-playback-auth-transport docs/workstreams/README.md`
- Status: READY
- Review: Compare short-lived playback tickets, cookie/session auth, and
  JavaScript HLS/MSE with headers before implementation.
- Evidence: update `EVIDENCE_AND_GATES.md`

## Decisions To Make First

- Is short-lived playback ticket the accepted MVP transport?
- Does ticket issuance need a new ADR or an update to ADR 0024/0028?
- Which playback modes are MVP: direct stream, remux, HLS, or a subset?
- Should Library Access be revalidated at ticket use, issuance, or both?
- How are HLS segment URLs protected?

## Next Recommended Action

Run BPAT-010 and freeze the transport decision before touching server stream
routes.
