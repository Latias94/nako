# Web Admin Acquisition Intake - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Set

Opening gate:

```bash
python -m json.tool docs/workstreams/web-admin-acquisition-intake/WORKSTREAM.json
git diff --check -- docs/workstreams/web-admin-acquisition-intake
```

Implementation gates:

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
| 2026-05-28 | WAAI-010 | Opened this lane from WDRP-030 after confirming the generated Admin contract exposes acquisition intake candidates and the old Admin Web V2 route is closed as prior art. | Passed. |
| 2026-05-29 | WAAI-020 | Audited generated Admin acquisition intake contracts; added `AdminApiClient.getAcquisitionIntakeCandidates`, `loadAcquisitionIntake`, an explicit fixture, query normalization, redacted candidate read-model mapping, and data-source contract tests. Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`, `npm --prefix web run check`, `python -m json.tool`, and `git diff --check`. | Passed. |
| 2026-05-29 | WAAI-030 | Implemented `/admin/acquisition/intake` in the new `web/` shell with Admin navigation, route-owned `library_id`, `state`, `source_kind`, `managed_import_artifact_id`, `limit`, and `offset` state, fixture/live data-source behavior, read-only pagination, and redaction-sensitive rendering. Route tests cover fixture rendering, live query serialization, Bearer authorization, URL writes, and absence of raw locator/prompt/token text. Browser smoke passed at desktop and `390x844` mobile, with screenshots at `target/waai-030-desktop.png` and `target/waai-030-mobile.png`. Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts`; `npm --prefix web run test -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx`; `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`. | Passed. |
