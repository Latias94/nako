# Client Surface And Access Product Architecture

Status: Draft
Last updated: 2026-05-26

## Why This Lane Exists

Nako now has a capable Admin Web V2, a separated Admin API and Public Client
API, Single-Admin Mode, user-scoped playback state primitives, Android client
planning, and several Admin Web governance routes. The product boundary is no
longer only "can Nako serve media?" It is now "how should operators and viewers
move between media consumption and server management without collapsing both
experiences into one confused web console?"

Jellyfin and Plex are useful product references here. Both make it easy for an
administrator to move from a media-library problem into management tasks such
as scanning, metadata refresh, playback/transcode settings, connected sessions,
and task status. Nako should learn that workflow pattern while preserving
Nako's own API boundaries, redaction rules, and Admin Web governance language.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `DESIGN.md`
- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/workstreams/admin-web-v2-product-architecture/`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/`
- `docs/workstreams/admin-web-v2-users-access-readiness/`
- `docs/workstreams/admin-settings-configuration-authority/`
- `docs/workstreams/admin-web-v2-settings-mutation-authority/`
- `docs/workstreams/android-client-foundation/`
- `docs/workstreams/public-client-api/`
- `docs/workstreams/user-playback-state-contract/`
- `docs/workstreams/playback-transcode-ops-hardening/`
- `docs/workstreams/network-access-boundary/`
- `repo-ref/jellyfin`

## Problem

The next product decisions are currently spread across several lanes:

- Admin Web is deliberately administration-first, but it already exposes
  `/catalog`, `/items/:itemId`, `/libraries`, `/playback/sessions`, and
  `/settings`, so it is close enough to media browsing that future work could
  accidentally turn it into the main playback client.
- Public Client API supports media browsing and playback decisions, but Nako
  does not yet have a first-party Media Web surface for local media playback.
- Single-Admin Mode is truthful today, but users, roles, registration, login,
  sharing, and Library Access need an ordered path before family/small-group
  usage feels real.
- Operators need Jellyfin-like contextual escape hatches from viewing to
  management: rescan this library, refresh this item's metadata, review NFO or
  artwork, inspect playback failure evidence, adjust hardware acceleration
  settings, see current sessions, or inspect jobs.
- Desktop playback could reuse web UI through Tauri, but high-quality local
  media playback needs native decoder/player ownership rather than relying
  only on a system WebView `<video>` element.
- Mobile clients should remain native playback surfaces and should not inherit
  Admin Web settings, diagnostics, or metadata-management workflows.

## Target State

When this lane closes, Nako has an accepted product architecture for client
surfaces and access:

- Admin Web remains the operator console.
- Media Web becomes the browser-based local media browsing and playback
  surface, either under a separate app package or an explicitly separated route
  namespace.
- Admin Web and Media Web share domain terms, auth session context, public
  design primitives where useful, and safe deep links, but they do not share
  Admin API data models as consumer UI state.
- Users, Roles, and Library Access have a staged model that grows beyond
  Single-Admin Mode without faking unsupported account controls.
- Admin users can move from Media Web to management actions through
  permission-gated Management Context Links.
- Media users without admin rights never see admin links, Admin API data, raw
  paths, provider diagnostics, tokens, or settings controls.
- Desktop client strategy is split into a low-risk web wrapper and a real
  playback path: Tauri shell plus native playback core such as mpv/libmpv or a
  platform media backend.
- Mobile native remains the flagship playback direction for phones/tablets,
  with Media Web and desktop clients consuming the same Public Client API
  where possible.
- Recommendations, online media aggregation, and streaming-storefront
  discovery remain later features; first product breadth focuses on local
  media correctness, playback, library repair, and access.

## In Scope

- Product architecture for Admin Web, Media Web, desktop client, and mobile
  client boundaries.
- Account/auth staging: Single-Admin Mode, local account mode, invitation or
  admin-created users, later external identity.
- Role and permission vocabulary for administrator, library manager, viewer,
  and future restricted users.
- Library Access behavior needed for browsing, playback, source selection,
  Continue Watching, and admin context links.
- Management Context Links from Media Web into Admin Web.
- Admin Web links back into Media Web, such as "Open in Media Library" or
  "Play as current user" when safe.
- UX principles for switching surfaces without confusing mode, role, or data
  authority.
- Follow-on lane decomposition and validation gates.

## Out Of Scope

- Implementing user account persistence.
- Password hashing, registration pages, OAuth/OIDC, LDAP, passkeys, email,
  invitation token storage, or session-cookie infrastructure.
- Adding Public Client API routes in this lane.
- Adding Admin API routes or Admin Web code in this lane.
- Building `apps/media-web` or Tauri packaging in this lane.
- Mobile native implementation changes.
- Recommendation algorithms, global discovery, social sharing, comments,
  review systems, or streaming-provider aggregation.
- Copying Jellyfin or Plex UI/source/assets/object models.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Admin Web should not become the primary playback client. | High | `PRODUCT.md`, ADR 0026, and media-browsing governance closeout all state administration-first scope. | Reopen ADR 0026 and product context before adding watch-first Admin Web routes. |
| Media Web should consume Public Client API, not Admin API. | High | ADR 0027 keeps Admin API and Public Client API separate. | Add explicit public contract routes before Media Web depends on hidden admin DTOs. |
| Single-Admin Mode can remain the MVP access mode while preserving future User/Role/Library Access language. | High | `CONTEXT.md`, ADR 0028, and Users & Access readiness surface. | If multi-user becomes immediate, split account persistence before Media Web broadens. |
| Admin-to-media and media-to-admin links are product-critical. | High | Jellyfin/Plex-style flows let admins resolve problems at the point of discovery. | Media Web will feel disconnected from server operations and operators will jump manually. |
| Tauri plus WebView alone is insufficient for Nako's serious desktop playback target. | Medium | ADR 0026 rejects web-first flagship playback; desktop local media often requires broad codec/subtitle/hardware control. | If WebView proves enough for a constrained MVP, keep it as a compatibility tier, not the only desktop strategy. |
| Registration should not be open by default for self-hosted servers. | High | Nako is private self-hosted software with auth enabled by default. | Open registration requires abuse, invite, email, rate-limit, and remote exposure work first. |

## Architecture Direction

### Surface Model

Use separate product surfaces, even if they are served by one Nako server:

```text
Admin Web
  Operator console for server state, governance, settings, jobs, addons,
  storage, network, playback diagnostics, users, roles, and Library Access.

Media Web
  Browser-based client application for browsing, playback, personal state,
  source/version selection, Continue Watching, and local media discovery.

Desktop Client
  Tauri-hosted Media Web shell plus native playback core for robust codecs,
  subtitles, hardware acceleration, and desktop media integration.

Mobile Clients
  Native platform shells using Public Client API and native player stacks.
```

Admin Web may include light browse/detail views when they support governance.
Media Web may include admin links when the resolved principal has the required
role. Neither surface should pretend the other surface's data authority is its
own.

### Account And Access Staging

Nako should add user identity in stages:

1. **Single-Admin Mode**: current mode. One stable local principal,
   `local-admin`, has full access. Admin Web shows readiness, not fake CRUD.
2. **Local Account Mode**: admin-created local users with password or access
   token login. Registration is disabled by default. The first useful roles
   are `administrator`, `library_manager`, and `viewer`.
3. **Library Access Mode**: each user or role receives explicit Library Access.
   Public Client API browsing, playback decisions, Continue Watching, source
   variants, and Media Web visibility all respect Library Access.
4. **Invitation Mode**: optional invite links or admin-issued onboarding
   tokens for family/small trusted groups. Public self-registration remains
   off unless explicitly enabled by an operator.
5. **External Identity Mode**: OAuth/OIDC/LDAP/passkeys only after local
   accounts, roles, sessions, audit, and recovery paths are stable.

Public "open registration" should not be an early default. For a self-hosted
media server, the safe default is admin-created users or invitation-only
onboarding.

### Role Vocabulary

Start with coarse roles:

- `administrator`: full Admin Web and Media Web access.
- `library_manager`: can run library/media maintenance for assigned libraries,
  such as scan, metadata refresh, artwork/NFO review, and limited repair
  workflows.
- `viewer`: can browse and play assigned libraries, manage personal playback
  state, and use client settings.
- `restricted_viewer` or parental-control roles: later, only after content
  rating, tags, sharing, and policy semantics are designed.

Avoid field-level permission systems in the first account slice. Use coarse
roles plus Library Access, then deepen only where a workflow proves it needs
more precision.

### Management Context Links

Management Context Links are permission-gated bridges from Media Web to Admin
Web at the point where an administrator discovers a problem.

Examples:

- From a Media Library page: `Manage library`, `Scan now`, `Metadata profile`,
  `NFO policy`, `Jobs for this library`.
- From a Media Item detail page: `Open admin detail`, `Refresh metadata`,
  `Review artwork`, `Review NFO`, `Repair hierarchy`, `View source evidence`.
- From the Player or playback error sheet: `View playback support evidence`,
  `Current sessions`, `Playback runtime`, `Hardware acceleration settings`.
- From a source/version picker: `Inspect Media Source`, `Review variants`,
  `Playback Source Selection diagnostics`.
- From a server status indicator: `Network access`, `Connected sessions`,
  `Jobs`.

Rules:

- Links appear only when the authenticated principal has the required role.
- Links should keep context through stable IDs and safe query params, never raw
  Source Locators, local paths, secrets, provider payloads, FFmpeg command
  lines, or output paths.
- Admin Web should receive links as normal route URLs, not hidden cross-app
  state.
- When a management action is destructive or broad, Admin Web still owns the
  review plan and confirmation.
- Returning to Media Web should preserve the user's media context where
  possible.

### Admin Links Back To Media

Admin Web should offer consumer-context links when safe:

- `Open in Media Library` from `/items/:itemId`.
- `Open library in Media Web` from `/libraries/:libraryId`.
- `Play as current user` only when the admin principal also has Library Access
  and the action uses Public Client API playback flows.
- `Copy client-safe link` for a media item only after user/account sharing
  semantics exist.

Admin Web must not create privileged playback URLs that bypass Public Client
API auth, Library Access, Playback Source Selection, or playback-session
policy.

### Media Web First Slice

The first Media Web should be local-media-first:

- login/connect using the accepted access model;
- Libraries;
- Media Library detail;
- Search;
- Media Item detail;
- Source/Version Picker;
- Player;
- Continue Watching when User Playback State is available;
- Recently Added when backed by explicit Public Client API data.

Defer recommendations, online media discovery, and Plex-like streaming-store
features until local browse/play/repair/support flows are strong.

### Desktop Client Direction

Desktop should reuse Media Web UI, not Admin Web UI, for the playback
experience.

Recommended tiers:

1. **Browser Media Web**: baseline local web client using browser-supported
   playback and server remux/HLS/transcode.
2. **Tauri WebView Client**: packaged Media Web for convenience, server
   selection, updates, local integration, and simple playback.
3. **Tauri Native Playback Client**: Tauri shell plus native playback core
   such as mpv/libmpv or another platform media backend. This is the serious
   desktop target for broad codec support, subtitles, hardware acceleration,
   audio output, HDR, and local player diagnostics.

Tauri packaging should not be introduced into Admin Web as the default player
strategy. Admin Web can be packaged later for operator convenience, but the
desktop playback client should be Media Web-centered.

### UX Mode Switching

Mode switching must be explicit but not heavy:

- Use a surface switcher label such as `Media` / `Admin` for administrator
  principals.
- Preserve context across routes with IDs: library id, item id, session id,
  job id.
- Do not show admin-only navigation inside Media Web for non-admin users.
- Do not show consumption rails inside Admin Web except governance-supporting
  browse/detail summaries.
- Keep visual tone different enough to signal mode: Admin Web remains light,
  dense, and operational; Media Web may be artwork-led, darker, and playback
  first.

## Closeout Condition

This lane can close when:

- product docs record the accepted surface/access direction;
- a follow-on identity/account lane is split with a first backend contract;
- a follow-on Media Web foundation lane is split with first Public Client API
  gaps listed;
- a follow-on management-context-link lane is split for Admin Web/Media Web
  navigation contracts;
- a desktop Tauri playback spike is either split or explicitly deferred;
- docs/workstreams index and evidence docs are updated;
- no implementation claim is made without fresh validation.
