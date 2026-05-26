# Admin Web V2 Media Browsing And Item Detail Governance - Evidence And Gates

Status: Closed
Last updated: 2026-05-25

## Current Evidence

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | MBG-010 | Workstream opened from `admin-web-v2-library-management-and-localization` closeout and `PARITY_GAP_SPLIT.md`. | Pass. Scope, milestones, task ledger, and handoff created. |
| 2026-05-25 | MBG-020 | `ROUTE_API_READINESS.md` | Pass. Accepted public-read bridges for catalog browse/search and item detail; split metadata diagnostics, per-item Generated Artifacts, Admin artwork decisions, Provider Mapping, Local Inference, NFO status, and repair/apply actions. |
| 2026-05-25 | MBG-020 | `git diff --check` | Pass. No whitespace errors after route/API readiness documentation updates. |
| 2026-05-25 | MBG-030 | `/catalog` implementation in `apps/admin-web/src/features/catalog/CatalogBrowsePage.tsx`, `App.tsx`, and `adminApi` bridge/data-source files. | Pass. Added route-owned browse/search with `q`, `facet`, `limit`, `offset`; explicit public read bridges for `/items` and `/search`; safe summaries; deterministic fallback; stable links to reserved `/items/:itemId`. |
| 2026-05-25 | MBG-030 | `cd apps/admin-web && npm run check` | Pass. TypeScript route, bridge, data-source, and test types compile. |
| 2026-05-25 | MBG-030 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. 83 focused tests passed, covering URL params, bridge request paths, safe projection, fallback, redaction, and detail links. |
| 2026-05-25 | MBG-030 | `cd apps/admin-web && npm run test` | Pass. 85 Admin Web tests passed. |
| 2026-05-25 | MBG-030 | `cd apps/admin-web && npm run build` | Pass. Production build completed; existing Vite large chunk warning remains. |
| 2026-05-25 | MBG-030 | Browser smoke via Vite `http://127.0.0.1:4177` and headless Edge CDP for `/catalog` and `/items/item-unknown-1` at `1440x1000` and `390x844`. | Pass. Routes were nonblank, had no document horizontal overflow, no console errors, and no unsafe text matches for source locators, local paths, raw provider bodies, artifact handles, tokens, or secret-like values. `/items/:itemId` is a reserved placeholder for MBG-040, not the completed detail read model. |
| 2026-05-25 | MBG-030 | `review-workstream` self-review against MBG-030 scope, bridge policy, docs, and tests. | Pass. No blocking findings. Residual risk is intentionally deferred: public list/search DTOs do not expose source/image counts, so `/catalog` marks those facts as detail-route information until MBG-040. |
| 2026-05-25 | MBG-030 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | MBG-040 | `/items/:itemId` implementation in `apps/admin-web/src/features/items/ItemDetailPage.tsx`, `App.tsx`, and `adminApi` bridge/data-source files. | Pass. Added governance item detail facts, Canonical Metadata summary, safe Media Source filenames, bounded source probe summaries, public image readiness, split-workflow readiness placeholders, support links, deterministic fallback, and redaction-safe projection. |
| 2026-05-25 | MBG-040 | `cd apps/admin-web && npm run check` | Pass. TypeScript route, bridge, data-source, and test types compile. |
| 2026-05-25 | MBG-040 | `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts` | Pass. 87 focused tests passed, covering item detail route rendering, public item/detail bridge requests, source probe limit of three live item sources, fallback, and unsafe-field redaction. |
| 2026-05-25 | MBG-040 | `cd apps/admin-web && npm run test` | Pass. 89 Admin Web tests passed. |
| 2026-05-25 | MBG-040 | `cd apps/admin-web && npm run build` | Pass. Production build completed; existing Vite large chunk warning remains. |
| 2026-05-25 | MBG-040 | Browser smoke via Vite preview `http://127.0.0.1:4178` and `playwright-cli` Edge for `/items/item-unknown-1` and `/catalog` at `1440x1000` and `390x844`. | Pass. Routes were nonblank, headings resolved, had no document/body horizontal overflow, no console errors, and no unsafe text matches for source locators, local paths, raw provider bodies, artifact handles, playback output paths, tokens, or secret-like values. |
| 2026-05-25 | MBG-040 | `review-workstream` self-review against MBG-040 scope, bridge policy, docs, tests, and smoke evidence. | Pass. No blocking findings. Repair/apply mutations, metadata diagnostics/raw provider evidence, generated artifact item review, Admin artwork selection, Provider Mapping decisions, Local Inference evidence, and NFO item status remain explicitly split for MBG-050 follow-ons. |
| 2026-05-25 | MBG-040 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | MBG-050 | `FOLLOW_ON_SPLIT.md` | Pass. Re-scored repair/action gaps after `/catalog` and `/items/:itemId`; split Generated Artifact review/actions, item artwork selection, catalog repair/actions, safe metadata diagnostics, item NFO status/actions, and playback support detail into bounded follow-ons with scope, non-goals, gates, and recommended order. |
| 2026-05-25 | MBG-050 | `docs/workstreams/README.md` future splits update | Pass. Registered the planned Admin Web V2 follow-ons without opening broad or overlapping lanes. |
| 2026-05-25 | MBG-050 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |
| 2026-05-25 | MBG-060 | `review-workstream` closeout self-review against `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`, ADR 0027, and current git status. | Pass. No blocking findings. Target state is satisfied; remaining mutation/diagnostics breadth is split to follow-ons. |
| 2026-05-25 | MBG-060 | `cd apps/admin-web && npm run check` | Pass. Admin Web TypeScript project compiles. |
| 2026-05-25 | MBG-060 | `cd apps/admin-web && npm run test` | Pass. 89 Admin Web tests passed. |
| 2026-05-25 | MBG-060 | `cd apps/admin-web && npm run build` | Pass. Production build completed; existing Vite large chunk warning remains. |
| 2026-05-25 | MBG-060 | Browser smoke via Vite preview `http://127.0.0.1:4179` and `playwright-cli` Edge for `/items/item-unknown-1` and `/catalog` at `1440x1000` and `390x844`. | Pass. Routes were nonblank, headings resolved, had no document/body horizontal overflow, no console errors, and no unsafe text matches for source locators, local paths, raw provider bodies, artifact handles, playback output paths, tokens, or secret-like values. |
| 2026-05-25 | MBG-060 | `git diff --check` | Pass. No whitespace errors; Git reported existing LF-to-CRLF working-copy warnings only. |

## Gate Set

### Route Readiness Gate

```bash
git diff --check
```

Use after MBG-020 or planning-only follow-on split updates.

### Targeted Frontend Gate

```bash
cd apps/admin-web
npm run check
npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts
```

Use after browse/detail route, data-source, or bridge changes.

### Full Admin Web Gate

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

Run before closeout or after broad route/i18n changes.

### Browser Smoke Gate

Verify desktop `1440x1000` and mobile `390x844` for:

- `/catalog`
- `/items/:itemId`

Checks:

- nonblank route content;
- no document-level horizontal overflow;
- no console errors in mocked/fallback path;
- no unsafe rendered Source Locators, local paths, artifact storage handles,
  raw provider payloads, playback output paths, tokens, or secret-like values.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Use
`verify-rust-workstream` before completion claims.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/DESIGN.md`
- `docs/workstreams/admin-web-v2-media-browsing-and-item-detail-governance/TODO.md`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/features/catalog/`
- `apps/admin-web/src/features/items/`
- `apps/admin-web/src/adminApi/`

## Notes

Fresh verification is required before marking MBG tasks, this Codex goal, or
the workstream complete.
