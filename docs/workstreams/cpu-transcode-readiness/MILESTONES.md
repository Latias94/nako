# CPU Transcode Readiness - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

Status: Complete

Exit criteria:

- ADR 0047 exists.
- Workstream docs exist with task ledger and gates.

## M1 - CPU Capability Mapping

Status: Complete

Exit criteria:

- Probe-derived CPU capability records required software encode stages.
- Missing `libx264` or `aac` makes CPU unavailable.
- Admin stage capabilities can explain which software encoder is missing.

## M2 - Pipeline Fallback Semantics

Status: Complete

Exit criteria:

- Explicit CPU planning rejects unavailable CPU pipeline.
- Hardware fallback-to-CPU rejects when CPU is unavailable.
- Readiness reasons are typed and mapped to Admin diagnostics.

## M3 - Tests And Closeout

Status: Complete

Exit criteria:

- Focused gates pass.
- Evidence is current.
- Follow-ons are documented.
