# User Playlists Contract And Web Slice - Evidence And Gates

Status: Active
Last updated: 2026-05-28

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
