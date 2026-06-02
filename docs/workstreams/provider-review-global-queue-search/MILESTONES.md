# Provider Review Global Queue Search - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Status: Complete after `PRGQ-010`.

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude Public Client API, raw provider payloads, batch governance,
  and related hierarchy application.

## M1 - Read-Only Admin API Global Queue

Status: Ready at `PRGQ-020`.

Exit criteria:

- repository query contract exposes global Candidate Review queue reads with
  stable pagination and filters;
- Admin DTOs expose redaction-safe queue rows;
- route tests prove no Provider Subject, Provider Mapping, Canonical Metadata,
  status, apply, or related hierarchy writes;
- generated Admin contract remains synchronized.

## M2 - Web Admin Global Queue Navigation

Status: Pending `PRGQ-020`.

Exit criteria:

- Web shows global Candidate Review rows with status, source, item, root
  summary, application action, and safe navigation to the existing detail/apply
  route;
- route-state and data-source tests prove filter/search transitions;
- type-check, tests, bundle, and browser smoke gates pass.

## M3 - Closeout And Follow-On Split

Status: Pending `PRGQ-030`.

Exit criteria:

- fresh evidence is recorded;
- batch governance and hierarchy application follow-ons remain split or
  explicitly deferred;
- architecture maps route no active work to a closed lane.
