# User Playlists Contract And Web Slice - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Set

Opening gate:

```bash
python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json
git diff --check -- docs/workstreams/user-playlists-contract-and-web-slice
```

Backend/API gates:

```bash
cargo nextest run -p nako-db playlist --no-fail-fast
cargo nextest run -p nako-api playlist --no-fail-fast
cargo nextest run -p nako-server user_playlist --no-fail-fast
cargo fmt --all -- --check
```

Frontend gates:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts
npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

Closeout should also record browser smoke for desktop and mobile viewports.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | UPCW-010 | Opened this lane from WDRP-050 after confirming user principal, User Playback State, and Library Access prerequisites are present but no Public Client playlist contract exists. | Passed. |
| 2026-05-29 | UPCW-020 | Froze current-user private User Playlist route inventory, DTOs, access-filtering, ordering, duplicate membership, mutation, and SDK expectations in `CONTRACT.md`, `CONTRACT_READINESS.md`, `docs/api/HTTP_API.md`, protocol DTOs, OpenAPI schemas, and generated TypeScript/Kotlin SDK entries. Validation: `cargo nextest run -p nako-client-protocol playlist --no-fail-fast`; `cargo nextest run -p nako-api playlist --no-fail-fast`; `cargo nextest run -p nako-api sdk --no-fail-fast`; `cargo nextest run -p nako-client sdk_inventory --no-fail-fast`; `npm run check --prefix sdk/typescript`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json`; `git diff --check -- docs/workstreams/user-playlists-contract-and-web-slice docs/api/HTTP_API.md crates/nako-client-protocol crates/nako-api sdk/typescript sdk/kotlin`. | Passed. |
| 2026-05-29 | UPCW-030 | Implemented principal-scoped User Playlist records/repository trait, SQLite/PostgreSQL baseline schema and adapters, NakoDatabase facade forwarding, `UserPlaylistAppService`, and repository/app tests for idempotent membership, ordering/reorder, stale version conflicts, invalid names, media item existence, owner scope, and delete cascade. Validation: `cargo nextest run -p nako-db playlist --no-fail-fast`; `cargo nextest run -p nako-server user_playlist --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/user-playlists-contract-and-web-slice/WORKSTREAM.json`; `git diff --check -- docs/workstreams/user-playlists-contract-and-web-slice crates/nako-core crates/nako-db crates/nako-server/src/app`. | Passed. |
