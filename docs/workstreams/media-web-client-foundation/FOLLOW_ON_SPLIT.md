# Media Web Client Foundation - Follow-On Split

Status: Proposed
Last updated: 2026-05-26

This file captures work intentionally left out of the Media Web foundation lane.
Each item should become its own bounded lane or issue before implementation.

## Recommended Order

1. Browser Playback Auth Transport
2. Credential And Session UX
3. Management Context Links
4. Library-Scoped Browse And Recently Added
5. Desktop Tauri Native Playback
6. Invitation Onboarding
7. Local-Media Recommendations

## Browser Playback Auth Transport

Problem: The generated SDK can attach bearer headers to `fetch`, but a normal
`<video src>` request cannot. Native HLS has similar header limits on some
platforms.

Scope:

- choose JavaScript HLS/MSE with headers, cookie/session playback auth, or
  short-lived playback tickets;
- preserve Library Access and playback-session policy;
- add the real browser media element only after transport is accepted;
- wire continuous progress writes to User Playback State.

Non-goals:

- native desktop hardware decode;
- mobile-native playback;
- privileged permanent stream URLs.

## Credential And Session UX

Problem: Media Web currently supports token entry and in-memory fixture/live
connection state. It has no Public Client current-principal, login, persistent
session, or profile contract.

Scope:

- Public Client current-principal/session summary;
- browser-safe login/session persistence if credentials are accepted;
- logout and account switching semantics;
- no public self-registration by default.

## Management Context Links

Problem: Jellyfin-style admin/media switching is useful, but links must be
permission-gated and must not leak Admin API state into Media Web.

Scope:

- Admin-to-Media links from library, item, scan, playback, and session contexts;
- Media-to-Admin links only when the principal has an admin role;
- stable safe IDs in route URLs;
- no shared privileged client state.

## Library-Scoped Browse And Recently Added

Problem: `/libraries/:libraryId` currently shows source evidence because Public
Client API lacks a first-class library item grid and Recently Added feed.

Scope:

- `GET /libraries/{library_id}/items` or typed `library_id` filter on
  `GET /items`;
- explicit sort keys including recently added;
- route-owned facets/pagination;
- server-side Library Access enforcement.

## Desktop Tauri Native Playback

Problem: Desktop should be able to reuse Media Web browsing UX, but serious
playback needs native capability for codec support, hardware decode, HDR,
subtitle handling, and audio routing.

Scope:

- decide whether `apps/admin-web` Media surface is embedded directly or split
  into reusable client modules later;
- Tauri shell and native playback core spike;
- hardware/software decode capability matrix;
- playback state synchronization through Public Client API.

## Invitation Onboarding

Problem: Private self-hosted media usually needs controlled onboarding, not open
registration.

Scope:

- invitation redemption contract;
- first-login flow;
- audit and expiry semantics;
- admin controls for invitations.

## Local-Media Recommendations

Problem: Recommendations are valuable but should come after local browse,
playback, and state are stable.

Scope:

- local library-based rails;
- continue watching and recently added improvements;
- no streaming storefront or online aggregation in this lane.
