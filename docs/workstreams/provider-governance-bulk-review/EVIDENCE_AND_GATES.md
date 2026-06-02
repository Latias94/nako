# Provider Governance Bulk Review - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Evidence

Source coverage:

- `CONTEXT.md` read for Nako domain terms.
- ADR 0007, ADR 0018, ADR 0021, and ADR 0053 read before opening.
- `docs/GOALS.md`, `docs/ROADMAP.md`, `docs/architecture/LANES.md`, and
  `docs/architecture/LIBRARY_PIPELINE.md` inspected after PRGQ closeout.
- `docs/workstreams/provider-review-global-queue-search/CLOSEOUT.md` read and
  used as the immediate follow-on authority.
- `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md` read and
  used as the single-review Admin/Web governance authority.
- `scripts/workstream_inventory.py`, `scripts/program_status.py`, and
  `scripts/validate_orchestration_state.py` are not present in this checkout;
  a read-only `WORKSTREAM.json` status scan found no active implementation
  workstreams before opening this lane.

Green opening gates for `PGBR-010`:

- `python -m json.tool docs/workstreams/provider-governance-bulk-review/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed: 5 task records, 2 campaign records, and 13 context records.
- `git diff --check` passed with Git CRLF normalization warnings only.

## PGBR-020 Gates

Implementation gates:

- Attempt `cargo nextest` when available; record local fallback if unavailable.
- `cargo test -p nako-api admin_contract -- --nocapture`
- `cargo test -p nako-server metadata_candidate_review -- --nocapture`
- `cargo test -p nako-metadata candidate_review_application -- --nocapture`
- `cargo fmt --all -- --check`
- `git diff --check`

Behavior evidence required:

- batch plan route is read-only and writes no Candidate Review, Provider
  Subject, Provider Mapping, Canonical Metadata, job, or outcome rows;
- selected review IDs are bounded and duplicate handling is deterministic;
- stale, rejected, pending, already-applied, blocked, noop, and eligible rows
  are classified through existing single-review application planning semantics;
- DTOs redact raw provider payloads, provider bodies, image URLs, local paths,
  headers, proxy URLs, source fingerprints, and raw idempotency keys;
- generated Admin TypeScript contract remains synchronized.

## PGBR-030 Gates

Implementation gates:

- focused `nako-metadata` Candidate Review application gates;
- focused `nako-api` Admin contract gates;
- focused `nako-server` Candidate Review route gates;
- `cargo fmt --all -- --check`;
- `git diff --check`.

Behavior evidence required:

- confirmed batch apply preserves per-review stale guard and idempotent replay;
- partial results are redaction-safe and distinguish applied, noop, conflict,
  stale, blocked, failed, and skipped rows;
- only root Provider Subject / Provider Mapping state may mutate;
- no related hierarchy mutation occurs;
- durable job/runtime execution is used or explicitly split if synchronous
  bounded execution is insufficient.

## PGBR-040 Gates

Implementation gates:

- `npm --prefix web run check`
- `npm --prefix web run test`
- `npm --prefix web run build:budget`
- browser smoke for queue selection -> batch plan -> confirmation/result route
- `git diff --check`

Behavior evidence required:

- Web Admin selection is explicit and bounded;
- operators inspect a batch plan before confirmation;
- route state preserves queue context;
- fixture fallback does not claim live mutation success;
- no Public Client API surface or related hierarchy mutation is introduced.
