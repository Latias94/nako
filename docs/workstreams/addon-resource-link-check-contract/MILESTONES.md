# Addon Resource Link Check Contract - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit criteria:

- Workstream exists.
- ADR 0050 is referenced.
- UI/download/cloud-drive/password persistence are out of scope.

Status: done.

## M1 - Protocol Vocabulary And DTOs

Exit criteria:

- `resource_link_check` wire resource exists.
- Dedicated read scope exists.
- v1 request/response schemas exist.
- DTOs round-trip through serde.
- Manifest validation enforces the dedicated scope.

Status: done.

## M2 - Client Helper

Exit criteria:

- Typed helper sends a `resource_link_check` envelope.
- Helper validates request schema before HTTP.
- Helper validates manifest resource and granted scope before HTTP.
- Helper validates response schema and typed payload after HTTP.

Status: done.

## M3 - Verification And Closeout

Exit criteria:

- Targeted protocol/client tests pass.
- Formatting and check gates pass.
- Server/product integration is explicitly deferred.
- Workstream is closed and committed.

Status: done pending commit.
