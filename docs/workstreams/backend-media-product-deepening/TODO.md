# Backend Media Product Deepening - TODO

Status: Active
Last updated: 2026-05-27

## M0 - Workstream Open

- [x] BMPD-010 [owner=planner] [deps=none] [scope=docs/workstreams/backend-media-product-deepening]
  Goal: Open the backend product deepening lane with migration, identity,
  playback, and management-link scope.
  Validation: Workstream docs exist and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`; `TODO.md`; `WORKSTREAM.json`.
  Handoff: First executable task is BMPD-020.

## M1 - Migration Baseline Cleanup

- [x] BMPD-020 [owner=codex] [deps=BMPD-010] [scope=crates/nako-db/migrations,crates/nako-db/src]
  Goal: Rewrite SQLite/PostgreSQL baselines so they describe the final schema
  directly instead of replaying numbered migration history.
  Validation: `cargo nextest run -p nako-db --no-fail-fast`;
  `cargo nextest run -p nako-server -E 'test(admin_access) | test(local_session) | test(playback)' --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: check schema parity, identity tables, playback/transcode tables,
  addon partial indexes, and startup migration behavior.
  Evidence: Baselines no longer contain avoidable historical replay fragments
  or duplicate create/drop/alter blocks. Verified with `cargo nextest run -p
  nako-db --no-fail-fast` and focused `nako-server` identity/playback smoke.
  Handoff: BMPD-030 is next.

## M2 - Invitation Registration

- [x] BMPD-030 [owner=codex] [deps=BMPD-020] [scope=crates/nako-core/src/identity.rs,crates/nako-core/src/repository/identity.rs,crates/nako-db/src,crates/nako-api/src,crates/nako-client-protocol/src,crates/nako-server/src/app.rs,crates/nako-server/src/http/account.rs,crates/nako-server/src/http/admin.rs]
  Goal: Add controlled invitation-based registration and redemption while
  keeping public self-registration closed by default.
  Validation: focused `nako-db` identity contract tests; focused
  `nako-server` auth/admin HTTP tests; public route inventory tests.
  Review: token hashing, one-time redemption, expiry, atomic user/credential/
  role/session creation, redaction, and disabled invitation behavior.
  Evidence: Admin invitation create/list/revoke routes and Public Client
  invitation redemption route. Verified with focused `nako-db` identity tests,
  focused `nako-server` auth/invitation tests, `cargo fmt --all -- --check`,
  and `git diff --check`.
  Handoff: Email delivery, recovery, invitation delivery UI, and OIDC/LDAP stay
  follow-ons.

## M3 - Playback Session Runtime

- [x] BMPD-040 [owner=codex] [deps=BMPD-020] [scope=crates/nako-core/src,crates/nako-db/src,crates/nako-server/src/app/playback,crates/nako-server/src/http/playback.rs,crates/nako-api/src/admin,crates/nako-client-protocol/src]
  Goal: Introduce Playback Session as the durable user/client playback attempt
  and link optional Transcode Session artifacts to it.
  Validation: focused playback app/server tests for direct, remux, HLS, cancel,
  ticket, heartbeat/current-state, and redaction; `cargo nextest run -p
  nako-streaming --no-fail-fast`; `cargo nextest run -p nako-transcode
  --no-fail-fast`.
  Review: compatibility with existing transcode routes, no path/token leakage,
  direct-play sessions represented without fake transcode records, and client
  capability persistence.
  Evidence: New `PlaybackSession` ID/records/repository/schema/API DTOs track
  direct, remux, and HLS playback attempts separately from optional transcode
  artifacts. Public playback session get/cancel/heartbeat routes and Admin
  playback lists now use Playback Session language. Verified with focused
  `nako-client-protocol`, `nako-api`, `nako-db`, `nako-server`,
  `nako-streaming`, and `nako-transcode` gates.
  Handoff: Desktop native playback consumes the same session contract later;
  richer player capability negotiation can build on persisted client
  capabilities without making transcode artifacts user-facing.

## M4 - Management Context Links

- [ ] BMPD-050 [owner=codex] [deps=BMPD-030,BMPD-040] [scope=crates/nako-api/src,crates/nako-client-protocol/src,crates/nako-server/src/app,crates/nako-server/src/http]
  Goal: Add permission-gated context links from library/item/source/playback
  contexts to safe admin/media operations.
  Validation: focused HTTP tests for administrator, library manager, viewer,
  no-access, disabled user, and bootstrap admin cases.
  Review: enabled/disabled state, reasons, safe IDs, no admin data leakage,
  stable route names, and no frontend route coupling.
  Evidence: Public Client or Admin API context-link route returns scan,
  metadata refresh, jobs, playback diagnostics, runtime settings, and access
  links according to principal authority.
  Handoff: Frontend can render links later without hard-coding Admin internals.

## M5 - Closeout

- [ ] BMPD-060 [owner=codex] [deps=BMPD-050] [scope=docs/workstreams/backend-media-product-deepening,docs/workstreams/README.md]
  Goal: Verify the lane, record evidence, split remaining follow-ons, and close
  or continue the workstream truthfully.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-core
  --no-fail-fast`; `cargo nextest run -p nako-db --no-fail-fast`;
  `cargo nextest run -p nako-server --no-fail-fast`; `git diff --check`;
  `python -m json.tool docs/workstreams/backend-media-product-deepening/WORKSTREAM.json`.
  Review: close-workstream only after implementation evidence is current.
  Evidence: `EVIDENCE_AND_GATES.md`; `HANDOFF.md`; commits.
  Handoff: Split desktop native player, recommendations, OIDC/LDAP, and
  frontend UX lanes as needed.
