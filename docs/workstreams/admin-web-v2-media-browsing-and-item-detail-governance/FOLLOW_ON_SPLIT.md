# Admin Web V2 Media Detail Follow-On Split

Status: Accepted
Date: 2026-05-25
Task: MBG-050

This note re-scores the item-scoped repair/action gaps after `/catalog` and
`/items/:itemId` landed. It intentionally does not add more behavior to the
media browsing/detail lane. The first browse/detail slice is complete when it
shows safe readiness and hands off unsafe or mutating workflows to bounded
follow-ons.

## Current Evidence

| Area | Current evidence | MBG-050 decision |
| --- | --- | --- |
| `/catalog` browse/search | Implemented in MBG-030 through explicit public-read bridges. | Keep read-only. Do not attach repair buttons to browse rows. |
| `/items/:itemId` detail | Implemented in MBG-040 through `getPublicItemDetailBridge` and bounded source probes. | Keep read-only. Use readiness rows and support links only. |
| Catalog governance queue | `admin-web-v2-catalog-governance-route` is complete and read-only. Closeout already split detail and repair workflows. | Open a dedicated catalog repair/action lane before adding accept, split, merge, or rematch controls. |
| Generated Artifact proposals | `admin-web-v2-automation-generated-artifacts-route` is complete and read-only. Generated Admin contract already includes review-plan and review routes. | Open a generated-artifact review/action lane before rendering accept/reject controls. |
| Item artwork gallery and selection | HTTP docs/server expose Admin artwork gallery, select, unpublish, and managed-artwork lifecycle routes. Admin Web generated contract currently does not expose item artwork route constants. | Open an item artwork selection lane that starts with generated contract coverage, then adds guarded UI. |
| NFO item status/apply | Library-level NFO import/export commands exist. Item-scoped NFO status/apply semantics are not a generated Admin item detail read model. | Open an NFO item status/action lane only after safe status DTOs and confirmation semantics are defined. |
| Metadata diagnostics and Provider Mapping | Public item metadata diagnostics routes exist outside the generated Admin Web contract and can expose raw provider/cache evidence. Catalog governance shows safe counts only. | Split safe Admin metadata/provider evidence before any Provider Mapping accept workflow. |
| Local Inference evidence | Catalog governance summarizes Local Inference issue state, but item-scoped evidence values are intentionally redacted/split. | Split safe Local Inference evidence into the catalog repair lane or a prerequisite read-model lane. |
| Playback support detail | Generated Admin API includes playback support evidence. `/items/:itemId` links to playback sessions instead of embedding playback diagnostics. | Split a support-detail route if operators need item/source scoped playback troubleshooting. Keep playback controls out. |

## Follow-On Lanes

### 1. `admin-web-v2-generated-artifact-review-actions`

Goal: turn the read-only `/automation/generated-artifacts` route into a
confirmation-based review workflow for one proposal at a time.

Scope:

- Use generated Admin routes for review plan and review command.
- Add a proposal detail/review route or modal with safe review plan evidence.
- Require explicit operator confirmation for accept/reject.
- Show audit/result summaries without prompt bodies, raw payloads, provider
  raw data, source locators, local paths, tokens, or artifact storage handles.
- Keep autonomous apply, bulk review, and cross-item repair out of scope.

Suggested gates:

- `cd apps/admin-web && npm run check`
- `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`
- `cd apps/admin-web && npm run build`
- `git diff --check`
- Browser smoke for generated artifact list and one review path.

Why first: backend and generated frontend contract surface already exist, and
the route has a completed read-only V2 base.

### 2. `admin-web-v2-item-artwork-selection`

Goal: add item-scoped artwork governance around existing Managed Artwork
gallery, selection, and unpublish semantics.

Scope:

- Generate or add Admin Web contract coverage for item artwork gallery,
  select, and unpublish routes.
- Add an item-scoped artwork view reachable from `/items/:itemId`.
- Render only safe candidate/artifact/selected artwork summaries and first-party
  image refs.
- Require confirmation for selecting/replacing/unpublishing artwork.
- Preserve Managed Artwork redaction: no `source_uri`, `storage_uri`,
  `managed-artwork://`, cache URIs, local paths, content hashes, or provider
  tokens.
- Keep fetch/re-ingest, artifact cleanup, missing-artifact repair, and provider
  search out of the first lane.

Suggested gates:

- `cd apps/admin-web && npm run generate:admin-api`
- `cd apps/admin-web && npm run check`
- `cd apps/admin-web && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`
- `cd apps/admin-web && npm run build`
- focused Admin artwork HTTP/API tests if generator changes expose DTO drift
- `git diff --check`
- Browser smoke for item detail and artwork selection route.

Why second: backend semantics exist, but Admin Web must first make the generated
contract and confirmation UX explicit.

### 3. `admin-web-v2-catalog-repair-actions`

Goal: turn the read-only catalog governance queue into bounded operator repair
workflows.

Scope:

- Start with a catalog governance item detail/read-model if needed.
- Define Provider Mapping accept/reject, unknown-item repair, duplicate/split,
  merge, and rematch semantics before UI controls.
- Show Local Inference and provider evidence only through redacted Admin DTOs.
- Require dry-run or review-plan style previews for every mutation.
- Require confirmation and audit/result summaries.
- Keep raw provider payloads, raw Local Inference evidence values, file paths,
  Source Locators, sidecar content, tokens, and arbitrary metadata write forms
  out of scope.

Suggested gates:

- backend read/mutation tests for repair semantics before frontend controls;
- `cd apps/admin-web && npm run check`
- focused route/data-source tests for each repair action;
- `cd apps/admin-web && npm run build`
- `git diff --check`
- Browser smoke for queue, detail, confirmation, and result states.

Why third: this is the highest product value, but it needs the strongest
semantics and redaction work before Admin Web can safely expose mutations.

### 4. `admin-web-v2-metadata-diagnostics-read-model`

Goal: add a safe Admin item metadata diagnostics read model that supports repair
decisions without exposing provider raw bodies.

Scope:

- Replace direct use of public `/items/{item_id}/metadata/*` diagnostics with
  generated Admin DTOs.
- Summarize attempts, candidates, provider status, confidence, error codes, and
  cache availability.
- Redact provider raw bodies, raw cache keys, request URLs, secrets, local paths,
  and arbitrary provider payload fields.
- Do not add metadata refresh/apply controls in the first diagnostics slice.

Suggested gates:

- backend DTO/redaction tests;
- generated Admin API contract sync;
- route/data-source tests;
- `git diff --check`.

Why separate: it can become a prerequisite for catalog repair without forcing
repair mutations into the same slice.

### 5. `admin-web-v2-item-nfo-status-actions`

Goal: provide item/source scoped NFO sidecar status and guarded NFO actions from
Admin Web.

Scope:

- Add or reuse safe Admin DTOs for item/source NFO sidecar status.
- Distinguish library-level import/export jobs from item-scoped actions.
- Require dry-run/review and confirmation before any sidecar write.
- Preserve NFO Round Trip, backup, VFS write policy, and job audit semantics.
- Redact sidecar paths, raw XML, backup paths, storage handles, Source Locators,
  and file contents.
- Keep bulk library NFO actions in the Media Library management route.

Suggested gates:

- focused NFO/storage/server tests for status/action semantics;
- Admin Web route/data-source tests;
- browser smoke for item NFO status and confirmation states;
- `git diff --check`.

Why separate: NFO sidecar mutation has storage and backup implications that are
larger than the item detail read slice.

### 6. `admin-web-v2-playback-support-detail`

Goal: add an operator troubleshooting view for playback support evidence linked
from item/source context.

Scope:

- Use generated Admin playback support evidence routes.
- Add route-owned filters for session/source/item where supported.
- Summarize decisions, transcode/runtime evidence, and safe failure taxonomy.
- Do not add playback controls, direct watch-state edits, user playback
  personalization, or transcode output links.
- Redact transcode output paths, Source Locators, tokens, remote credentials,
  local paths, and full FFmpeg command lines.

Suggested gates:

- Admin Web route/data-source tests;
- existing playback support API tests when DTOs change;
- `cd apps/admin-web && npm run build`;
- browser smoke for support evidence route.

Why later: MBG-040 already links to playback sessions, and playback support is
diagnostic rather than repair/action critical.

## Non-Item Admin Web Backlog

These are still part of the broader Admin Web V2 objective, but they are not
MBG-050 item-scoped repair/action follow-ons:

- settings mutation beyond the closed read-only System Settings route;
- user, role, permission, and Library Access management;
- full-site i18n expansion beyond the library-management localization boundary;
- Addon lifecycle breadth not already covered by completed Addon routes.

They should be opened as separate workstreams after the media browsing/detail
lane is closed or when product priority demands them.

## Recommended Next Lane

Open `admin-web-v2-generated-artifact-review-actions` next.

Rationale:

- The read-only route is already complete.
- Generated Admin API contract constants already include review-plan and review
  routes.
- The first workflow can be one proposal, one review plan, one explicit
  accept/reject confirmation, and one redacted result.
- This advances the user's requested Generated Artifact management parity
  without forcing catalog repair, artwork selection, NFO, or provider evidence
  semantics into the same lane.
