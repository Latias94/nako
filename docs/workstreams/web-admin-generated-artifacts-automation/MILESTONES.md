# Web Admin Generated Artifacts Automation - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Open Lane

Exit criteria: scope, target state, route/API readiness, task ledger, gates, and
handoff are aligned.

Completed by `WAGA-010`.

## M1 - Admin API And Read-Model Audit

Exit criteria: generated Admin proposal/review DTOs and the `web/` read-model
mapping are documented and covered by data-source contract tests.

Completed by `WAGA-020`.

## M2 - Read-Only Proposal Route

Exit criteria: `/admin/automation/generated-artifacts` renders proposal
diagnostics with route-owned pagination and redaction assertions.

## M3 - Review Mutation Guard Decision

Exit criteria: review-plan and accept/reject actions are either explicitly
deferred or guarded by proven confirmation, boundary, idempotency, and redaction
tests.

## M4 - Closeout

Exit criteria: frontend tests, TypeScript check, bundle budget, browser smoke,
and workstream docs agree on what shipped and what remains deferred.
