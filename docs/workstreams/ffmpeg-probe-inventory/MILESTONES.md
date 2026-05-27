# FFmpeg Probe Inventory - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- ADR 0046 exists and references this lane.
- Workstream docs exist with task ledger and gates.

Result: Complete.

## M1 - Probe Inventory Parser

Exit criteria:

- Parser returns normalized names for encoders, decoders, hwaccels, filters, and
  bitstream filters.
- Parser tests cover FFmpeg header lines and representative capability rows.
- Raw FFmpeg command output is not stored in public diagnostics.

Result: Complete.

## M2 - Stage Capability Mapping

Exit criteria:

- Current accelerator reports derive stage capabilities from inventory facts.
- Missing decoder/hwaccel/filter/bitstream-filter facts are visible as missing
  stage capabilities.
- Existing fallback and fail-policy behavior remains explicit.

Result: Complete.

## M3 - Detector Execution

Exit criteria:

- Startup detector runs the required list commands.
- Failure of any required probe command degrades to typed probe-error evidence.
- Smoke-probe and device-initialization evidence is preserved.

Result: Complete.

## M4 - Diagnostics And Closeout

Exit criteria:

- Admin diagnostics expose stage capability details.
- Public Client API remains hardware-redacted.
- Focused gates pass.
- Workstream evidence and handoff are current.

Result: Complete.
