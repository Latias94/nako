# Addon Resource Search Product Flow - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Lane Open

Exit criteria:

- The lane states why diagnostic search is insufficient for product search.
- Non-goals preserve search/download/cloud-drive boundaries.

## M1 - Admin Contract

Exit criteria:

- Product-search request/response DTOs exist.
- Selection request/response DTOs exist.
- Route constants are generated for product search and selection.
- Contract tests prevent raw sensitive fields from entering the Admin contract.

## M2 - Search Session

Exit criteria:

- Nako can call a resource-search addon and return display-safe results.
- Raw links and passwords stay inside a host-owned transient session.
- Opaque IDs are stable enough for the operator's selection flow.

## M3 - Intake Selection

Exit criteria:

- A selected opaque link creates or replays a `resource_search_selection`
  acquisition intake candidate.
- The browser does not submit raw link URLs/passwords back to Nako.
- Downloader, link-check, and cloud-drive save remain absent.

## M4 - HTTP And Generated Contracts

Exit criteria:

- HTTP routes exist.
- Admin TypeScript contracts are refreshed.
- Focused server/API tests pass.

## M5 - Closeout

Exit criteria:

- Final evidence is recorded.
- Admin UI, official addon migration, link checking, downloader execution,
  cloud-drive save, and password persistence are split to follow-ons.
