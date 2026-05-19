# Admin Web Console Milestones

Status: Completed
Last updated: 2026-05-19

## M-AWC.0 Planning Baseline

Objective:

- Establish the admin web console as a durable product/design workstream.
- Define page families, route families, brand direction, API implications, and
  non-goals.
- Provide a v0.dev-oriented context document.

Deliverables:

- `README.md`
- `DESIGN.md`
- `V0_CONTEXT.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

Exit criteria:

- The first admin console scope is distinguishable from playback-client work.
- The v0 context can be used without locking a front-end framework.
- The route inventory is product-oriented and not mistaken for a server API
  compatibility promise.

## M-AWC.1 Admin API Contract Baseline

Objective:

- Determine the admin API surface needed by the console.
- Keep admin diagnostics separate from the Public Client API.
- Define leakage and redaction rules for UI-facing admin DTOs.

Deliverables:

- Admin route matrix.
- Admin API namespace/versioning decision.
- ADR or design note if the boundary changes public/server contracts.

Exit criteria:

- Existing route coverage and missing route gaps are documented.
- Public Client API is not expanded with admin-only surfaces.
- Secret, token, raw path, raw provider body, and addon-hosted-page rules are
  explicit.

Status: completed for AWC-030. ADR 0027 accepts `/admin/v1/*` as the
admin-only route boundary, keeps admin DTOs in `taru-api`, keeps
`taru-client-protocol` public-client-only, and defines leakage/redaction rules
for future Admin API slices.

## M-AWC.1.1 Admin API v1 Overview Seam

Objective:

- Implement the first code-backed `/admin/v1/*` route without expanding the
  Public Client API.
- Give the future web console a small safe overview summary that composes
  existing diagnostics.

Deliverables:

- `GET /admin/v1/overview`.
- Admin-owned overview DTOs in `taru-api::admin`.
- Server route tests that prove the overview is read-only and redacted.
- Public OpenAPI and SDK leakage checks that reject admin route terms.

Exit criteria:

- The route reports server/API version, storage status, metadata provider
  status, runtime counters, and startup recovery counters.
- The response excludes secrets, tokens, unsafe local filesystem paths, raw
  provider bodies, and local transcode output paths.
- Existing root/public route behavior remains compatible.
- `taru-client-protocol` has no changes.

Status: completed for AWC-035 / M52. `GET /admin/v1/overview` now reports
safe storage, metadata-provider, runtime, and startup summaries through
admin-owned DTOs in `taru-api::admin`.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check -p taru-api --tests`
- `cargo nextest run -p taru-api --no-fail-fast`: 14 tests passed.
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast`: 5
  tests passed.
- `git diff --check`
- `git diff --name-only -- crates/taru-client-protocol`: no changed files.

## M-AWC.2 v0 Prototype Prompt

Objective:

- Turn the planning baseline into a practical prompt for generating the first
  admin console prototype.

Deliverables:

- Updated `V0_CONTEXT.md`.
- A concise v0 prompt derived from the context.
- Mock-data expectations for the first prototype.

Exit criteria:

- The prompt covers brand, navigation, routes, page families, and safety rules.
- The prompt avoids hard-coding a front-end framework unless separately chosen.
- Generated UI can be evaluated against Taru's product language.

Status: completed for AWC-040/AWC-050 / M53. `V0_CONTEXT.md` now records which
first prototype pages can reference live API data and which remain mock or
planned Admin API data. `HANDOFF.md` captures the concise v0.dev prompt.

Close-out validation:

- `git diff --check`

## M-AWC.3 Real Web App Follow-On

Objective:

- Start implementation only after the prototype and API boundary are accepted.

Deliverables:

- Future web app scaffold.
- Mock-data/API boundary.
- First real API integration slice.
- Browser verification plan.

Exit criteria:

- The app builds.
- First pages are either backed by realistic mocks or real Admin API.
- Sensitive information is redacted in UI, fixtures, logs, and diagnostics.

Status: completed for AWC-060 and AWC-070. `apps/admin-web` now contains the
first Vite/React/TypeScript scaffold, an explicit Admin API client/data-source
boundary, mock/planned data fixtures, and live-capable Admin API reads for
overview, jobs, playback, storage, config, events, and catalog governance.

AWC-060 validation:

- `npm install`
- `npm run check`
- `npm run test`: 4 tests passed.
- `npm run build`
- Playwright CLI smoke at `http://127.0.0.1:5174/`
- Desktop viewport 1440x1000: no horizontal overflow.
- Mobile viewport 390x844: no horizontal overflow.

Closeout:

- The admin web baseline is complete.
- Generated Admin API TypeScript contract work is split to
  `docs/workstreams/admin-api-typescript-contract/`.
- Deeper Jobs, Catalog Governance, Playback, and Settings workflows should wait
  until the contract strategy is settled.

AWC-070 validation:

- `npm run check`
- `npm run test`: 9 tests passed.
- `npm run build`
- `git diff --check`
- Playwright CLI smoke at `http://127.0.0.1:5174/`
- Desktop viewport 1440x1000: no horizontal overflow.
- Mobile viewport 390x844: no horizontal overflow.
