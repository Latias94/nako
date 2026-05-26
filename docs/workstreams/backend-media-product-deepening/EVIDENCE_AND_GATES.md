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
