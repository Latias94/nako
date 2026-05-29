# Subtitle Complete Chain Milestones

Status: Complete
Last updated: 2026-05-28

## M0 - Scope And ADR

Exit criteria:

- Workstream docs exist and agree.
- ADR 0051 records host-owned subtitle import and rejects direct addon writes.

## M1 - Shared Protocol Contract

Exit criteria:

- `nako-addon-protocol` owns subtitle request/response/candidate/delivery
  structs and schema constants.
- Official catalog facts reference shared schema constants.
- Protocol tests cover serialization and redaction boundaries.

## M2 - Official Provider Migration

Exit criteria:

- `nako-subtitle-provider` consumes shared protocol types.
- Provider remains read-only and fixture-backed.
- Checked-in manifest still matches runtime manifest.

## M3 - Host Follow-On Contract

Exit criteria:

- Candidate selection, import planning, Library File Write apply, refresh, and
  playback visibility are split into follow-on stages.
- No current task writes subtitle files.

## M4 - Closeout

Exit criteria:

- Focused protocol and official provider gates pass.
- Workspace formatting and diff hygiene pass in touched repos.
- Remaining subtitle complete-chain tasks are explicit follow-ons.
