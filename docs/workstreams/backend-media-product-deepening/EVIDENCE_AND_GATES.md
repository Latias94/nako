# Backend Media Product Deepening - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Planned Gates

Run focused gates first, then broaden when a slice touches shared behavior.

### Documentation

```powershell
python -m json.tool docs/workstreams/backend-media-product-deepening/WORKSTREAM.json
git diff --check -- docs/workstreams/backend-media-product-deepening docs/workstreams/README.md
```

### Migration Baseline

```powershell
cargo nextest run -p nako-db --no-fail-fast
cargo nextest run -p nako-server -E 'test(admin_access) | test(local_session) | test(playback)' --no-fail-fast
```

### Invitation Registration

```powershell
cargo nextest run -p nako-db identity --no-fail-fast
cargo nextest run -p nako-server -E 'test(admin_access) | test(local_session) | test(register) | test(invitation)' --no-fail-fast
```

### Playback Session Runtime

```powershell
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast
cargo nextest run -p nako-db playback_session_tracks_user_attempt_independent_of_transcode --no-fail-fast
cargo nextest run -p nako-streaming --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Management Context Links

```powershell
cargo nextest run -p nako-server -E 'test(management_context) | test(admin_access) | test(playback)' --no-fail-fast
```

### Closeout

```powershell
cargo fmt --all -- --check
cargo nextest run -p nako-core --no-fail-fast
cargo nextest run -p nako-db --no-fail-fast
cargo nextest run -p nako-server --no-fail-fast
git diff --check
python -m json.tool docs/workstreams/backend-media-product-deepening/WORKSTREAM.json
```

## Evidence Log

- 2026-05-27: BMPD-010 opened the lane and recorded reference research from
  `repo-ref/jellyfin`, `repo-ref/kyoo`, `repo-ref/dim`, `repo-ref/oximedia`,
  and `repo-ref/libmedia`.
- 2026-05-27: BMPD-020 flattened SQLite/PostgreSQL baselines by folding
  historical add-column/index cleanup blocks into direct table/index
  definitions and adding regression tests that reject replay fragments.
  Verified:
  - `python -m json.tool docs/workstreams/backend-media-product-deepening/WORKSTREAM.json`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with Git line-ending warnings only.
  - `cargo nextest run -p nako-db --no-fail-fast` passed: 110 passed, 37
    skipped.
  - `cargo nextest run -p nako-server -E 'test(admin_access) | test(local_session) | test(playback)' --no-fail-fast`
    passed: 67 passed, 274 skipped.
- 2026-05-27: BMPD-030 added controlled invitation registration. Admin API can
  create/list/revoke invitations, public API can redeem an invitation into a
  local user/password/session, raw invitation tokens are stored only as hashes
  and list responses do not expose tokens, and redemption is transactional in
  the identity repository.
  Verified:
  - `cargo nextest run -p nako-db invitation_lifecycle --no-fail-fast` passed:
    1 passed, 148 skipped.
  - `cargo nextest run -p nako-server invitation --no-fail-fast` passed: 1
    passed, 341 skipped.
  - `cargo nextest run -p nako-db identity --no-fail-fast` passed: 4 passed,
    145 skipped.
  - `cargo nextest run -p nako-server -E 'test(admin_access) | test(local_session) | test(register) | test(invitation)' --no-fail-fast`
    passed: 9 passed, 333 skipped.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with Git line-ending warnings only.
- 2026-05-27: BMPD-040 introduced durable Playback Sessions as the
  user/client playback attempt boundary. Direct play now records sessions
  without fake transcode rows. Remux and HLS sessions link to optional
  Transcode Session artifacts. Public API and generated SDKs expose playback
  session get/cancel/heartbeat routes, while Admin playback lists report
  session state and linked artifact IDs without output paths or raw failure
  messages. The public route inventory was also corrected to include
  invitation redemption and playback heartbeat.
  Verified:
  - `cargo nextest run -p nako-client-protocol public --no-fail-fast` passed:
    10 passed.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
    passed: 19 passed, 39 skipped.
  - `python -m json.tool docs/workstreams/backend-media-product-deepening/WORKSTREAM.json`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with Git line-ending warnings only.
  - `cargo nextest run -p nako-db playback_session_tracks_user_attempt_independent_of_transcode --no-fail-fast`
    passed: 1 passed, 150 skipped.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 65
    passed, 277 skipped.
  - `cargo nextest run -p nako-streaming --no-fail-fast` passed: 10 passed.
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 35 passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with Git line-ending warnings only.
- 2026-05-27: BMPD-050 added backend-computed Management Context Links at
  `/management/context-links`. The app service resolves library/item/source/
  playback-session context, enforces browse visibility before returning safe
  IDs, and returns stable route names with enabled/disabled state and reasons
  for scan, metadata refresh, filtered jobs, playback support, playback
  runtime, metadata profile, and library access policy actions. Admin API
  routes now require administrator role, and item metadata refresh/diagnostic
  routes require Library Access Manage.
  Verified:
  - `cargo nextest run -p nako-server management_context --no-fail-fast`
    passed: 5 passed, 342 skipped.
  - `cargo nextest run -p nako-server -E 'test(management_context) | test(metadata_refresh_route_queues_background_job)' --no-fail-fast`
    passed: 6 passed, 341 skipped.
  - `cargo nextest run -p nako-server -E 'test(local_session_auth) | test(admin_v1_access) | test(bearer_auth)' --no-fail-fast`
    passed: 5 passed, 342 skipped.
  - `cargo nextest run -p nako-server -E 'test(management_context) | test(admin_access) | test(playback)' --no-fail-fast`
    passed: 70 passed, 277 skipped.
  - `cargo nextest run -p nako-server metadata --no-fail-fast` passed: 47
    passed, 300 skipped.
  - `cargo nextest run -p nako-client-protocol public --no-fail-fast`
    passed: 10 passed.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk)' --no-fail-fast`
    passed: 15 passed, 43 skipped.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
    passed: 19 passed, 39 skipped.
