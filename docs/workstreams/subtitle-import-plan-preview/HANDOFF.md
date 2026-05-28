# Subtitle Import Plan Preview Handoff

Status: complete.

Current task: none.

Completed:

- SIPP-010 opened the follow-on workstream for host-owned subtitle import plan
  preview.
- SIPP-020 added Admin subtitle import-plan DTOs and TypeScript contract
  entries.
- SIPP-030 implemented selected-reference import-plan preview with media
  item/source validation, sidecar file-name derivation, idempotency key, and
  redaction-safe plan output.
- SIPP-040 closed the lane with passing focused gates.

Next:

- Follow-on lane: subtitle content download/import apply through Library File
  Write.
- Later lane: refresh library subtitle facts and expose them to playback
  planning.

Watch points:

- Do not implement subtitle download, file write, import apply, or Library File
  Write apply.
- Do not expose Source Locators, paths, provider URLs, inline subtitle text, or
  artifact ids.
- Avoid the user's current web/Tauri files.
