# Backend Media Product Deepening Design

Status: Completed
Last updated: 2026-05-27

## Problem

Nako has strong backend pieces, but several product boundaries are still
shallower than the local-media target requires:

- Database migrations are already consolidated to baseline files, but those
  baselines still contain historical replay fragments and compatibility
  leftovers. With no production users, the correct baseline should directly
  describe the final schema.
- Users can log in with admin-provisioned local credentials, but there is no
  controlled invitation or registration contract for onboarding normal viewers.
- Playback routes expose direct/remux/HLS behavior, browser playback tickets,
  hardware diagnostics, and transcode sessions, but the durable domain still
  uses `TranscodeSessionRecord` as the public concept. That conflates a user
  playback attempt with one possible temporary FFmpeg artifact.
- Admin APIs expose many operations, but Media Web cannot ask "what management
  actions are available for this library/item/source/session?" without knowing
  Admin Web route internals.

The target is not a smaller patch. The target is a deeper backend product seam:
the server decides authority, session lifecycle, source selection, and
management affordances; clients decide how to render or play them.

## Target State

### Migration Baseline

`crates/nako-db/migrations/baseline.sql` and
`crates/nako-db/migrations/postgres/baseline.sql` describe the current schema
directly. They should not look like a replay of old numbered migrations.

The first hardening pass should:

- remove historical `-- From 00xx` sections where the final schema can be
  represented directly;
- remove duplicate create/drop/alter steps that only exist because migrations
  were concatenated;
- keep SQLite and PostgreSQL schema intent aligned;
- prove the schema still migrates cleanly and repository contract tests still
  pass.

### Identity And Invitation Onboarding

Local password login remains the session authority accepted in ADR 0037.
Public registration stays closed by default. The new onboarding model should be
invitation-based:

- administrators create one-time or limited-use invitations through Admin API;
- invitation records store token hashes, never raw token values;
- list/detail Admin API responses redact invitation secrets;
- public redemption creates the user, local credential, role assignments, and
  session atomically;
- disabled/expired/redeemed invitations cannot be used;
- future email delivery, recovery, OIDC, LDAP, passkeys, and profile UX stay
  outside this first backend contract.

### Playback Runtime

Nako should introduce **Playback Session** as the user/client playback attempt.
A **Playback Transcode** is one possible artifact attached to a Playback
Session, not the session itself.

The backend should keep FFmpeg as the server-side transcode/remux engine and
leave desktop/native decoding to mature engines such as mpv/libmpv,
GStreamer, platform media frameworks, or client-side libraries like WebCodecs
and libmedia. Nako's server boundary should express:

- selected Media Source or Source Variant;
- client capabilities and requested transport;
- playback mode: direct, remux, HLS/transcode, or future optimized version;
- authenticated principal and Library Access decision;
- ticket/session expiry and heartbeat;
- optional transcode session/artifact link;
- current connection/read-model state for Admin API.

Existing public transcode session routes may remain compatibility wrappers
while the deeper model lands. New Admin and Public Client contracts should
prefer Playback Session language.

### Management Context Links

Management Context Links are a discoverable, permission-gated bridge between
Media Web and Admin Web. They do not share privileged state and do not make
Admin API data part of viewer state.

For a library/item/source/session context, the backend should return safe
actions such as:

- scan this Media Library;
- refresh metadata for this Media Item;
- view related jobs;
- view playback support evidence for this Media Source or Playback Session;
- open runtime diagnostics;
- open library metadata profile;
- open access management when the user is administrator.

Each link should include stable IDs, HTTP method when it is a command, a target
surface, enabled/disabled state, reason, and required role/access summary.

## Architecture Direction

### Deep Modules

- `IdentityOnboarding` in `nako-server` owns invitation redemption orchestration
  and calls repository adapters for users, credentials, roles, sessions, and
  invitations.
- `PlaybackRuntime` should grow a `PlaybackSession` module that owns session
  lifecycle and only delegates transcode/remux execution to existing FFmpeg
  modules.
- `ManagementContextLinks` should be a server app module that reads current
  principal/access, source/item/library records, and route capability metadata
  to produce a small link set.

Keep pure records and repository traits in `nako-core` only when a second crate
needs them. Keep HTTP DTOs in `nako-api` for Admin API and in
`nako-client-protocol` only when they are genuine Public Client concepts.

### Reference Boundaries

Kyoo and Dim both support the direction of separating playback runtime state
from media file records and making running streams inspectable. Their code is
not imported. `libmedia` informs client capability vocabulary; it is not a
server dependency. `oximedia` informs pipeline layering; it is too broad to
pull into Nako's server runtime.

### Desktop Direction

Desktop should be allowed to reuse the web surface for browse/admin UX, but
serious playback should use a native playback core. The backend must therefore
expose capability-driven playback contracts and stable session state, not
hard-code browser-only assumptions into the Playback Runtime.

## Scope

In scope:

- database baseline cleanup;
- invitation registration backend;
- playback session domain/repository/API;
- admin/current-connection read models;
- Management Context Link DTOs and routes;
- focused tests and generated contract updates where needed.

Out of scope:

- frontend route/component implementation;
- open public registration;
- email/SMS invitation delivery;
- OAuth/OIDC/LDAP/passkeys;
- recommendation rails;
- native desktop shell implementation;
- mobile client implementation;
- wholesale media engine replacement.

## Risks

- Playback Session migration can create confusing compatibility if public
  routes keep using Transcode Session names forever. The workstream should add
  wrappers only as a transition, not as the new model.
- Invitation redemption must be atomic. A partially created user without
  credential/session is unacceptable.
- Management Context Links can leak admin capability if they return links only
  gated by frontend visibility. The backend must evaluate principal roles and
  Library Access before returning enabled actions.
- Baseline cleanup must not accidentally remove schema needed by completed
  lanes. Repository contract tests are required after edits.

## Closeout Result

This lane is complete as of BMPD-060. The backend now owns the product seams
needed before frontend/native clients compose the mature local-media
experience:

- database baseline files describe the current pre-production schema directly;
- invitation redemption is controlled, hashed, one-time, and transactional;
- Playback Session is the user/client playback attempt, with optional
  Transcode Session artifacts linked beneath it;
- Management Context Links are computed by server-side authority checks and do
  not expose Admin Web routes, raw locators, or storage paths.

Remaining work belongs in separate lanes. Frontend should consume these
contracts. Desktop should choose a native playback engine adapter later, not
push decoder ownership into Nako server. SSO, account recovery, email delivery,
recommendations, scoped manager job views, and richer player capability
negotiation are intentionally outside this closed lane.
