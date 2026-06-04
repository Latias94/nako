# Durable Job Priority Policy And Scheduler Migration

## Goal

Introduce durable job priority policy and scheduler migration as a control-plane
follow-on split from provider governance durable batch execution.

## Requirements

* Start from ADR 0005, ADR 0006, and ADR 0053 control-plane boundaries.
* Keep the policy generic to durable jobs; do not bake provider review semantics
  into scheduler priority.
* Preserve bounded async behavior and explicit retry policy.
* Avoid raw `tokio::spawn` control-plane work where supervised job runtime
  patterns exist.
* Add tests that prove priority ordering/admission without starving lower
  priority work.

## Acceptance Criteria

* [ ] Durable jobs have an explicit priority policy and migration path.
* [ ] Existing provider governance durable batch execution remains compatible.
* [ ] Tests cover priority ordering, fairness or starvation guard, and recovery
      behavior relevant to the selected design.
* [ ] Control-plane architecture/spec notes are updated if priority becomes a
      durable invariant.

## Definition of Done

* Scoped nextest run passes for runtime/job areas touched.
* No provider-review-specific behavior is added to generic scheduler code.
* No Admin Web work unless a separate read-only diagnostic follow-on is opened.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01f-durable-job-priority-policy`

Suggested branch: `task/01f-durable-job-priority-policy`

Conflict note: coordinate with metadata/provider work only if it changes durable
batch execution semantics.
