# Nako server control-plane seam deepening

## Context

`nako-server` already has explicit app/composition/runtime modules, but the
startup path still reads like a broad construction chain. The goal is to make
startup feel like a named control-plane workflow with clearer ownership for
composition, startup reporting, and runtime supervision.

## Target Shape

* `NakoApp` remains the composition root.
* `startup.rs` becomes the owner of a named startup workflow.
* `runtime.rs` remains the authority for supervised task and durable-job
  helpers.
* The startup workflow returns a typed report that can be tested directly.
* Route handlers stay out of the refactor unless they already depend on startup
  output.

## Candidate Boundaries

### Option A: Startup workflow extraction

Move the most visible startup logic behind a named app-service entry point and
keep `NakoApp` focused on wiring.

Pros:

* Highest locality gain.
* Smallest surface area.
* Fits current docs and lane boundaries.

Cons:

* May leave some construction couplings for a later pass.

### Option B: Startup workflow plus runtime helper tightening

Extract the startup workflow and also narrow the runtime helper surface where
startup currently depends on broad runtime details.

Pros:

* Better long-term seam.
* Makes diagnostics and future recovery work easier.

Cons:

* More churn in one task.
* Higher regression risk.

## Chosen Direction

Use Option A as the first slice, then tighten runtime helper ownership only if
the resulting workflow still feels shallow.

## Data Flow

```text
Config / Store / Runtime inputs
  -> NakoApp composition root
  -> named startup workflow
  -> startup report / runtime supervisor wiring
  -> diagnostics and tests
```

## Risks

* Startup behavior may be spread across too many small helpers and become
  mechanically moved instead of meaningfully deepened.
* Startup-visible diagnostics may regress if the report contract is not pinned
  with tests.
* Over-deepening the seam could make the composition root harder to follow.

## Rollout / Rollback

* Keep changes behavior-preserving.
* Lock existing startup report and runtime diagnostics behavior with tests.
* If the seam deepening proves too broad, stop after the startup workflow
  extraction and defer runtime tightening to a follow-on task.

