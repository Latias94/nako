# Addon Event Delivery Generated Route Contract

## Goal

Make the existing Admin Addon Event Delivery control routes reachable through
the generated Admin API contract and a bounded Admin Web operator page. This
closes a hidden-route gap: the server already exposes Addon delivery attempts,
scheduler work, deliver, and replay commands for outbox events, but Admin Web
cannot call them through `NAKO_ADMIN_ROUTES`.

## What I Already Know

- The parent overnight campaign is comparing Nako with `repo-ref/jellyfin` and
  shipping independently verified fearless-refactor slices.
- Nako already has server handlers for:
  - `GET /admin/v1/events/{event_id}/addon-event-attempts`
  - `GET /admin/v1/events/{event_id}/addon-event-scheduler/work`
  - `POST /admin/v1/events/{event_id}/addon-events/deliver`
  - `POST /admin/v1/events/{event_id}/addon-events/replay`
- These four routes are currently explicit exclusions in
  `crates/nako-api/src/admin_contract.rs`.
- `crates/nako-api/src/extension.rs` already owns redaction-safe response DTOs
  for Addon Event Delivery attempts, scheduler work, dispatch, and replay.
- Admin Web currently has only a narrow dashboard event summary. It has no
  route-owned Events page and no generated routes for Addon Event Delivery
  operations.

## Reference-Code Boundary

- Jellyfin is used only for architecture and operator workflow comparison.
- Do not copy, translate, or import Jellyfin code, comments, tests, schemas, or
  assets.
- Jellyfin has broad scheduled-task and activity-log operator surfaces. Nako
  should not imitate those broadly in this slice. The Nako-native boundary is an
  outbox-event drilldown with Addon delivery visibility and explicit operator
  actions.

## Requirements

- Add generated Admin route keys for Addon Event Delivery:
  - `eventAddonDeliveryAttempts`
  - `eventAddonSchedulerWork`
  - `eventAddonDeliver`
  - `eventAddonReplay`
- Remove those four routes from `ADMIN_ROUTE_EXCLUSION_SUFFIXES`.
- Add the missing TypeScript DTOs to the generated Admin contract body:
  - `AddonEventDeliveryAttemptsResponse`
  - `AddonEventSchedulerWorkResponse`
  - `AddonEventDispatchResponse`
  - `AddonEventReplayResponse`
  - `ReplayAddonEventRequest`
  - nested Addon delivery attempt, dispatch event, and scheduler work types.
- Regenerate both Admin TypeScript contract copies:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Add typed Admin Web client methods for list attempts, scheduler work,
  deliver, and replay using generated routes and encoded `event_id`.
- Add route-local Admin Web data-source methods that:
  - safely map event list rows into route rows,
  - safely map Addon delivery attempts and scheduler work,
  - preserve mock fallback for reads,
  - reject mutation failures instead of fabricating success,
  - keep replay reason code as an explicit operator input.
- Add an Admin Web `/events` page with URL-owned pagination:
  - list event ID, kind, status, attempt counts, payload/error presence, and
    timestamps,
  - show selected event Addon delivery attempts and scheduler work,
  - deliver due Addon events only when data source is live,
  - replay an event only after explicit prepare/confirm and a reason code.
- Do not expose raw event payload, idempotency key, raw error, headers,
  request/response body, URLs, tokens, credentials, local paths, backend URLs,
  fingerprints, manifest raw payload, or Addon sidecar internals.

## Acceptance Criteria

- [ ] `nako-api` generated route inventory includes the four Addon Event
      Delivery route constants.
- [ ] Generated contract drift tests pass.
- [ ] Admin Web client tests cover generated routes, encoded `event_id`, query
      params, deliver POST body, and replay POST body.
- [ ] Admin Web data-source tests cover safe mapping, read fallback, live
      mutation failure behavior, and redaction.
- [ ] Events route tests cover pagination, zh-Hans copy, read fallback, disabled
      mock mutations, deliver, replay confirmation/reason code, and redaction.
- [ ] Focused Rust and Admin Web gates pass before commit.

## Definition Of Done

- Code and generated artifacts are updated.
- Task evidence records commands run and results.
- Relevant spec memory is updated if this establishes a reusable pattern.
- Commit only this slice with a Conventional Commit message.

## Out Of Scope

- Broad Jellyfin-style scheduled task UI.
- Live activity-log streaming or websocket notifications.
- New durable scheduler behavior.
- Exposing raw event payloads, Addon request/response payloads, sidecar URLs, or
  raw errors.
- Schema migrations.

## Technical Notes

- Server route evidence:
  `crates/nako-server/src/http/addons.rs`.
- API DTO evidence:
  `crates/nako-api/src/extension.rs`.
- Current event list DTO evidence:
  `crates/nako-api/src/admin/event.rs` and generated
  `AdminOutboxEventListResponse`.
- Admin Web should follow the verified Access Invitation and Addon Task Run
  route/data-source/test patterns for live-only mutations and safe projections.
