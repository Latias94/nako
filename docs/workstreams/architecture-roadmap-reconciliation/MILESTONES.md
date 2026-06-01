# Architecture Roadmap Reconciliation - Milestones

Status: Active
Last updated: 2026-06-01

## ARR-M0 - Lane Opened

Outcome: A durable planner workstream exists for reconciling architecture and
roadmap status.

Deliverables:

- workstream docs and `WORKSTREAM.json`;
- audit notes from the six sub-architecture reviews;
- task ledger and validation gates.

Exit criteria:

- JSON validation passes;
- the active task advances beyond `ARR-010`.

## ARR-M1 - Program Queue Reconciled

Outcome: top-level planning docs show a truthful active queue.

Deliverables:

- `docs/GOALS.md`
- `docs/ROADMAP.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Exit criteria:

- no top-level queue points at a closed implementation lane as active;
- completed recent Web/GAMA/MVP lanes are described as closed or completed.

## ARR-M2 - Capability Maps Reconciled

Outcome: architecture maps reflect shipped provider, artwork, playback policy,
storage health, cache/header, addon, and realtime evidence.

Deliverables:

- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/STORAGE_VFS.md` as needed

Exit criteria:

- missing evidence links are added for the high-risk audit findings;
- proposed lanes name real next-depth work instead of already-shipped MVPs.

## ARR-M3 - Historical Routing Hazards Repaired

Outcome: old handoff/TODO references that can misroute future work are fixed or
explicitly deferred.

Exit criteria:

- targeted stale-reference `rg` checks pass or are documented as accepted
  historical context;
- broad historical cleanup is not mixed with implementation work.

## ARR-M4 - Verified Planner State

Outcome: the repository has a clear next implementation queue.

Exit criteria:

- all docs gates pass;
- `HANDOFF.md` names the recommended next lane choices;
- this workstream is ready for closeout or a follow-on split.
