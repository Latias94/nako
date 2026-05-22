# Admin Web Addon Credential and Grant Onboarding Design

Status: Completed
Last updated: 2026-05-22

## Problem

Admin Web can now register an Addon manifest as disabled, generate an Addon
Install Guide, and run Addon operations for already registered sidecars. The
remaining onboarding gap is credentials and authority:

- an operator needs a one-time Addon Token to configure the sidecar runtime;
- token rotation and revocation exist in the Admin API but are not productized;
- accepted Addon Grants exist but cannot be edited from Admin Web;
- enabling an Addon should be guided by an explicit readiness checklist so the
  operator understands health, token, grant, and lifecycle boundaries.

## Target State

Admin Web exposes a safe credential/grant onboarding slice for the selected
Addon:

- issue a new Addon Token with a label and display the raw token exactly once;
- rotate an active token and display the replacement raw token exactly once;
- revoke a token and update token status without showing raw values;
- replace accepted Addon Grants from a small explicit editor;
- show an enable readiness checklist covering registration status, health,
  active tokens, declared resource scope coverage, accepted grants, and external
  sidecar lifecycle ownership.

## Architecture Direction

Keep the existing Admin Web seam:

- generated Admin API TypeScript contract owns request/response DTOs;
- `AdminApiClient` owns route paths and HTTP verbs;
- `dataSource.ts` maps wire DTOs to a UI-oriented Addon Operations read model;
- `App.tsx` renders narrow actions and never stores raw token material outside
  immediate component state.

If the current generated Admin contract excludes one-time raw token DTOs for
redaction safety, add them deliberately and update the contract test to allow
`raw_token` only in the explicit one-time issue/rotation response types.

## Safety Rules

- Raw Addon Tokens may appear only in issue/rotation action results and UI
  notices that clearly say "copy now".
- Raw Addon Tokens must not be part of `load()` data, mock fallback, list
  responses, generated default fixtures, logs, docs examples, install guides,
  or persisted UI model.
- Admin bearer tokens, resolved Secret References, token hashes, provider
  secrets, Source Locators, storage URIs, local paths, and raw sidecar payloads
  must never be rendered.
- Token/grant actions must be Admin API only. Do not add Public Client API
  surfaces.
- Do not expand into Addon Manager lifecycle automation.

## Non-Goals

- No Docker/systemd/Kubernetes/SSH/host-agent process control.
- No automatic sidecar restart after token rotation.
- No secret manager integration.
- No arbitrary manifest URL discovery.
- No OAuth-first Addon authorization.
- No full role-based Admin Web permission model in this lane.

## Follow-Ons

- Secret Reference resolution/configuration UX.
- URL-based manifest discovery after SSRF/trust policy.
- Addon Manager planning only if Nako/Taru should own lifecycle automation.
