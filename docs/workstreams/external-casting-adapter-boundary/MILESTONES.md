# External Casting Adapter Boundary Milestones

Status: Active
Last updated: 2026-05-27

## Milestone 1 - Boundary Opened

Exit criteria:

- ADR 0042 records the sidecar renderer adapter boundary.
- Workstream docs define target state, non-goals, task order, and gates.

Tasks:

- `ECAB-010`

## Milestone 2 - Current External Gap Locked

Exit criteria:

- External protocol renderer registration remains rejected from Public Client
  routes.
- Admin diagnostics truthfully report Nako remote-client transport as ready and
  Chromecast/DLNA/AirPlay as planned.
- Redaction expectations are covered.

Tasks:

- `ECAB-020`

## Milestone 3 - Host Adapter Contract Proven

Exit criteria:

- Host-side discovered renderer target and adapter command envelope contracts
  exist.
- Synthetic adapter tests prove policy, command, and cast-safe URL handoff.
- Denied policy creates no adapter side effects.

Tasks:

- `ECAB-030`
- `ECAB-040`

## Milestone 4 - First Real Protocol Chosen

Exit criteria:

- Chromecast and DLNA implementation options are evaluated.
- The first protocol and repository boundary are selected.
- Blockers for other protocols are documented.

Tasks:

- `ECAB-050`

## Milestone 5 - First Protocol Slice Lands Or Splits

Exit criteria:

- The chosen first protocol slice lands, or a narrower workstream is opened with
  concrete evidence for the split.
- Admin diagnostics remain redaction-safe.
- Final gates pass.

Tasks:

- `ECAB-060`
- `ECAB-070`
