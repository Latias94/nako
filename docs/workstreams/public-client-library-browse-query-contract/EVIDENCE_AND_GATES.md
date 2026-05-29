# Public Client Library Browse Query Contract - Evidence And Gates

Status: Completed
Last updated: 2026-05-29

## Gate Set

```bash
python -m json.tool docs/workstreams/public-client-library-browse-query-contract/WORKSTREAM.json
git diff --check -- docs/workstreams/public-client-library-browse-query-contract
cargo nextest run -p nako-server catalog --no-fail-fast
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | PLBQ-010 | Opened this lane from WDRP-065 after WMLP-030/WMLP-060 kept library-scoped browse and stable sort/filter as missing Public Client contracts. | Passed. |
| 2026-05-29 | PLBQ-020 | Froze `GET /libraries/{library_id}/items`, `LibraryItemsQuery`, sort/filter/watch-state vocabulary, `LibraryItemsResponse`, access behavior, and SDK expectations in `CONTRACT.md`; added an HTTP API contract note. Validation: `python -m json.tool`, `git diff --check`, `cargo nextest run -p nako-client-protocol --no-fail-fast`, `cargo nextest run -p nako-api openapi --no-fail-fast`, and `cargo fmt --all -- --check`. | Passed. |
| 2026-05-29 | PLBQ-030 | Implemented `GET /libraries/{library_id}/items` in server/API/protocol/Rust client and regenerated TypeScript/Kotlin SDKs. Validation: `cargo nextest run -p nako-server catalog --no-fail-fast`; `cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast`; `cargo nextest run -p nako-api openapi --no-fail-fast`; `cargo nextest run -p nako-api typescript_sdk --no-fail-fast`; `cargo nextest run -p nako-api kotlin_sdk --no-fail-fast`; `cargo nextest run -p nako-client client_builds_library_items_query_contract --no-fail-fast`; `cargo nextest run -p nako-db sqlite_library_media_contract_preserves_library_scoped_source_identity --no-fail-fast`. | Passed. |
| 2026-05-29 | PLBQ-040 | Wired `web/` Public media data source and `/media/library` to `GET /libraries/{library_id}/items` for supported sort/watch-state combinations while preserving missing-contract readiness for unsupported filters. Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`. | Passed. |
| 2026-05-29 | PLBQ-050 | Closed the lane after confirming PLBQ-020 through PLBQ-040 target state, evidence, and follow-ons. Validation: `python -m json.tool docs/workstreams/public-client-library-browse-query-contract/WORKSTREAM.json`; `git diff --check -- docs/workstreams/public-client-library-browse-query-contract`; `cargo nextest run -p nako-server catalog --no-fail-fast`; `npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx`; `npm --prefix web run check`; `npm --prefix web run build:budget`. | Passed. |
