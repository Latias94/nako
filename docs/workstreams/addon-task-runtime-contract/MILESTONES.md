# Addon Task Runtime Contract - Milestones

Status: Completed
Last updated: 2026-05-23

## M0 - Boundary Frozen

Status: Done 2026-05-23.

Exit criteria:

- the task runtime problem is distinct from declaration routing;
- execution, progress, result, cancellation, and retry are the first slice;
- source catalog, package signing, and process supervision are explicit
  non-goals or follow-ons;
- the first executable task is identified.

## M1 - Run Model Frozen

Status: Done 2026-05-23.

Exit criteria:

- the first host-owned task run shape is documented;
- progress and result are represented as Nako-owned runtime data;
- cancellation and retry are observable without leaking secrets.

## M2 - Runtime Surface

Status: Done 2026-05-23.

Exit criteria:

- one Addon Task can be represented through a Nako-owned runtime surface;
- progress and result behavior are surfaced through Nako;
- state transitions are explicit and testable.

## M2.5 - Direct Sidecar Task-Path Dispatch

Status: Done 2026-05-23.

Exit criteria:

- direct dispatch uses the same host-owned Addon Task run model;
- Nako calls the declared task path with a task envelope for one target run;
- success, failure retry, and in-flight cancellation remain Nako-owned;
- source catalog, package signing, provider breadth, and process supervision
  remain split follow-ons.

## M3 - Closeout

Status: Done 2026-05-23.

Exit criteria:

- fresh gates prove the task runtime slice;
- docs describe the shipped execution boundary;
- signing, source catalog, provider breadth, and process supervision are split
  or deferred.
- authenticated outbound task dispatch credential management and official
  addon task-path smoke coverage are split follow-ons.
