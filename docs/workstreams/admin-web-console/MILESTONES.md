# Admin Web Console Milestones

Status: Proposed
Last updated: 2026-05-17

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
