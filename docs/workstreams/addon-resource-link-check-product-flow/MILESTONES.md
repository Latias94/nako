# Addon Resource Link Check Product Flow - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Status: done.

Exit criteria:

- Workstream exists.
- Admin UI, downloader, cloud-drive transfer, and password persistence are out
  of scope.

## M1 - API And Server Flow

Status: done.

Exit criteria:

- Product route exists.
- Request body has no raw URL/password/context fields.
- Host reads selected link from session store.
- Addon receives `resource_link_check` envelope.
- Product response is safe.

## M2 - Contract And Docs

Status: done.

Exit criteria:

- Static Admin contract includes the route and DTOs.
- Workstream evidence records behavior.

## M3 - Verification And Closeout

Status: done.

Exit criteria:

- Targeted server/API tests pass.
- Formatting/check gates pass.
- Workstream is closed and committed.
