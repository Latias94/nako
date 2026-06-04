# Architecture Boundary Refactor Review

## Goal

Find high-leverage architecture and module-boundary refactoring opportunities in
Nako, with a fearless-refactor bias toward deleting accidental complexity,
deepening shallow modules, and improving test locality before the codebase grows
around weak seams.

## What I Already Know

* The user wants sub-agent review of architecture and module boundaries.
* The desired output is a set of fearless refactoring opportunities, not an
  immediate code change.
* Nako is a Rust modular-monolith workspace with explicit crate boundaries in
  `AGENTS.md`, domain vocabulary in `CONTEXT.md`, and durable architecture
  decisions in `docs/adr/`.
* Recent work touched Admin VFS cache repair previews across API, server,
  repository, DB adapters, generated contracts, and storage architecture specs.
* Good candidates should use project vocabulary such as Media Library, Media
  Source, Source Locator, Admin API, Public Client API, Playback Runtime,
  Storage Backend Health, Storage Circuit Breaker, Addon, Generated Artifact,
  and Acceptance Workflow.

## Requirements

* Review architecture and module boundaries using sub-agents.
* Look for deepening opportunities: smaller interfaces with more behavior behind
  them, better locality, fewer pass-through modules, and clearer seams.
* Prioritize opportunities that can remove redundant mapping, duplicated
  orchestration, shallow helper layers, or cross-crate leakage.
* Respect existing ADRs. If a recommendation conflicts with an ADR, flag the ADR
  explicitly and explain why reopening it may be justified.
* Keep this task read-only unless the user later chooses a refactor candidate.
* Persist findings under this Trellis task so future work can continue after
  compaction.

## Acceptance Criteria

* [x] At least three independent sub-agent reviews are completed and persisted
      under `research/`.
* [x] Each review names concrete files/modules, the friction observed, and the
      proposed refactoring direction.
* [x] The main report consolidates findings into a ranked list of opportunities
      with expected leverage, locality gains, test impact, risk, and likely
      workflow scale.
* [x] The report distinguishes immediate small cleanups from larger workstreams
      or architecture lanes.
* [x] No code changes are made during the review task.

## Definition of Done

* Findings are written to task artifacts.
* The user receives a concise ranked architecture review.
* Any selected candidate can be turned into a `fearless-refactor` brief or a new
  implementation task.
* Git remains clean except for committed or explicitly reported Trellis task
  artifacts.

## Out of Scope

* Implementing refactors in this task.
* Rewriting crates or changing public API shape without a follow-up task.
* Opening schema migrations or changing runtime behavior.
* Auditing every line of the full workspace exhaustively.

## Review Axes

* Crate boundaries and dependency direction.
* Admin API / server app-service / repository / DB adapter mapping depth.
* Storage, VFS cache, staging, and Storage Backend Health seams.
* Playback Runtime and transcode/streaming boundary shape.
* Addon, automation, event, and Generated Artifact workflows.
* Tests: whether behavior is tested at the right interface or through brittle
  implementation detail.

## Technical Notes

* Domain glossary: `CONTEXT.md`.
* ADRs: `docs/adr/`.
* Architecture maps: `docs/ARCHITECTURE.md`, `docs/architecture/`.
* Skill framing: `improve-codebase-architecture`, then `fearless-refactor` only
  after the user chooses a candidate.
