# Addon Manager Lifecycle Automation - Milestones

Status: Completed
Last updated: 2026-05-23

## M0 - Boundary Frozen

Exit criteria:

- the manager problem is distinct from Addon runtime/distribution;
- registry, permissions, token rotation, Addon Health Check, and Addon Install
  Guide behavior are the first slice;
- process/container supervision, marketplace, and package signing are explicit
  non-goals or follow-ons;
- the first executable task is identified.

## M1 - Source Shape Frozen

Exit criteria:

- the first managed addon source shape is documented;
- install/update/remove intent is represented as a Nako-owned plan;
- operator-facing lifecycle state is observable without leaking secrets.

## M2 - First Lifecycle Slot

Exit criteria:

- one addon can be represented through a Nako-owned lifecycle slot;
- health and install-guide behavior are surfaced through Nako;
- operator-confirmed plan transitions are explicit and testable.

Status note: AMG-030 satisfied this milestone's exit criteria; continue with
AMG-060 for closeout or split follow-ons.

## M3 - Closeout

Exit criteria:

- fresh gates prove the manager slice;
- docs describe the shipped lifecycle boundary;
- marketplace, package signing, provider breadth, and process supervision are
  split or deferred.

Status note: AMG-060 closed the lane after fresh format, server addon, API
compile, and official addon smoke gates passed. Remaining breadth is deferred
to follow-on lanes.
