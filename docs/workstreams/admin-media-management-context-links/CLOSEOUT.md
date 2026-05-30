# Admin Media Management Context Links Closeout

Status: Closed
Closed: 2026-05-30
Task: AMCL-090

## Closeout Claim

This lane is complete for the current `web/` product frontend. Management
Context Links are consumed through the Public Client boundary, resolved through
one frontend route mapper, rendered in Media contexts from backend-computed
state, and handed off to Admin-owned command or confirmation flows without
letting Media Web call Admin mutations directly.

The lane does not claim broader Admin Web or Media Web product expansion. It
also does not claim desktop/native playback strategy, scoped manager job views,
or the Generated Artifact Metadata Authority apply workflow.

## Delivered

- Public Client Management Context Link data-source boundary in `web/src/api`.
- Explicit `route_name` resolver in the `web/` shell.
- Media detail, library, selected source, and playback diagnostic link
  rendering from backend-computed state.
- Admin route state for library scan, item metadata refresh, jobs, playback
  support/runtime, access-policy targets, and safe Media return links.
- Admin-owned confirmation/mutation entrypoints for broad or mutating actions.
- Media/Public import guard coverage preventing Admin API or mutation-client
  dependencies.
- Cross-surface browser smoke covering Media-to-Admin links, Admin-to-Media
  return links, disabled states, and unsafe `source_id` redaction.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- `TODO.md` tasks AMCL-010 through AMCL-090 are complete.
- `DESIGN.md` target state is satisfied for the focused Management Context
  Link scope.
- ADR 0027 is respected: Admin-owned operations stay on the Admin boundary and
  Media/Public code does not import Admin API DTOs or mutation clients.
- ADRs 0024, 0028, and 0037 are respected: link state remains
  permission-gated by backend-resolved principals rather than frontend role
  inference.

### Code Quality

- Blocking: none.
- Important: none.
- AMCL-090 changed documentation only.
- Earlier implementation evidence shows route behavior through data-source,
  route-state, and browser seams rather than internal-only tests.
- The aggregate bundle ceiling increase made in AMCL-040 is documented with
  measured output and route-level budgets left unchanged.

### Missing Gates

- None for the shipped frontend/docs scope.
- AMCL-050 recorded fresh `npm --prefix web run test`, `npm --prefix web run
  check`, `npm --prefix web run build:budget`, import guard, and browser smoke
  evidence.
- AMCL-090 is docs-only, so closeout validation is limited to JSON and diff
  hygiene unless a reviewer requests a full web rerun.

## Follow-Ons

- Desktop/native playback strategy remains in CSAPA-050 or a dedicated desktop
  playback spike.
- Role-specific UX polish or scoped manager job views should be split into a
  bounded `web-product` workstream.
- Generated Artifact Metadata Authority apply UI remains separate under the
  GAMA lane after the final Admin apply route is stable.
- Future Management Context Link route breadth should be split when backend
  route names or permission states change.

## Evidence Anchors

- `docs/workstreams/admin-media-management-context-links/EVIDENCE_AND_GATES.md`
- `docs/workstreams/admin-media-management-context-links/JOURNAL/2026-05-30-amcl-050.md`
- `docs/workstreams/admin-media-management-context-links/TODO.md`
- `docs/workstreams/admin-media-management-context-links/HANDOFF.md`
- `target/amcl050-media-detail.png`
- `target/amcl050-admin-refresh.png`
- `target/amcl050-admin-return-links.png`
- `target/amcl050-media-library.png`
- `target/amcl050-library-scan-admin.png`
