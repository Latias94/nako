# FFmpeg Hardware Pipeline Planner - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- ADR 0045 exists and references this lane.
- Workstream docs exist with task ledger and gates.

Result: Complete.

## M1 - Capability Inventory

Exit criteria:

- Hardware report records stage capability evidence.
- Encoder-only helper names are removed or reduced to compatibility-free test
  helpers.
- Tests cover listed, missing, and probe-error capability facts.

Result: Complete.

## M2 - Pipeline Planner

Exit criteria:

- Pipeline planner returns typed stage plans for software, NVENC, VAAPI, QSV,
  and fallback cases.
- Fail policy produces a typed unsupported result.
- Transcode profile identity changes when the pipeline changes.

Result: Complete.

## M3 - FFmpeg Adapter

Exit criteria:

- HLS command planning consumes the pipeline-derived policy.
- FFmpeg adapter owns command strings but not policy/fallback decisions.
- Playback HLS path keeps existing behavior.

Result: Complete.

## M4 - Diagnostics And Contracts

Exit criteria:

- Admin diagnostics summarize stage readiness and fallback evidence.
- Public Client API remains hardware-redacted.
- Generated SDKs are refreshed if Admin DTO shape changes.

Result: Complete. No SDK regeneration was required because the changed
diagnostics are Admin-only and generated/public contract gates pass.

## M5 - Closeout

Exit criteria:

- Focused gates pass.
- Workstream evidence is current.
- Follow-ons are documented.
- Lane status is completed.

Result: Complete.
