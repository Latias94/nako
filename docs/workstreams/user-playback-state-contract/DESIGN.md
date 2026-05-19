# User Playback State Contract

Status: Active
Last updated: 2026-05-19

## Why This Lane Exists

Android now has honest device-local resume, but Taru still lacks a public
server-authoritative **User Playback State** contract. Without that contract,
the app cannot correctly claim cross-device Continue Watching, watched state,
favorites, hidden state, user rating, or last-played ordering.

This lane exists to define and then implement the first server/client contract
for user-scoped playback state without collapsing it into **Canonical
Metadata**, **Media Technical Facts**, or local Android storage.

## Relevant Authority

- Domain glossary:
  - `CONTEXT.md`: **User Playback State**, **User Library State**, and
    **Library Item State** definitions.
- Existing docs:
  - `docs/api/HTTP_API.md`
  - `docs/workstreams/user-playback-state-contract/CONTRACT.md`
  - `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`
- Architecture decisions:
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
  - `docs/adr/0024-inbound-token-authentication-boundary.md`
  - `docs/adr/0025-openapi-public-client-sdk-contract.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Related workstreams:
  - `docs/workstreams/android-client-foundation/`
  - `docs/workstreams/android-device-local-playback-position/`
  - `docs/workstreams/android-public-client-api-coverage/`
  - `docs/workstreams/public-client-api/`
  - `docs/workstreams/client-sdk-contract/`

## Problem

The current Public Client API exposes playback decision, stream targets, and
playback session inspection, but it does not expose:

- user-scoped resume lookup;
- progress reporting;
- watched/unwatched transitions;
- last played timestamps;
- favorite/hidden/user-rating state;
- Continue Watching or resume rail inputs.

Android correctly labels its current state as device-local, but that means a
resume point on one installation is invisible on another device and cannot
drive server-side browse ordering. The server also currently uses bearer-token
authentication rather than a durable first-class user identity model, so the
first contract must explicitly choose how **Single-Admin Mode** maps to a user
principal without making the domain permanently single-user.

## Target State

When this lane closes:

- Public Client API has explicit **User Playback State** routes and DTOs.
- Server storage persists state by user principal and Media Item, with Source
  identity included where source-specific resume matters.
- Android can read authoritative resume state and report playback progress
  through public routes.
- Continue Watching UI appears only when backed by server-authoritative state.
- Existing Android device-local resume remains a fallback/cache, not the
  authoritative cross-device state.
- Rust SDK, TypeScript SDK, API docs, and smoke/local validation agree on the
  shipped contract.

## UPS-010 Contract Freeze

UPS-010 freezes the first public route and DTO contract in
`CONTRACT.md`. The implementation target is deliberately narrow:

- current-user routes under `/users/me/playback-state/...`;
- lookup state for one Media Item;
- Continue Watching for the resolved principal;
- progress reporting;
- explicit watched/unwatched transitions;
- server-owned watched threshold policy;
- **Single-Admin Mode** principal resolution to the stable internal
  `local-admin` principal.

Favorites, hidden state, and user rating remain **User Playback State** domain
concepts, but first-slice routes intentionally defer them. They must not be
modeled as global Media Item metadata.

## In Scope

- Contract design for **User Playback State** public routes.
- User principal strategy for **Single-Admin Mode** that can evolve to multiple
  users.
- Server repository traits, SQLite schema, migrations, and app-service logic.
- Public API DTOs, OpenAPI/SDK generation, Rust SDK and TypeScript SDK surface.
- Android client DTO/client methods and UI integration for authoritative resume
  and progress reporting.
- Validation fixtures and smoke evidence for authoritative Continue Watching.

## Out Of Scope

- Full multi-user account management UI.
- OAuth, external identity providers, or household profiles.
- Offline sync conflict resolution across disconnected clients.
- Recommendations unrelated to playback state.
- Downloads/offline playback.
- Audio/subtitle/chapter track selection.
- Admin-only diagnostics beyond what is needed to test the public contract.
- First-slice routes for favorites, hidden state, or user rating.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| **User Playback State** must remain distinct from **Canonical Metadata** and **Media Technical Facts**. | High | `CONTEXT.md` explicitly separates these terms. | Data model would pollute shared item facts with per-user state. |
| Android's existing device-local resume is useful as fallback but cannot back Continue Watching claims. | High | `android-device-local-playback-position` closed with server state out of scope. | The app may falsely present cross-device behavior. |
| Single-Admin Mode can start with a stable internal user principal. | High | ADR-0028 freezes `local-admin` as the first stable internal principal. | A later account system may need migration or principal remapping. |
| Progress reporting should be idempotent and tolerant of frequent player ticks. | High | Mobile playback emits many position updates and may background/exit abruptly. | Naive writes could overload storage or regress playback. |
| Watched thresholds need a server-owned policy. | Medium | Clients differ in duration knowledge and exit behavior. | Client-only thresholds will diverge across Android, web, and SDK users. |

## Architecture Direction

Model **User Playback State** as a server-owned user-scoped read/write contract.
The domain record should belong in `taru-core`, storage in `taru-db`, mapping
and DTOs in `taru-api`/`taru-client-protocol`, route orchestration in
`taru-server`, and Android consumption under the existing public client layer.

The first slice defines a stable principal even in **Single-Admin Mode**. The
HTTP auth layer resolves every accepted admin bearer token to the internal
`local-admin` principal before playback-state services run. The principal must
be explicit in storage and service boundaries. Bearer token values must never be
stored as user ids. This keeps the data model ready for later user accounts
without rewriting every playback-state row.

Route design should separate three jobs:

- lookup state for one Media Item or a page of Continue Watching candidates;
- report progress for one Media Item/Source after playback ticks or exit;
- explicitly mark watched/unwatched when the user or policy decides.

Progress writes should not require clients to send raw storage locators, local
paths, or playback-session internals. Clients should report safe IDs, position,
duration when known, source id when relevant, and a client timestamp. The server
owns watched-threshold policy and normalizes non-positive or completed
positions.

## Closeout Condition

This lane can close when:

- the contract is implemented through server, public API, SDKs, and Android;
- Android smoke evidence shows Continue Watching backed by server state;
- device-local resume remains clearly scoped as fallback/local cache;
- all relevant unit, API, SDK, and Android gates pass;
- docs and workstream evidence reflect the shipped behavior and follow-ons.
