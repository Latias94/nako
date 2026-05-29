# Subtitle Search Candidate Selection Handoff

Status: complete.

Current task: none.

Completed:

- SSCS-010 opened the follow-on workstream for host-owned subtitle candidate
  selection.
- SSCS-020 added typed subtitle search helpers to `nako-addon-client`, including
  grant and schema coverage.
- SSCS-030 added Admin/App subtitle search and selected-reference endpoints with
  short-lived host sessions and redaction-safe candidate cards.
- SSCS-040 regenerated Admin TypeScript contracts and closed the lane with
  passing focused gates.

Next:

- Follow-on lane: subtitle import planning from selected refs and target media
  identity.
- Later lane: Library File Write apply and library subtitle fact refresh.

Watch points:

- Do not touch concurrent web route files unless the user redirects.
- Do not add import planning, download execution, or Library File Write apply in
  this lane.
- Selection refs remain short-lived and in-memory; durable selected subtitles
  should be introduced with the import plan model, not by exposing provider
  payloads to clients.
