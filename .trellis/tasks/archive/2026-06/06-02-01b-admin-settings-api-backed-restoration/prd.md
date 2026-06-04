# Admin Settings API Backed Restoration

## Goal

Restore placeholder Admin settings surfaces with API-backed reads and writes
using existing Admin API, auth, generated contract, and Admin Web controlled
form patterns.

## Requirements

* Start from ADR 0027 and the existing Admin API/Web contract split.
* Add or reuse `nako-api` Admin DTOs and regenerate the Admin Web contract; do
  not edit generated contract output by hand.
* Add server routes under `/admin/v1/*` with admin principal checks, redacted
  errors, and version-header behavior matching existing route patterns.
* Use Admin Web route/search/form patterns from the current app: controlled
  fields, TanStack Query mutations, inline error/success, and no bearer token in
  Vite env.
* Keep changes focused on API-backed restoration of settings surfaces; do not
  mix provider review, playback, VFS, or release packaging behavior.

## Acceptance Criteria

* [ ] Admin settings data loads from the live Admin API when live mode is used.
* [ ] Settings mutations require admin auth and return the standard error
      envelope on failure.
* [ ] Admin Web tests cover form state, mutation behavior, and live/mock
      fallback where relevant.
* [ ] Backend tests cover auth/admin rejection and a successful route path.
* [ ] Generated Admin Web contract is refreshed through the repo command.

## Definition of Done

* Relevant Rust tests pass with nextest where scoped.
* Admin Web check/test/build pass for touched frontend surfaces.
* No generated file is hand-edited.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01b-admin-settings-api-restoration`

Suggested branch: `task/01b-admin-settings-api-restoration`

Conflict note: serialize with any other task that changes `nako-api` Admin DTOs,
the generated Admin Web contract, or shared Admin settings routes.
