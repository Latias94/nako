# Android API Contract Integration - Milestones

Status: Active
Last updated: 2026-05-20

## M1 - Matrix Frozen

Status: Complete

Exit criteria:

- Android Public Client API route status is captured in
  `API_INTEGRATION_MATRIX.md`.
- First implementation slice is selected.

## M2 - Person Detail Contract

Status: Complete

Exit criteria:

- Android has typed client support for `GET /people/{person_id}`.
- Unit tests cover request, response, auth, version, and errors.

## M3 - Person Detail Productized

Status: Complete

Exit criteria:

- Person Detail has route state, navigation, data-source loading, and related
  Media Items.
- Dedicated Person Detail UI remains APICI-040 scope.

## M3.5 - Person Detail UI

Status: Complete

Exit criteria:

- Cast & Crew rows open the person route when a stable ID is present.
- Person Detail has a dedicated Material Expressive screen instead of reusing
  facet result rendering.

## M4 - Server-Backed Proof

Status: Complete

Exit criteria:

- Focused media smoke proves a real server-backed Person Detail path.
- Follow-on scope for People/Tags/Genres index pages is explicit.
