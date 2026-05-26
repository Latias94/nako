# Metadata Application Cross-Path Audit - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

The audit is closed as a documentation-only lane. No provider/hierarchy code was
changed.

## Decision

Do not make provider refresh or hierarchy confirmation call server
`MetadataApplication`.

The first-class shared abstraction remains `MetadataMergePolicy` in
`nako-core`. A future extraction should be pure core only, and only if several
apply paths need a common command/result/report type independent of repositories
and catalog projection.

## Blockers

None.
