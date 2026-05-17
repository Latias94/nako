# Playback Source Selection Deepening Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M43 is complete. `PSSD-020`, `PSSD-030`, `PSSD-040`, and `PSSD-050` are
implemented, focused gates pass, and workspace closeout gates pass.

## Follow-Ons

- Metadata Provider Attempt Runtime Extraction: split provider attempt,
  classification, raw response, and refresh commit orchestration inside
  `taru-metadata`.
- `taru-api` module split: separate **Public Client API**, **Admin API**,
  metadata diagnostics, storage diagnostics, and extension DTO modules.
- NFO Round Trip: define preservation, unknown XML retention, local-field
  conflict reporting, and partial update behavior before library file writes.
- Typed VFS storage errors: replace HTTP error string classification with
  stable storage error categories.
- Playback follow-on: deepen client profiles, subtitle/audio selection, HDR,
  bitrate, remote endpoint, and **Source Variant** selection.

## Cautions

- Do not absorb Android client implementation into M43.
- Do not redo M42 catalog hydration work.
- Existing unstaged Android workstream files may belong to another terminal;
  do not stage, revert, or edit them unless explicitly asked.
