# Admin Web intake action plan

## Goal

Render the Media Library Scan intake action plan in Admin Web operator
readiness so a self-hosted operator can see which intake subsystem needs
attention without reverse-engineering backend counters.

## What I Already Know

- The Admin API now exposes
  `details.media_library_scan.intake_action_plan`.
- The action plan is read-only and contains stable components for library scan,
  source fingerprint hash, and watch-folder runtime evidence.
- `OperatorReadinessPage` already renders a Media Library Scan detail panel with
  configured libraries, library scan posture, source hash coverage, watch-folder
  coverage, reason code, source reason, and action route.
- The page uses generated Admin types, deterministic mock fallback data,
  localized copy, and existing `DataPanel`/`Badge`/fact-row patterns.
- Existing unrelated dirty files must not be touched, staged, reverted, or
  committed.

## Requirements

- Render the Media Library Scan intake action plan inside the existing Operator
  Readiness drilldown.
- Show all backend-provided action-plan components in a deterministic order from
  the response.
- For each component, show:
  - localized component label
  - localized readiness status
  - localized readiness reason
  - attention count
  - safe source reason when present
  - safe existing Admin action target when present
- Show a read-only marker from the backend `read_only` flag.
- Keep the UI read-only. Do not add scan, repair, scheduler, or job mutation
  controls.
- Preserve redaction boundaries: do not render raw paths, source locators,
  hashes, backend URLs, tokens, job input JSON, or summary JSON.
- Add focused route tests for English and zh-Hans rendering and redaction.

## Acceptance Criteria

- [ ] `/operator-readiness` renders an Intake action plan section under Media
  Library Scan details.
- [ ] The section shows `Library scan`, `Source fingerprint hash`, and
  `Watch folders` component rows from mock/live response data.
- [ ] The section shows the read-only marker and existing Admin action labels.
- [ ] Route tests assert unsafe strings are not rendered.
- [ ] zh-Hans route test covers the new section labels.
- [ ] `npm run check --prefix apps/admin-web` passes.
- [ ] Focused Admin Web tests for operator readiness pass.

## Definition of Done

- The UI uses existing Admin Web components and CSS patterns.
- No new frontend dependency is introduced.
- The implementation relies on generated Admin types and mock fallback data.
- Trellis task validates and is archived after completion.
- Work is committed and pushed with a Conventional Commit message.

## Technical Approach

Add a small `IntakeActionPlan` rendering helper inside
`OperatorReadinessPage.tsx` rather than introducing a new design system. Use
existing formatter helpers for status/reason/action/source reason. Add only the
missing component-label messages to the operator-readiness catalog.

## Decision (ADR-lite)

Context: The backend now owns intake action-plan prioritization. Admin Web
should display that plan directly instead of deriving action items from
individual counters.

Decision: Render the backend-provided plan as read-only component rows in the
existing Media Library Scan detail panel.

Consequences: The page stays simple and redaction-safe, while future backend
component additions can be displayed without duplicating priority rules in the
frontend.

## Out of Scope

- Adding mutation buttons or confirmation flows.
- Changing Admin API contracts or generated TypeScript contracts.
- Redesigning the Operator Readiness page layout.
- Adding a separate route for scan intake diagnostics.

## Technical Notes

- Relevant files:
  - `apps/admin-web/src/features/overview/OperatorReadinessPage.tsx`
  - `apps/admin-web/src/features/overview/operatorReadinessFormatters.ts`
  - `apps/admin-web/src/i18n/catalogs/operatorReadiness.ts`
  - `apps/admin-web/src/App.test.tsx`
- Required specs:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
