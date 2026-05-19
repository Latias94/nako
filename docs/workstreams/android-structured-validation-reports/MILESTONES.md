# Android Structured Validation Reports - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Contract Freeze

Exit criteria:

- Report shape goals and non-goals are documented.
- JSON output is scoped as an additive report adapter.

Evidence:

- `TODO.md` ASVR-010 complete
- `DESIGN.md`

## M1 - Smoke Regression JSON

Exit criteria:

- `Smoke-Regression.ps1` writes `report.json`.
- Generated smoke JSON parses and includes state results, evidence paths, and
  overall result.

Evidence:

- focused smoke regression command
- JSON parse command
- `TODO.md` ASVR-020 complete

## M2 - Local Validation JSON

Exit criteria:

- `Validate-AndroidLocal.ps1` writes `report.json`.
- Generated validation JSON parses and links delegated smoke report paths when
  smoke runs.

Evidence:

- local validation command
- JSON parse command
- `TODO.md` ASVR-030 complete

## M3 - Closeout

Exit criteria:

- Docs describe where to find structured reports.
- Workstream evidence records fresh command output.
- Follow-ons are split or explicitly deferred.

Evidence:

- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
