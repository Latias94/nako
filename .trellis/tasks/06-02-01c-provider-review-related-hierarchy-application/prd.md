# Provider Review Related Hierarchy Application

## Goal

Plan and implement the focused follow-on for applying accepted provider review
related hierarchy changes without reopening completed provider governance bulk
or durable execution work.

## Requirements

* Preserve existing metadata merge and root Provider Mapping authority
  decisions unless a new ADR explicitly changes them.
* Start with a backend-first slice that proves the related hierarchy application
  policy and persistence boundary before adding Admin Web UX.
* Reuse durable candidate review and provider governance evidence as historical
  context only; Trellis task state owns current execution.
* Keep Public Client API exposure, audit/undo, and provider endpoint breadth out
  of this task unless explicitly promoted to scope.
* If Admin API/Web surfaces are needed, coordinate generated contract changes
  with the Admin settings task.

## Acceptance Criteria

* [ ] PRD confirms the exact related hierarchy operation and out-of-scope
      provider governance areas before implementation begins.
* [ ] Backend tests prove accepted-review-only behavior and rejection of
      ambiguous or unsafe hierarchy application.
* [ ] Persistence behavior remains consistent with ADR 0007 unless an ADR update
      is approved.
* [ ] Admin/Public API boundary remains explicit.

## Definition of Done

* Scoped nextest run passes.
* Metadata/control-plane docs or Trellis specs are updated only if durable
  behavior changes.
* No Admin Web/generated contract change is introduced without coordinating the
  shared surface.

## Worktree

Suggested path: `E:\Rust\nako-worktrees\01c-provider-related-hierarchy`

Suggested branch: `task/01c-provider-related-hierarchy`

Conflict note: high overlap with Admin API/Web generated contracts if the task
expands beyond backend policy and persistence.
