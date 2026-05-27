# Playback Transcode Startup Degradation - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

Status: Complete

Exit criteria:

- ADR 0048 exists.
- Workstream docs exist with a task ledger and gates.

## M1 - Startup State Boundary

Status: Complete

Exit criteria:

- HLS startup stores readiness independent of executable plan.
- HLS service construction can succeed with unavailable transcode readiness.
- HLS execution still fails before FFmpeg spawn when no plan exists.

## M2 - Admin Diagnostics

Status: Complete

Exit criteria:

- Admin runtime diagnostics expose unavailable HLS readiness.
- Selected HLS slots are zero when HLS cannot execute.
- Selected fallback readiness does not claim ready for unavailable pipeline.

## M3 - Gates And Closeout

Status: Complete

Exit criteria:

- Focused gates pass.
- Evidence is current.
- Follow-ons are explicit.
