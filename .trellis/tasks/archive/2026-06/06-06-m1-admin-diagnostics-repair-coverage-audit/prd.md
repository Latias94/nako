# M1 Admin Diagnostics Repair Coverage Audit

## Goal

Audit Admin diagnostics and repair coverage against the Product-Operator M1
quality gate, then update the current M1 queue so completed ladder evidence is
not treated as future work.

## Requirements

- Add a durable architecture matrix for M1 Admin diagnostics and repair
  coverage.
- Treat `m1-ladder-evidence-matrix` as completed evidence, not the immediate
  next candidate.
- Classify current Admin diagnostics/repair surfaces as shipped, adequate for
  M1, backend-only, read-only only, or deferred.
- Name follow-on tasks only when a concrete release ladder or operator journey
  failure would justify opening them.
- Update current roadmap, goal map, and lane routing to point at evidence-driven
  next actions.
- Do not change Rust, TypeScript, generated contracts, schema, runtime behavior,
  release scripts, or legacy workstream directories.

## Success Metrics

| Metric | Target | Measurement |
| --- | --- | --- |
| Matrix coverage | Key M1 diagnostics/repair areas audited | Review `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` |
| Queue freshness | `m1-ladder-evidence-matrix` removed from next queue and added to completed evidence | Review `docs/ROADMAP.md`, `docs/GOALS.md`, `docs/architecture/LANES.md` |
| Follow-on quality | Candidate tasks have evidence-backed opening conditions | Review follow-on routing table |
| Scope containment | Docs/Trellis only | `git status --short` |

## Alternatives Considered

### Option A: Coverage Matrix And Queue Refresh

Pros:

- Converts broad "Admin diagnostics/repair" concern into traceable evidence.
- Prevents reopening completed M1 ladder matrix work.
- Keeps future implementation tied to failed ladder evidence.

Cons:

- Does not itself add new product behavior.

Decision: chosen because the current gap is task selection and evidence
routing, not a proven missing implementation path.

### Option B: Start Media Web Browse/Player Smoke Now

Pros:

- Adds more product-facing browser evidence.

Cons:

- Current release ladder evidence did not expose a browse/player blocker.
- Would make player work speculative instead of evidence-driven.

Decision: rejected until the ladder exposes a concrete browser/player failure.

### Option C: Start A Broad Admin Repair UI Platform

Pros:

- Could unify repair commands across jobs, storage, metadata, and artwork.

Cons:

- Over-scoped for Product-Operator M1.
- Risks bypassing existing feature-owned redaction and confirmation boundaries.

Decision: rejected. Repairs should remain feature-owned unless a shared
control-plane need is proven.

## Acceptance Criteria

- [x] `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md` exists.
- [x] The matrix separates completed evidence from actionable M1 gaps.
- [x] `docs/ROADMAP.md` lists `m1-ladder-evidence-matrix` as completed
      evidence and no longer names it as next work.
- [x] `docs/GOALS.md` records the post-matrix queue state.
- [x] `docs/architecture/LANES.md` routes next work through the Admin
      diagnostics/repair coverage matrix and ladder failures.
- [x] Trellis context validation passes.
- [x] `git diff --check` passes for touched docs and task files.

## Technical Notes

Relevant context:

- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
- `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
- `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`

No implementation code changes are in scope.
