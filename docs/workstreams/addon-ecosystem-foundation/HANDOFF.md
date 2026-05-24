# Addon Ecosystem Foundation — Handoff

Status: Active
Last updated: 2026-05-25

## Current State

The lane has been opened after the Addon ecosystem architecture review. The
accepted decision is that Nako should keep fine-grained Addon manifests,
permissions, tasks, events, and audit while allowing coarse-grained Addon
Package and Addon Suite deployment. Official addons should not force operators
to write one Docker Compose service per small capability.

AEF-010 is complete as the authority freeze.

## Active Task

- Task ID: AEF-020
- Owner: codex
- Files:
  - `crates/nako-core/src/addon_task.rs`
  - `crates/nako-db/src/sqlite/addon_tasks.rs`
  - PostgreSQL schema/contracts if touched
  - `crates/nako-server/src/app/addons/task_runtime.rs`
- Validation:
  - `cargo nextest run -p nako-db addon_task --no-fail-fast`
  - `cargo nextest run -p nako-server addon_task --no-fail-fast`
- Status: NEEDS_CONTEXT
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Addon is the authority and capability unit.
- Addon Package and Addon Suite are deployment/distribution units.
- Official addons should default to suite packaging when capabilities share
  trust, dependencies, and lifecycle.
- Network Tunnel Provider behavior stays outside core; core owns Remote Access
  Endpoint facts.
- MCP/AI outputs should enter through Generated Artifacts and Acceptance
  Workflow unless a specific Addon write grant applies.

## Blockers

- None currently.

## Next Recommended Action

Implement AEF-020 Addon Task request fingerprinting before event delivery or
new official addon breadth.
