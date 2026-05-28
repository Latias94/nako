# Subtitle Search Candidate Selection Milestones

Status: Complete
Last updated: 2026-05-28

## M0 - Lane Setup

Exit criteria:

- Workstream docs define the host-owned selected subtitle reference boundary.
- Non-goals explicitly reject writes, download execution, and import apply.

## M1 - Typed Client Boundary

Exit criteria:

- Addon client has a typed subtitle helper.
- Missing `subtitle_read` grants, wrong request schema, wrong response schema,
  and invalid payload shape are covered by tests.

## M2 - Host Search And Selection API

Exit criteria:

- Admin endpoint searches subtitle providers and returns safe candidate cards.
- Admin endpoint records selected references by opaque id from a short-lived
  host session.
- Focused HTTP tests prove raw subtitle delivery data does not leak.

## M3 - Contract And Closeout

Exit criteria:

- TypeScript contract includes the new routes and DTOs.
- Fresh validation evidence is recorded.
- Remaining work is clearly split to import planning / Library File Write.
