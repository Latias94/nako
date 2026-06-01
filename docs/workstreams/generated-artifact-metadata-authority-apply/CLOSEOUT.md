# Generated Artifact Metadata Authority Apply - Closeout

Status: Closed
Closed: 2026-06-01
Task: GAMA-070

## Shipped

Nako now has a complete one-artifact Metadata Authority workflow for accepted
metadata Generated Artifacts:

- review acceptance remains a staging action and does not mutate Canonical
  Metadata;
- Admin apply-plan is
  `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan`
  with no request body and
  `AdminGeneratedArtifactMetadataApplyPlanResponse`;
- apply-plan is redacted, read-only, field-level, lock-aware, and proves target
  freshness without exposing raw payloads, prompts, locators, paths, or secrets;
- final Admin apply is
  `POST /admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply`
  with `AdminGeneratedArtifactMetadataApplyRequest { idempotency_key }` and
  `AdminGeneratedArtifactMetadataApplyResponse`;
- final apply revalidates the plan, respects field locks, rejects stale targets
  before mutation, writes through host-owned `MetadataApplication`, updates
  catalog/search projection, and persists durable apply outcomes;
- repeated use of the same idempotency key returns the persisted outcome as an
  idempotent replay;
- Web Admin has a separate
  `/admin/automation/generated-artifacts/metadata-apply?artifact_id=...` route
  that renders redacted plan facts, disables fixture/fallback mutation, requires
  operator preparation, and submits a stable UI idempotency key.

## Final Evidence

Fresh backend verification on 2026-06-01:

```bash
cargo nextest run -p nako-api generated_artifact_metadata_apply admin_contract --no-fail-fast
cargo nextest run -p nako-server generated_artifact_metadata_apply --no-fail-fast
cargo nextest run -p nako-db metadata_application --no-fail-fast
cargo nextest run -p nako-db generated_artifact_metadata_apply_outcome --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/generated-artifact-metadata-authority-apply/WORKSTREAM.json
git diff --check -- docs/workstreams/generated-artifact-metadata-authority-apply docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md
git diff --check
```

Results:

- `nako-api`: 7/7 passed.
- `nako-server`: 8/8 passed.
- `nako-db metadata_application`: 1/1 passed.
- `nako-db generated_artifact_metadata_apply_outcome`: 1/1 passed.
- `cargo fmt --all -- --check`: passed.
- `WORKSTREAM.json` validation: passed.
- Targeted and repository-wide diff checks: passed; Git emitted only LF/CRLF
  normalization warnings.

Fresh Web verification on 2026-06-01:

```bash
npm --prefix web run test -- src/test/data-source-contracts.test.ts src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx
npm --prefix web run check
npm --prefix web run build:budget
npm --prefix web run test -- src/test/route-state-contracts.test.tsx
npm --prefix web run test
```

Results:

- Focused data-source/route/route-state tests: 88/88 passed.
- TypeScript check: passed.
- Bundle budget: passed; `admin-route-js` 245.16 raw KiB / 53.24 gzip KiB,
  `total-js` 1121.69 raw KiB / 330.28 gzip KiB.
- Route-state test file: 29/29 passed.
- Full Web test rerun: 104/104 passed.

An initial full Web test run hit a single `Admin logs` route-state timeout that
did not reproduce on immediate focused rerun or full-suite rerun. No GAMA
behavioral failure was found.

Browser screenshots from `GAMA-060`:

- `target/gama060-apply-plan-desktop.png`
- `target/gama060-apply-plan-mobile.png`
- `target/gama060-apply-result-desktop.png`
- `target/gama060-apply-result-mobile.png`

PostgreSQL parity was not rerun in `GAMA-070` because closeout changed only
docs and no schema/PostgreSQL repository code changed after `GAMA-040`.
`GAMA-040` already recorded the local PostgreSQL ignored contract evidence for
idempotent and atomic apply outcome persistence.

## Review Result

No blocking workstream compliance findings remain:

- the target state in `DESIGN.md` is satisfied for one accepted metadata
  Generated Artifact targeting a Media Item;
- review, apply-plan, and final apply remain separate authority decisions;
- generated Admin TypeScript contracts are consumed by Web;
- evidence covers redaction, no-mutation apply-plan behavior, lock-respecting
  mutation, stale-target rejection, idempotent replay, and Web confirmation.

No blocking code-quality findings remain:

- backend mutation still flows through host-owned metadata application policy;
- durable apply outcomes preserve explicit idempotency-key replay;
- Web fixture/fallback mode renders safe facts but cannot claim final apply;
- bundle limits remain under the existing budget.

## Follow-Ons

- `proposed:generated-artifact-bulk-metadata-apply`: batch apply semantics,
  queueing, per-item idempotency, partial failure display, and operator review
  ergonomics.
- `proposed:generated-artifact-provider-mapping-breadth`: provider-specific
  mapping beyond the neutral first metadata suggestion shape.
- `proposed:generated-artifact-apply-operations-repair`: outcome audit search,
  failed/noop repair, replay diagnostics, and operator recovery tooling.
- `proposed:admin-settings-api-backed-restoration`: restore low-frequency
  Admin settings pages as real API-backed panels while preserving bundle
  budgets.

## Residual Risks

- The workflow intentionally supports one Generated Artifact at a time; bulk
  semantics need a separate durable job/control-plane design.
- Provider-specific mapping breadth is not hidden in this lane and should be
  designed against provider contracts and metadata merge policy.
- The Web Admin settings placeholders are acceptable as a budget protection
  measure only until API-backed settings lanes restore those controls.
- Full Web test had one non-reproduced timeout before passing on rerun; keep an
  eye on route-state test isolation if this recurs.
