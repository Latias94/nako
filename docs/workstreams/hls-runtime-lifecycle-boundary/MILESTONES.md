# HLS Runtime Lifecycle Boundary - Milestones

Status: Active
Last updated: 2026-05-31

## M0 - Lifecycle Invariant Freeze

Exit criteria:

- current lifecycle states and transitions are documented;
- existing test coverage and gaps are mapped;
- artifact I/O pressure and resource admission follow-ons are explicitly
  classified;
- implementation tasks have owned and shared scopes.

Status: Pending `HRLB-010`.

## M1 - Behavior-Preserving Lifecycle Boundary

Exit criteria:

- server HLS lifecycle tests prove the frozen invariants;
- route/app code has a clearer lifecycle boundary or an explicit reason to
  defer extraction;
- existing HLS behavior is not changed unless the task ledger is updated.

Status: Pending `HRLB-010`.

## M2 - Follow-On Split

Exit criteria:

- artifact I/O pressure, resource admission unification, remote workers,
  LL-HLS/CMAF, and player UX are either deferred or split into concrete
  workstreams;
- storage/VFS shared scope is explicit when artifact I/O pressure is selected.

Status: Pending `HRLB-020`.

## M3 - Closeout

Exit criteria:

- final gates pass with fresh evidence;
- docs and `WORKSTREAM.json` reflect active/completed/deferred status;
- no lifecycle ownership decision remains only in chat.

Status: Pending `HRLB-030`.
