# Admin API TypeScript Contract

Status: Completed
Last updated: 2026-05-20

## Why This Lane Exists

`apps/admin-web` now has a real Admin API client boundary, but its DTOs are
hand-written. That is acceptable for the first scaffold, yet it becomes a
contract drift risk before the console grows filters, detail pages, mutations,
or richer diagnostics.

This lane owns the Admin API TypeScript contract strategy for the web console.
It is deliberately separate from the Public Client TypeScript SDK and from the
admin-web UI workstream.

## Relevant Authority

- ADRs:
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Existing docs:
  - `docs/workstreams/admin-api-typescript-contract/ADMIN_CONTRACT_INVENTORY.md`
  - `docs/workstreams/admin-web-console/ADMIN_API_MATRIX.md`
  - `docs/workstreams/admin-web-console/V0_CONTEXT.md`
  - `docs/api/HTTP_API.md`
- Existing code:
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-api/src/openapi.rs`
  - `crates/taru-api/src/sdk.rs`
  - `apps/admin-web/src/adminApi`
  - `sdk/typescript`

## Problem

The admin web app currently duplicates Admin API response shapes by hand.
Without a generated or explicitly synchronized contract, future Admin API
changes can compile in Rust while silently breaking the web console, or the
web console can normalize fields that no server route actually returns.

The public TypeScript SDK cannot solve this because ADR 0025 and ADR 0027
require Public Client API and Admin API surfaces to stay separate.

## Target State

When this lane closes:

- Admin API routes included in the first web console have a typed TypeScript
  contract derived from, or mechanically synchronized with, `taru-api`.
- The contract includes only accepted `/admin/v1/*` routes and the small
  shared runtime needed by admin-web.
- Public Client SDK generation continues to reject admin/internal surfaces.
- `taru-client-protocol` remains free of admin DTOs.
- `apps/admin-web` consumes the Admin API contract through a clear boundary,
  not through long-lived hand-written response interfaces.
- Redaction-sensitive fields remain absent from generated types, fixtures, and
  UI tests.

## In Scope

- Inventory the current hand-written Admin API DTOs in `apps/admin-web`.
- Decide the first contract artifact location and generation command.
- Add an Admin API TypeScript contract generator or sync check in the
  `taru-api` boundary.
- Wire admin-web to the generated or synchronized contract.
- Add tests that prove Public Client SDK and Admin API contract surfaces remain
  separate.
- Document generation, check, and redaction gates.

## Out Of Scope

- Publishing an Admin API npm package.
- Moving admin DTOs into `taru-client-protocol`.
- Combining Public Client and Admin API SDKs.
- Adding new Admin API routes solely for contract generation.
- Implementing admin-web filters, detail pages, mutations, or settings editing.
- Relaxing redaction rules for secrets, local paths, raw provider payloads,
  cache URIs, transcode output paths, or addon credentials.

## Architecture Direction

Keep ownership in `taru-api`, mirroring the public SDK generator location, but
generate a separate Admin API contract artifact. The first artifact should be
app-local unless a later packaging lane proves a reusable admin package is
needed. That keeps the implementation close to `apps/admin-web` while
preserving the rule that Public Client SDK artifacts never gain admin routes.

Prefer a contract shape that is cheap to regenerate and compile-check:

- explicit `/admin/v1/*` route inventory;
- generated response/request TypeScript interfaces from admin DTO schemas or
  an admin-only OpenAPI subset;
- generated route constants and query interfaces for the first read-model
  filters;
- no generated fetch runtime in the first slice;
- Rust tests for route inventory, leakage, and sync;
- TypeScript tests/checks for admin-web consumption.

## Closeout Condition

This lane can close when:

- the contract strategy is accepted and documented;
- the first generated or synchronized Admin API TypeScript artifact exists;
- admin-web no longer owns duplicated long-lived DTO definitions for the
  routes covered by AWC-070;
- public TypeScript SDK tests still reject admin routes;
- focused Rust and admin-web gates pass;
- and UI follow-ons for route filters/detail pages are split or queued.

Closeout result:

- AATC-010 through AATC-050 are complete.
- The app-local generated Admin API contract lives at
  `apps/admin-web/src/adminApi/generated/contract.ts`.
- Admin-web consumes generated wire/query/route types while keeping the
  hand-written fetch/runtime boundary.
- Public Client SDK and `taru-client-protocol` remain free of Admin API routes.
- Npm Admin SDK packaging and deeper Admin UI workflows are follow-ons, not
  active work inside this lane.
