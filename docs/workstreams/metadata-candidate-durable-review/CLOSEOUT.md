# Metadata Candidate Durable Review - Closeout

Status: Closed
Closed: 2026-06-02

## Shipped Scope

This lane turned provider Candidate Graph previews into a durable,
redaction-safe review boundary without making preview evidence an implicit
Provider Mapping mutation.

Shipped behavior:

- `MCDR-020` added a pure `MetadataCandidateGraph` -> review plan contract.
- `MCDR-030` added durable candidate review snapshot records, repository
  contracts, SQLite/PostgreSQL baseline schema, and DB round-trip coverage.
- `MCDR-040` added backend-only accept/reject status transitions with
  idempotency, stale guards, expiry handling, and no Provider Mapping writes.
- `MCDR-050` closes the lane and splits remaining product/application work into
  explicit follow-ons.

## Confirmed Boundaries

- Candidate review snapshots store only redaction-safe review plans, not raw
  provider payloads, paths, headers, proxy URLs, tokens, or secrets.
- Automatic refresh remains root-only for accepted Provider Mapping behavior.
- Review decision status changes do not create or update Provider Mapping rows.
- Generated Artifact apply outcomes remain a separate control-plane workflow and
  are not reused as a generic candidate review queue.
- Admin/Web and Public Client routes remain out of this lane.

## Validation

Fresh closeout gates:

- `cargo nextest run -p nako-metadata candidate_review_decision --no-fail-fast`
  passed: 3 tests run, 3 passed.
- `cargo nextest run -p nako-db candidate_review --no-fail-fast` passed: 1
  test run, 1 passed.
- `python -m json.tool docs/workstreams/metadata-candidate-durable-review/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

Recent broader implementation gates:

- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo nextest run -p nako-db --no-fail-fast`

## Follow-Ons

- `proposed:admin-web-provider-depth-governance`: expose durable provider graph
  review evidence and explicit operator decisions through Admin/Web.
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed):
  apply accepted candidate reviews to Provider Subject / Provider Mapping state
  through a named backend service, separate from review status transitions.
- `proposed:douban-tv-episode-endpoint-depth`: prove Douban TV/episode endpoint
  semantics before broadening Douban capability claims.

## Residual Risks

- Upserting a refreshed snapshot currently creates a new pending review state
  for the item/source/source-key tuple. A future application lane should decide
  fingerprint/version semantics before applying accepted review decisions to
  Provider Mapping rows.
- Admin/Web must continue to display preview evidence separately from accepted
  Provider Mapping facts until the application follow-on ships.
