# Admin Web Addon Operations Design

Status: Completed
Last updated: 2026-05-22

## Problem

Nako's backend Addon work is now deeper than the Admin Web Console surface.
The Admin API can manage **Addon Sidecars**, but `apps/admin-web` still treats
Addons as a planned area. This creates product friction:

- operators cannot see which Addons are enabled, disabled, or unregistered;
- **Addon Health Check** and resource-call diagnostics exist but are not
  visible in the console;
- manifest surfaces such as **Addon Entry Points**, **Addon Hosted Pages**,
  configuration schema metadata, **Addon Tasks**, and **Addon Event
  Subscriptions** are API-only;
- the frontend generated Admin API contract does not include Addon DTOs, so
  adding UI directly would reintroduce hand-written drift.

## Target State

The Admin Web Console has a live-capable Addons operations slice that lets an
administrator inspect and operate registered **Addon Sidecars** safely:

- list registered Addons with status, protocol version, grants, base URL host
  facts, and update time;
- inspect one selected Addon detail without exposing raw tokens or secret
  values;
- enable or disable an Addon through the existing Admin API;
- run an **Addon Health Check** and show redaction-safe status;
- view declared **Addon Entry Points**, **Addon Hosted Pages**, configuration
  schema metadata, secret-reference fields, **Addon Tasks**, and **Addon Event
  Subscriptions**;
- run a bounded resource-call diagnostic from declared resources and display
  safe status, attempts, HTTP status, and safe error code.

## Architecture Direction

Deepen the frontend Addon seam instead of sprinkling Addon fetch calls across
`App.tsx`.

The desired module shape is:

- generated Admin API TypeScript contract owns Addon wire DTOs and route
  constants;
- `AdminApiClient` owns HTTP method details and path construction;
- `dataSource.ts` maps live Addon DTOs into a UI-oriented `AddonOperations`
  read model with mock fallback;
- `App.tsx` renders the Addons panel from that read model and calls narrow
  data-source actions.

This keeps the Interface small for the UI and gives leverage: future Addon UI
detail pages can grow behind the same data-source seam.

## Safety Rules

- Never render token values, bearer headers, resolved secrets, raw payloads,
  local paths, storage URIs, Source Locators, cache URIs, database URLs, or
  raw provider responses.
- Treat **Addon Hosted Pages** as external and untrusted.
- Do not store admin bearer tokens in build-time Vite environment variables.
- Do not add Public Client API routes for admin Addon operations.
- Do not create a Docker/process-control Interface in this lane.

## Non-Goals

- No **Addon Manager** lifecycle automation.
- No Addon install/update/remove package flow.
- No marketplace.
- No Docker socket, systemd, or Kubernetes adapter.
- No Addon Protocol version bump unless existing generated contract tests
  reveal an unavoidable mismatch.
