# Database Guidelines

> Database patterns and conventions for this project.

---

## Overview

<!--
Document your project's database conventions here.

Questions to answer:
- What ORM/query library do you use?
- How are migrations managed?
- What are the naming conventions for tables/columns?
- How do you handle transactions?
-->

(To be filled by the team)

---

## Query Patterns

<!-- How should queries be written? Batch operations? -->

(To be filled by the team)

---

## Migrations

<!-- How to create and run migrations -->

Add schema changes as versioned SQL files under `crates/nako-db/migrations/`
and `crates/nako-db/migrations/postgres/`, then register both in the matching
SQLite/Postgres `MIGRATIONS` arrays. Keep adapter row mappers, insert SQL, and
contract tests in sync with every new durable column.

## Scenario: Durable Job Priority Policy

### 1. Scope / Trigger

- Trigger: durable job rows gained a scheduler-visible `priority` column.
- Scope: `nako-core` job structs, SQLite/Postgres migrations, SQLite/Postgres
  job adapters, retry enqueue behavior, lease claiming, and DB contract tests.
- Boundary: priority is a generic durable-job policy. Do not encode provider,
  metadata review, addon, scan, or Admin-Web semantics in the scheduler.

### 2. Signatures

- `NewJob { priority: JobPriority, ... }`
- `Job { priority: JobPriority, ... }`
- `JobPriority::{Low, Normal, High}` maps to persisted scores
  `0`, `50`, and `100`.
- `jobs.priority` is `INTEGER NOT NULL DEFAULT 50` on SQLite and
  `bigint NOT NULL DEFAULT 50` on Postgres.

### 3. Contracts

- Every enqueue path must set a priority explicitly. Existing work should use
  `JobPriority::Normal` unless a generic durable-job policy says otherwise.
- `enqueue_job_retry` must copy the source job priority to the retry row.
- `claim_next_job_lease` orders eligible queued jobs by aged fairness first,
  then priority, then FIFO tie-breakers.
- API/Admin diagnostics should not expose a priority field unless a separate
  read-only diagnostic follow-on adds that surface deliberately.

### 4. Validation & Error Matrix

- Unknown persisted score -> return `NakoError::Database`.
- Missing migration registration -> migrated stores lack `jobs.priority` and
  job repository tests must fail.
- Retry row priority differs from source -> contract violation.
- Business-specific scheduler branch -> architecture violation against ADR 0053.

### 5. Good/Base/Bad Cases

- Good: a high-priority generic job claims before a fresh low-priority job in
  the same filter/resource class.
- Base: old jobs migrated without an explicit score become normal priority.
- Bad: an endless stream of fresh high-priority rows prevents aged low-priority
  rows from ever being claimed.

### 6. Tests Required

- Contract test for priority ordering.
- Contract test for starvation guard/fairness.
- Contract test that retry and lease recovery preserve priority.
- Migration tests must assert the new migration version is applied.

### 7. Wrong vs Correct

#### Wrong

```rust
NewJob {
    kind: JobKind::MetadataCandidateReviewBatchApply,
    resource_class: "metadata.candidate_review.apply".to_owned(),
    priority: JobPriority::High, // business-specific scheduler shortcut
    // ...
}
```

#### Correct

```rust
NewJob {
    kind,
    resource_class,
    priority: JobPriority::Normal, // default generic durable-job policy
    // ...
}
```

---

## Naming Conventions

<!-- Table names, column names, index names -->

(To be filled by the team)

---

## Common Mistakes

<!-- Database-related mistakes your team has made -->

(To be filled by the team)
