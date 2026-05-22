# Admin Web Addon Onboarding Design

Status: Completed
Last updated: 2026-05-22

## Problem

Taru can inspect and operate registered **Addon Sidecars**, and it can generate
an **Addon Install Guide** from a stored manifest snapshot. The missing product
step is onboarding: an administrator still needs to hand-craft Admin API calls
to register an Addon manifest before the rest of the Addon Operations Console
becomes useful.

This creates a gap between the architecture boundary and the user journey:

- the correct boundary says Taru does not install or start sidecars;
- the product still needs a guided way to tell Taru about a sidecar;
- registration must be safe even when the sidecar has not started yet;
- later actions such as token issuance, grant replacement, health checks, and
  enablement need a clear continuation point.

## Target State

Admin Web exposes a safe Addon onboarding flow:

1. Administrator pastes an Addon manifest JSON document.
2. Admin Web validates parseability locally and previews important manifest
   facts before submission.
3. Admin Web submits the manifest to `POST /admin/v1/addons` with
   `status: "disabled"` by default.
4. Taru server validates the manifest using the canonical addon protocol
   validator and stores a registration snapshot.
5. Admin Web shows the registration result and links the administrator into the
   existing Addon Operations and **Addon Install Guide** path.

The flow supports both real-world orders:

- register manifest first, then use the install guide to start the sidecar;
- start sidecar first, then paste or export its manifest and register it.

The registration step itself must not require sidecar network reachability.
Reachability belongs to the existing **Addon Health Check** step.

## Architecture Direction

Deepen the existing `apps/admin-web/src/adminApi` seam instead of putting raw
fetch logic in UI components.

Expected shape:

- generated Admin API TypeScript contract already owns `RegisterAddonRequest`
  and registration response DTOs;
- `AdminApiClient` exposes a typed `registerAddon` method;
- `dataSource.ts` exposes a UI-oriented onboarding action that parses and
  submits manifest JSON while preserving redaction rules;
- `App.tsx` renders a focused onboarding panel near Addon Operations;
- tests cover local JSON errors, server validation errors, disabled-by-default
  registration, and no sensitive output leakage.

## Safety Rules

- Registered Addons must default to `disabled` from this UI.
- The onboarding UI must not ask for or render raw Addon Tokens, admin bearer
  tokens, resolved secrets, local paths, storage URIs, Source Locators, cache
  URIs, database URLs, or raw resource-call payloads.
- Do not fetch arbitrary manifest URLs in this lane. URL-based discovery is a
  future workstream because it adds network, SSRF, trust, and timeout policy.
- Do not start, stop, install, update, remove, or supervise Addon Sidecars.
- Do not add Docker socket, systemd, Kubernetes, SSH, or host-agent control.
- Do not expose this flow through the Public Client API.

## Non-Goals

- No Addon Manager lifecycle automation.
- No marketplace, package catalog, package signing, or update flow.
- No URL fetch/discovery onboarding.
- No token issuance or grant editor in this lane unless needed to make the
  registration handoff coherent.
- No Addon Protocol version bump unless existing contract tests require it.

## Open Follow-Ons

- Addon token issuance and grant replacement UX.
- Fetch manifest from a running sidecar URL after SSRF and trust policy are
  designed.
- Reference addon packaging examples that produce a manifest and sidecar image.
