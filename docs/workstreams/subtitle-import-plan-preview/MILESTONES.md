# Subtitle Import Plan Preview Milestones

Status: Complete
Last updated: 2026-05-28

## M0 - Lane Setup

Exit criteria:

- The lane rejects file writes and browser-provided raw provider payloads.

## M1 - Plan DTO And API Contract

Exit criteria:

- Admin API has typed subtitle import-plan request/response DTOs.
- TypeScript contract exposes the preview route and types.

## M2 - Host Preview Endpoint

Exit criteria:

- Preview endpoint derives safe plan facts from selected candidate and media
  source IDs.
- HTTP tests prove no raw subtitle/provider/locator/path fields leak.

## M3 - Closeout

Exit criteria:

- Fresh focused gates pass.
- Follow-ons for Library File Write apply and subtitle fact refresh are explicit.
