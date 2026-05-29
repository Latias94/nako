# Web Admin Acquisition Intake - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Open Lane

Exit criteria: scope, target state, route/API readiness, task ledger, gates, and
handoff are aligned.

Completed by `WAAI-010`.

## M1 - Admin API And Read-Model Audit

Exit criteria: generated Admin acquisition DTOs and the `web/` read-model
mapping are documented and covered by data-source contract tests.

Completed by `WAAI-020`.

## M2 - Route-First Intake Page

Exit criteria: `/admin/acquisition/intake` renders a read-only candidate
diagnostic workflow with route-owned query state and redaction assertions.

Completed by `WAAI-030`.

## M3 - Mutation Boundary Decision

Exit criteria: watch-folder discovery and candidate acceptance are either
explicitly deferred or guarded by a proven mutation contract.

Completed by `WAAI-040`.

## M4 - Closeout

Exit criteria: frontend tests, TypeScript check, bundle budget, browser smoke,
and workstream docs agree on what shipped and what remains deferred.
