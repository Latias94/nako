# Nako server control-plane seam deepening

## Goal

Deepen the `nako-server` startup/composition/runtime seam so server initialization, runtime supervision, and startup reporting are owned by a clear control-plane workflow instead of leaking through `NakoApp` construction and route-adjacent helpers. The point is better locality and leverage now, and a smaller surface for future diagnostics, recovery, and durable-job work.

## What I Already Know

* `docs/architecture/CONTROL_PLANE.md` places durable jobs, runtime supervision, diagnostics, remote access, and API scale contracts under one shared control-plane roof.
* `docs/architecture/LANES.md` routes control-plane work as a concrete follow-on lane when M1/M2 evidence needs deeper ownership.
* `nako-server` already has `app/composition.rs`, `app/startup.rs`, `app/runtime.rs`, and `app/job_runtime.rs`, but the startup path still reads like one broad construction chain.
* The repo already expects HTTP handlers to stay thin and app services to own workflow logic.
* The strongest next candidate from the current analysis is the `nako-server` startup/composition/runtime seam, not playback or library intake.

## Research References

* [`research/control-plane-seam-deepening.md`](research/control-plane-seam-deepening.md) - candidate selection and why this seam is the highest-leverage next slice.
* [`docs/research/nako-product-competitive-analysis/competitive-analysis-summary.md`](../../docs/research/nako-product-competitive-analysis/competitive-analysis-summary.md) - overall competitive framing.
* [`docs/research/nako-product-competitive-analysis/nako-current-state.md`](../../docs/research/nako-product-competitive-analysis/nako-current-state.md) - current product and architecture status.
* [`docs/research/nako-product-competitive-analysis/jellyfin-plex-competitive-landscape.md`](../../docs/research/nako-product-competitive-analysis/jellyfin-plex-competitive-landscape.md) - benchmark context for operator trust and client polish.

## Assumptions

* The first slice should not reopen playback, library intake, metadata, or HTTP route shape.
* This task should deepen the control-plane seam, not introduce a new scheduler architecture.
* Existing startup-visible behavior such as startup reporting, runtime diagnostics, watcher coverage, and addon/runtime registration should remain intact.

## Requirements

* Make server startup a named workflow rather than a passive construction side effect.
* Keep `NakoApp` as the composition root, not a feature owner.
* Keep `RuntimeSupervisor` as the authoritative place for supervised task and durable-job execution helpers.
* Preserve current startup-visible behavior: startup report contents, runtime diagnostics, watcher coverage, addon/runtime registration, and redaction boundaries.
* Keep the seam small enough that tests can target startup/report/runtime behavior directly.
* Any new helper must improve locality rather than just moving code between files.

## Acceptance Criteria

* [ ] Startup/composition logic is thinner and more explicit in the `nako-server` app boundary.
* [ ] Runtime supervision and startup reporting have a clearer ownership seam that can be tested in isolation.
* [ ] Existing startup and runtime diagnostics behavior remains intact.
* [ ] Focused tests cover the refactored workflow and its redaction-safe outputs.
* [ ] `cargo fmt --all`, focused `cargo nextest`, and `cargo check -p nako-server --tests` pass or limitations are recorded.

## Definition of Done

* Tests added or updated.
* Lint, typecheck, and CI gates green for the touched package.
* Docs/notes updated if behavior changes.
* Rollout/rollback considered if the refactor proves risky.

## Out of Scope

* Playback planning, transcode, or renderer changes.
* Library intake, scan scheduling, or watcher semantics changes except where startup wiring requires them.
* HTTP route shape or public DTO changes.
* Database schema or addon protocol changes.
* A broader scheduler rewrite.

## Technical Approach

* First PR: map the current startup/composition/runtime flow and lock down the existing seam with task-local research and targeted tests.
* Second PR: move startup workflow responsibility behind a named app-service entry point while keeping `NakoApp` focused on construction and wiring.
* Third PR: tighten runtime supervisor ownership, remove remaining startup/report couplings, and update tests/docs if the seam changes.

## Decision (ADR-lite)

**Context**: the current server root is broad enough that initialization, service graph construction, and runtime supervision read like one layer.

**Decision**: deepen the control-plane seam by extracting a named startup workflow and clarifying runtime-supervisor ownership while preserving external behavior.

**Consequences**: fewer cross-file dependencies and better test locality, at the cost of some short-term churn in app wiring and tests.

