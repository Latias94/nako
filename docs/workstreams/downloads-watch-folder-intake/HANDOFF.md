# Downloads / Watch-Folder Intake — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is open as the next mainline child of `post-rpd-product-hardening`
after Playback/Transcode Ops Hardening closed.

The prerequisites are complete:

- `metadata-provider-breadth` made provider capability, match ambiguity, and
  cross-provider conflict review explainable.
- `nfo-link-authority` made local NFO/link authority and duplicate evidence
  non-mutating and explicit.
- `managed-import-staging` added durable Managed Import artifacts and
  non-mutating promotion preview.
- `link-apply-and-import-promotion` added accepted promotion apply, VFS-mediated
  target creation, catalog commit, duplicate evidence, and cleanup/rollback
  audit.
- `nfo-sidecar-promotion-apply` added accepted NFO sidecar import/export apply,
  backup, retention, rollback/repair, and redacted diagnostics.
- `playback-transcode-ops-hardening` added playback readiness, validation,
  failure taxonomy, and bounded Admin support evidence.

DWI-010 is complete. The lane is scoped to acquisition intake and watch-folder
candidate discovery, not built-in downloader protocols or direct library writes.

DWI-020 is complete. It added acquisition intake candidate IDs, source kinds,
states, records, repository traits, SQLite/PostgreSQL migrations and adapters,
facade dispatch, backend capability flags, and a backend-neutral contract for
round-trip, idempotent source-key lookup, state transitions, Managed Import
artifact linking, and list filters. It does not create Media Sources, run
promotion apply, or write library files.

## Active Task

- Task ID: DWI-030
- Owner: unassigned
- Files:
  - `crates/taru-server/src/app`
  - `crates/taru-server/src/app/tests`
- Validation:
  - `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`
  - focused Managed Import regression tests if shared handoff paths change
- Status: READY
- Review: app-service intake must prove idempotency, redaction, Managed Import
  handoff semantics, and no direct Media Source creation or Library File Write.

## Decisions Since Opening

- The lane name is Downloads / Watch-Folder Intake, but the first implementation
  slice is acquisition-intake domain/persistence.
- Watch folders are candidate sources, not trusted library roots.
- Intake candidates are not Media Sources.
- Intake acceptance creates or links Managed Import artifacts; promotion apply
  and NFO sidecar apply remain separate accepted workflows.
- VFS/storage list/stat primitives should own path safety for watch-folder
  discovery.
- Admin diagnostics are allowed; Public Client API and `taru-client-protocol`
  changes are not.
- Protocol-specific download clients, network traversal, AI, Addon runtime, UI
  polish, background scheduling, and automatic apply behavior are follow-ons
  unless explicitly opened.
- DWI-020 kept the boundary persistence-only. Candidate acceptance links a
  Managed Import artifact at the repository level, but app-service semantics
  for creating or reusing artifacts belong to DWI-030.

## Blockers

- None for DWI-030.

## Next Recommended Action

Execute DWI-030 with TDD:

1. add a failing server app test for recording/listing a redacted watch-folder
   intake candidate;
2. add a second failing test for accepting a candidate into a Managed Import
   artifact without promotion apply or Media Source creation;
3. implement the narrow app-service seam;
4. verify focused server gate plus relevant DB/Managed Import regressions;
5. update evidence before moving to DWI-040.
