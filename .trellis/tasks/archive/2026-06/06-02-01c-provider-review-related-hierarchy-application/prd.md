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

## Confirmed Operation

This backend-first slice adds an internal related hierarchy application method
for accepted Metadata Candidate Reviews. It does not change the existing Admin
apply route, durable batch execution, Public Client API, Admin Web, or generated
contracts.

The supported operation is deliberately narrow:

* The review must be `accepted`, and the root Provider Mapping for the review
  item must already be `accepted`.
* Only `contains` relationships anchored at the accepted root Provider Subject
  are eligible.
* Safe hierarchy shapes are `Series -> Season`, `Series -> Episode`, and
  `Season -> Episode`.
* The related node must match exactly one existing child Media Item under the
  root item within the root item's library memberships. Missing, duplicate, or
  otherwise ambiguous targets are rejected before related writes.
* Application upserts the related Provider Subject when needed, creates or
  promotes the related Provider Mapping to `accepted`, and marks the matched
  child library item state as non-provisional.
* The operation does not create Media Items, change existing item parentage,
  change canonical metadata fields, write NFO/library files, or bypass rejected
  Provider Mapping protection.

ADR 0007 remains unchanged because canonical metadata merge behavior is not
expanded in this slice.

## Acceptance Criteria

* [x] PRD confirms the exact related hierarchy operation and out-of-scope
      provider governance areas before implementation begins.
* [x] Backend tests prove accepted-review-only behavior and rejection of
      ambiguous or unsafe hierarchy application.
* [x] Persistence behavior remains consistent with ADR 0007 unless an ADR update
      is approved.
* [x] Admin/Public API boundary remains explicit.

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
