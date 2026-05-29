# User Playlists Contract And Web Slice - Closeout

Status: complete
Closed: 2026-05-29

## Outcome

Nako now has a server-owned **User Playlist** contract and first web slice.

Delivered:

- principal-scoped User Playlist records and ordered membership persistence;
- SQLite and PostgreSQL adapters plus repository/app-service coverage;
- current-user Public Client routes under `/users/me/playlists`;
- access-filtered playlist item responses and `item_count`;
- OpenAPI, generated TypeScript/Kotlin SDK entries, and Rust client methods;
- first `web/` playlist UI at `/media/my-list`;
- live Public Client data source with fixture fallback;
- route-owned `playlist` and `view` search state;
- data-source, route, route-state, bundle budget, and browser smoke evidence.

## Verification

Passed during the lane:

```bash
cargo nextest run -p nako-client-protocol playlist --no-fail-fast
cargo nextest run -p nako-api playlist --no-fail-fast
cargo nextest run -p nako-api sdk --no-fail-fast
cargo nextest run -p nako-server user_playlist --no-fail-fast
cargo nextest run -p nako-client user_playlist --no-fail-fast
cargo nextest run -p nako-client sdk_inventory --no-fail-fast
cargo nextest run -p nako-db playlist --no-fail-fast
NAKO_TEST_POSTGRES_URL=<ephemeral Docker PostgreSQL> cargo nextest run -p nako-db postgres_playback_runtime_contract_user_playlist_membership_is_principal_scoped_ordered_and_idempotent --run-ignored ignored-only --no-fail-fast
npm run check --prefix sdk/typescript
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
cargo fmt --all -- --check
python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json
git diff --check
```

Browser smoke passed for:

- desktop `/media/my-list`;
- mobile `390x844` `/media/my-list?playlist=fixture-favorites&view=list`.

## Review

Workstream compliance: no blocking findings.

Code quality: no blocking findings.

The shipped contract keeps User Playlist distinct from catalog Collection, HLS
transport playlists, User Playback State, canonical metadata, media source
locators, and Admin API internals.

## Follow-Ons

- Web playlist management UI: create, rename, delete, add item, remove item,
  and reorder controls.
- Shared/public playlists, invites, and collaboration.
- Smart playlists and recommendation-generated lists.
- Offline sync and conflict resolution.
- Playlist-aware mobile/Tauri surfaces after the web management UX stabilizes.

## Residual Risk

The first web slice is intentionally read-oriented. Mutation routes and SDK
methods exist, but browser product controls for create, rename, delete, add,
remove, and reorder should be designed in a new lane so permissions, optimistic
state, empty states, and conflict handling are not rushed into this contract
closeout.
