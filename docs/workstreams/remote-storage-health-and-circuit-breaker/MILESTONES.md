# Remote Storage Health And Circuit Breaker - Milestones

Status: Closed
Last updated: 2026-05-31

## M0 - Scope And Evidence Freeze

Exit criteria:

- workstream files exist and agree on target state;
- architecture maps link the real workstream instead of only a proposed slug;
- `RSHC-020` has a bounded owner scope and validation command.

Status: Done.

## M1 - Durable Health Contract

Exit criteria:

- health record vocabulary is explicit and redaction-safe;
- SQLite and PostgreSQL adapters pass shared repository contract tests;
- storage health can represent open, half-open/recovering, and healthy states
  without leaking raw locators or paths.

Status: Done.

## M2 - Runtime Policy Adapter

Exit criteria:

- runtime storage/VFS callers can record failures and successes through one
  adapter;
- repeated failures produce bounded and explainable circuit-breaker decisions;
- existing scan/probe/playback staging behavior remains compatible unless a
  task explicitly changes it.

Status: Done.

## M3 - Operator Diagnostics And Reset

Exit criteria:

- Admin diagnostics expose backend health, last failure class, counters, and
  reset eligibility;
- reset action is persisted, audited through normal route behavior, and tested;
- generated client contracts are refreshed if DTO shape changes.

Status: Done.

## M4 - Verification And Closeout

Exit criteria:

- focused nextest gates pass with fresh evidence;
- `WORKSTREAM.json` evidence and task status are current;
- architecture docs reflect shipped behavior;
- follow-ons are split or explicitly deferred.

Status: Done.
