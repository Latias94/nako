# Public Client Library Browse Query Contract - Evidence And Gates

Status: Active
Last updated: 2026-05-28

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
