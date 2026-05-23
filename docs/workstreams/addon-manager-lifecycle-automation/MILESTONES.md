# Addon Manager Lifecycle Automation - Milestones

Status: Active
Last updated: 2026-05-23

## M0 - Boundary Frozen

Exit criteria:

- the manager problem is distinct from Addon runtime/distribution;
- marketplace, package signing, and provider breadth are explicit non-goals;
- the first executable task is identified.

## M1 - Source Shape Frozen

Exit criteria:

- the first managed addon source shape is documented;
- install/update/remove intent is represented as a Nako-owned plan;
- operator-facing lifecycle state is observable without leaking secrets.

## M2 - First Lifecycle Slot

Exit criteria:

- one addon can be managed through a Nako-owned lifecycle slot;
- process state and logs are supervised through Nako;
- update/remove behavior is explicit and testable.

## M3 - Closeout

Exit criteria:

- fresh gates prove the manager slice;
- docs describe the shipped lifecycle boundary;
- marketplace, package signing, and broader distribution are split or deferred.
