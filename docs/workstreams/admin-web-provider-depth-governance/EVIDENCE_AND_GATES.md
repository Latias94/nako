# Admin Web Provider Depth Governance - Evidence And Gates

Status: Active
Last updated: 2026-06-02

## Opening Gates

```bash
python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json
```

```bash
python - <<'PY'
import json
from pathlib import Path
for rel in [
    "docs/workstreams/admin-web-provider-depth-governance/TASKS.jsonl",
    "docs/workstreams/admin-web-provider-depth-governance/CAMPAIGNS.jsonl",
    "docs/workstreams/admin-web-provider-depth-governance/CONTEXT.jsonl",
]:
    for line in Path(rel).read_text(encoding="utf-8").splitlines():
        if line.strip():
            json.loads(line)
print("jsonl ok")
PY
```

```bash
git diff --check
```

## Expected Gates

- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
- `cargo fmt --all -- --check`
- `npm --prefix web run test`
- `npm --prefix web run check`
- `npm --prefix web run build:budget`
- browser smoke if a route is added
- `git diff --check`

## Evidence Anchors

- `docs/workstreams/accepted-review-provider-mapping-application/CLOSEOUT.md`
- `docs/workstreams/metadata-candidate-durable-review/CLOSEOUT.md`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/CLOSEOUT.md`
- `docs/workstreams/admin-catalog-governance-read-model/README.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/LANES.md`

## Opening Recon

Local recon on 2026-06-02:

- Backend accepted-review application exists in
  `MetadataCandidateReviewApplicationService`.
- Existing Admin Catalog Governance Provider Mapping review routes mutate
  already persisted Provider Mapping rows; they do not expose durable Candidate
  Review snapshots or application plans.
- Metadata diagnostics Candidate Review DTOs expose matching/review facts but
  are not the durable application surface.
- Generated Artifact Provider Mapping apply proves plan/apply/result patterns,
  but its outcome tables remain out of scope for Candidate Review state.

## AWPDG-010 Evidence

Implemented behavior:

- opened this workstream as the Admin/Web provider depth governance follow-on
  from ARPMA closeout;
- selected a read-only Admin API Candidate Review detail/application-plan slice
  as the first executable task;
- kept apply mutation, Web, Public Client API, and related-node hierarchy
  application out of the first campaign.

Green checks:

- `python -m json.tool docs/workstreams/admin-web-provider-depth-governance/WORKSTREAM.json`
  passed.
- JSONL validation for `TASKS.jsonl`, `CAMPAIGNS.jsonl`, and `CONTEXT.jsonl`
  passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## AWPDG-020 Evidence

Implemented behavior:

- added `GET /admin/v1/metadata/candidate-reviews/{review_id}`;
- exposed durable Metadata Candidate Review detail, preview related graph
  evidence, and accepted-review application plan facts through Admin DTOs;
- redacted candidate metadata to titles, dates, presence flags, and counts
  instead of raw descriptions, tags, image URLs, provider bodies, paths, tokens,
  or fingerprints;
- reused `build_candidate_review_application_plan` for action/reason/source
  facts instead of duplicating application rules in HTTP;
- kept the route read-only: no Provider Subject, Provider Mapping, Canonical
  Metadata, or related graph state is written on read;
- regenerated both Admin TypeScript contract outputs.

Red check:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_detail --no-fail-fast`
  initially failed because Admin Candidate Review DTOs and route were missing.

Green checks:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_detail --no-fail-fast`
  passed: 1 test run, 1 passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 5
  tests run, 5 passed.
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
  passed: 108 tests run, 108 passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

## AWPDG-030 Evidence

Implemented behavior:

- added `POST /admin/v1/metadata/candidate-reviews/{review_id}/apply`;
- added an explicit Admin apply request with `item_id`,
  `expected_updated_at_ms`, and an operator-provided `idempotency_key`;
- validated the idempotency key at the Admin boundary and exposed only a short
  fingerprint in responses;
- called `MetadataCandidateReviewApplicationService` instead of duplicating
  apply rules in HTTP;
- made apply, conflict, noop, and idempotent replay outcomes visible through
  Admin DTO fields;
- proved stale `expected_updated_at_ms` conflicts do not write Provider Mapping
  state;
- proved empty idempotency keys fail with `invalid_input`;
- persisted only the root Provider Subject and accepted Provider Mapping on
  first apply;
- kept related candidate nodes as preview evidence and did not persist related
  hierarchy subjects;
- regenerated both Admin TypeScript contract outputs.

Red check:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_apply_commits_root_mapping_and_replays --no-fail-fast`
  initially failed because the Admin apply DTOs and route did not exist.

Green checks:

- `cargo nextest run -p nako-server admin_v1_metadata_candidate_review_apply_commits_root_mapping_and_replays --no-fail-fast`
  passed: 1 test run, 1 passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 5
  tests run, 5 passed.
- `cargo nextest run -p nako-metadata candidate_review_application --no-fail-fast`
  passed: 6 tests run, 6 passed.
- `cargo nextest run -p nako-server candidate_review admin --no-fail-fast`
  passed: 109 tests run, 109 passed.

## AWPDG-040 Evidence

Implemented behavior:

- added Web Admin client/read-model/mutation data-source support for
  `GET /admin/v1/metadata/candidate-reviews/{review_id}` and
  `POST /admin/v1/metadata/candidate-reviews/{review_id}/apply`;
- added the direct route
  `/admin/metadata/candidate-reviews?review_id=<id>` for durable Candidate
  Review inspection;
- rendered root Candidate Review evidence, root-only apply boundary facts, and
  related graph preview evidence without presenting related nodes as applied
  hierarchy;
- added an explicit two-step prepare/confirm apply path that sends
  `item_id`, `expected_updated_at_ms`, and an operator idempotency key to the
  Admin API while hiding the raw idempotency key from the UI;
- rendered apply result, idempotent replay, Provider Subject, Provider Mapping,
  and idempotency-key fingerprint facts;
- kept fixture mutation disabled and fixture read fallback clearly marked;
- reduced unbound Admin mock surfaces that were carrying static UI without
  durable Admin API contracts: dashboard fake quick actions/recent activity,
  DLNA mock settings, and transcode mock settings are now planned/API-scoped
  surfaces.

Red checks:

- `npm --prefix web run test -- data-source-contracts route-state-contracts`
  initially failed because the Candidate Review read/apply data-source methods
  and Web route did not exist.
- `npm --prefix web run build:budget` initially failed with total JS at
  344.12 KiB gzip over the 341 KiB budget.

Green checks:

- `npm --prefix web run check` passed.
- `npm --prefix web run test` passed: 10 test files, 116 tests.
- `npm --prefix web run build:budget` passed: `total-js` 1162.97 KiB raw /
  340.92 KiB gzip, under the 1250/341 KiB budget.
- Browser smoke passed against
  `http://127.0.0.1:3001/admin/metadata/candidate-reviews?review_id=smoke-review`:
  HTTP 200, page title loaded, key text (`Metadata Candidate Review`,
  `Root Provider Mapping`, `Related preview only`, `准备应用`) present, and no
  console errors. The only console warning was Vite/React DevTools Fast Refresh
  shim compatibility.
