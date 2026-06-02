# Provider Governance Durable Batch Execution - Closeout

Date: 2026-06-02
Status: DONE
Task: PGDBE-060

## Final State

Provider Governance Durable Batch Execution is closed after `PGDBE-060`.

Shipped:

- `PGDBE-020`: Candidate Review durable batch records, item statuses,
  execution summaries, explicit job kind/resource class, repository trait
  methods, SQLite/PostgreSQL schema support, and repository contract tests.
- `PGDBE-030`: Admin durable batch create/status routes with idempotent queued
  batch creation, durable job persistence, redaction-safe status reads, and
  generated Admin TypeScript contract sync.
- `PGDBE-040`: job-backed execution through `DurableJobRuntime`, metadata
  shared resource-class mapping, per-item application through the existing
  single-review authority, persisted outcomes, and cancellation checkpoints.
- `PGDBE-050`: Web Admin selected-review durable batch creation, compact
  redaction-safe status rendering, queued/running polling, data-source and
  route-state coverage, and green Web bundle budget.
- `PGDBE-060`: closeout and follow-on split.

## Preserved Boundaries

- Candidate Review batch governance remains Admin-only.
- Execution uses `DurableJobRuntime`; no raw `tokio::spawn` or hidden
  background helper was added.
- Per-item mutation calls the existing single-review application service.
- Root Provider Subject / Provider Mapping application remains the only
  mutation path in this lane.
- Related Provider Subject, child Provider Mapping, and Media Item hierarchy
  application remain out of scope.
- Public Client API exposure remains out of scope.
- Provider endpoint breadth, including Douban TV/episode endpoint depth,
  remains out of scope.
- Audit/undo expansion remains out of scope beyond persisted batch/item status
  evidence.
- Admin/Web surfaces do not render raw provider payloads, bearer tokens, local
  paths, source fingerprints, or raw idempotency keys.

## Closeout Gates

Fresh Web verification on 2026-06-02:

```bash
npm --prefix web run check
npm --prefix web run test
npm --prefix web run build:budget
```

Results:

- TypeScript check: passed.
- Web tests: passed, 10 test files / 122 tests.
- Bundle budget: passed; total JS gzip was 344.96 KiB against the 345 KiB
  limit.

Fresh backend verification from accepted tasks:

- `PGDBE-020`: `cargo test -p nako-db metadata_candidate_review_batch -- --nocapture`
  passed for SQLite; PostgreSQL contract remained ignored without
  `NAKO_TEST_POSTGRES_URL`.
- `PGDBE-030`: `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
  and `cargo test -p nako-api admin_contract -- --nocapture` passed.
- `PGDBE-040`: `cargo test -p nako-server metadata_candidate_review_batch -- --nocapture`
  and `cargo test -p nako-metadata candidate_review_application -- --nocapture`
  passed.

Local route smoke on 2026-06-02:

- `http://127.0.0.1:3000/admin/metadata/candidate-reviews?status=accepted&provider=bangumi&limit=25&offset=0`
  returned HTTP 200 with 1155 bytes.
- The same route on port `3001` returned HTTP 200 with 1155 bytes.

Fresh closeout docs verification on 2026-06-02:

```bash
python -m json.tool docs/workstreams/provider-governance-durable-batch-execution/WORKSTREAM.json
git diff --check
```

Results:

- `WORKSTREAM.json` validation: passed.
- `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl` validation: passed.
- `git diff --check`: passed; Git emitted only LF/CRLF normalization warnings.

## Review Result

No blocking workstream compliance findings remain:

- the target state in `DESIGN.md` is satisfied;
- all tasks in `TODO.md` and `TASKS.jsonl` are accepted;
- all campaigns are closed;
- `WORKSTREAM.json` is closed;
- architecture maps, `docs/GOALS.md`, and `docs/ROADMAP.md` route this lane as
  closed evidence.

No blocking code-quality findings remain:

- durable batch create/status does not execute Provider Mapping writes;
- execution flows through the existing single-review authority;
- per-item outcomes preserve partial failure instead of hiding it;
- Web fixture/fallback mode remains read-only;
- Web bundle budget remains green without widening limits.

## Follow-Ons

Open focused workstreams before implementing:

- `proposed:provider-review-related-hierarchy-application`
- `proposed:douban-tv-episode-endpoint-depth`
- `proposed:provider-review-public-client-governance`
- `proposed:provider-governance-audit-and-undo`
- `proposed:durable-job-priority-policy-and-scheduler-migration`

## Residual Risks

- The Web status panel is compact by design to keep bundle budget green; richer
  operations diagnostics should be part of an operations repair lane.
- PostgreSQL parity is implemented, but the local environment did not provide
  `NAKO_TEST_POSTGRES_URL` during PGDBE-020 verification.
- Browser plugin tooling was unavailable in this session; local HTTP route
  smoke and route-state tests covered the route behavior instead.
