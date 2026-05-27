# Old Admin Web Retirement Plan

Status: Active
Last updated: 2026-05-28

## Decision

Do not delete or archive `apps/admin-web` in this lane.

`web/` is the product frontend and Tauri shell line. `apps/admin-web` remains a
validation console until equivalent contract, route, redaction, mutation, and
smoke coverage exists in `web/` or lower-level CI gates.

The intended final state is deletion, not a long-lived source archive. Keeping
two frontends in the product tree would permanently tax dependency updates,
generated contract drift, visual QA, and route ownership. Historical reference
comes from Git history and this parity record, not from a copied archive.

## Maintenance Rules While Retained

Allowed changes:

- generated Admin API contract refreshes required by backend changes;
- test or fixture fixes required to keep validation gates meaningful;
- redaction, authentication, or unsafe-operation safety fixes;
- minimal build/dependency fixes required by repository-wide maintenance.

Avoid:

- broad product redesign;
- new viewer-facing product surfaces;
- new Admin workflows that are not needed to validate backend authority;
- importing v0, Vercel, Next.js, provider-secret, or third-party artwork
  assumptions.

## Parity Matrix

| Responsibility | `apps/admin-web` coverage | `web/` or CI coverage now | Retirement status | Gate before removal |
| --- | --- | --- | --- | --- |
| Admin TypeScript contract drift | Generated contract at `apps/admin-web/src/adminApi/generated/contract.ts`; Rust generator test compares it. | Generated contract at `web/src/api/admin/generated/contract.ts`; the same Rust test compares both copies. | Partial | Move drift authority to `web/` and update Rust tests so old generated output is no longer a required fixture. |
| Package validation | `npm run verify` covers generation, TypeScript, Vitest, and Vite build for the old console. | `npm --prefix web run verify` covers the new product frontend; Tauri build smoke now exists. | Partial | Keep both gates until every old validation responsibility below is ready or explicitly split. |
| Core Admin read routes | Overview, libraries, library detail, jobs, access, settings, Addons, playback sessions, storage staging, catalog, item detail, artwork, acquisition, generated artifacts, and governance routes. | First slice: `/admin`, `/admin/libraries`, `/admin/jobs`, `/admin/addons`, `/admin/settings`, with read-only fixture/live seams. | Partial | Add missing route families to `web/`, or split each family to a named follow-on and move its contract tests out of the old UI. |
| Admin mutations and confirmations | Settings raw-cache update, library metadata profile replacement, Addon token/grant actions, generated artifact review, catalog mapping review, artwork selection/unpublish, and job commands have focused tests. | `web/` Admin routes are intentionally read-only. | Blocking | Do not remove old UI until mutation authority is either implemented in `web/` with confirmation/redaction tests or covered by lower-level API/client tests. |
| Addon operations | Old console validates Addon registration detail, health, surfaces, install guide, credentials, token rotation/revoke, and grant replacement flows. | New `web/` only lists Addon registrations. | Blocking | Open Addon Manager product lane or move Addon action tests to non-UI contract/client gates. |
| Media browsing and playback | Old Media surface validates Public Client boundary, source selection URL state, playback decision preview, browser ticket safety, progress writes, pause flush, and watched-state marking. | New Media Web covers libraries, library detail, item detail, source selection, browser ticket request, and fixture/live ticket safety. | Partial | Add playback decision/progress/watched-state coverage to `web/` or split it to a Media playback depth lane. |
| Redaction corpus | Old tests inject local paths, source URIs, raw artwork paths, playback output paths, generated artifact paths, Addon secrets, and unsafe provider text across many routes. | New `web/` has focused token, Source Locator, local-path, and fixture-ticket safety tests. | Blocking | Build a shared redaction fixture corpus or equivalent `web/` route tests before old redaction tests disappear. |
| URL/query mapping | Old tests map many TanStack Router search params into generated Admin query DTOs. | New `web/` has limited Addons and Media source selection query coverage. | Partial | Cover query DTO mapping for every retained Admin route family in `web/` or lower-level client tests. |
| i18n validation | Old console has English and `zh-Hans` route copy tests. | New `web/` currently ships English-only product copy. | Open decision | Decide whether i18n is a release gate for `web/`; if yes, migrate i18n before old removal. |
| Visual/product shell | Old shell is operational and validation-oriented. | New `web/` owns product shell, Media/Admin split, setup, and Tauri route surface. | Ready | No old UI visual parity is required; product UX authority is already in `web/`. |
| Tauri desktop shell | None. | `web/src-tauri` owns shell packaging and connection bootstrap. | Ready | No old dependency. |

## Removal Checklist

Open a separate task before deleting `apps/admin-web`. That task must:

1. Mark every matrix row as `Ready`, `Moved to CI`, or `Split to follow-on`.
2. Run `npm --prefix apps/admin-web run verify` one last time to capture the
   final old-console baseline.
3. Run `npm --prefix web run verify`.
4. Run the Admin contract drift test after changing it to no longer require the
   old generated contract copy.
5. Run Browser/Playwright smoke for the replacement `web/` Admin and Media
   route families that took over old responsibilities.
6. Remove old app references from active package docs, scripts, and generated
   contract tests in the same deletion task.
7. Keep historical workstream references intact unless they are active
   instructions; historical docs may continue to mention `apps/admin-web` as
   past evidence.

Until that checklist is satisfied, `apps/admin-web` stays in place.
