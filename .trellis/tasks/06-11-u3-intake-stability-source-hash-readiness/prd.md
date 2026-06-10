# U3 Intake Stability And Source Hash Readiness First Slice

## Goal

Turn the next media-server maturity step into one day of shippable backend work:
make watcher-driven intake stability and scan-originated Source Fingerprint hash
readiness easier to reason about, test, and operate without widening into a
full watcher/scheduler productization rewrite.

## What I Already Know

- U1 operator readiness overview and U2 playback selected reasons have already
  shipped on branch `feat/operator-readiness-u1`.
- The roadmap's U3 lane targets watcher/intake/scheduler productization:
  stable-candidate observation, scan admission, source hash triggering,
  duplicate suggestion, and repair diagnostics.
- `nako-library::intake` already owns stable candidate evidence and pure intake
  planning.
- `nako-server::app::watch_folder_runtime` already discovers watch-folder
  candidates and enqueues library scans only when the intake plan says so.
- `nako-server::app::source_hash` already owns scan-originated Source
  Fingerprint hash enqueue through a redaction-safe durable job boundary.
- Public and Admin diagnostics must not expose Source Locators, local paths,
  backend URLs, credentials, raw fingerprints, raw hashes, or job input JSON.

## Requirements

- Preserve Nako vocabulary: Media Library, Media Source, Source Locator, Source
  Fingerprint, Source Duplicate Relationship, and operator readiness.
- Keep watcher observations lightweight until stability is proven.
- Keep scan planning and source observation planning pure; no VFS hash reads or
  durable queue writes may happen inside library planning.
- Keep scan-originated Source Fingerprint hash enqueue behind the existing
  server app service and durable job resource class.
- Improve the day-one slice around explicit state/reason/evidence contracts,
  tests, and redaction, not a new long-running runtime architecture.
- Prefer deleting or consolidating shallow helper code when the existing
  interface can become deeper and more testable.

## Acceptance Criteria

- [ ] A file observed once remains inspecting and does not trigger a scan job.
- [ ] A repeated unchanged watch-folder observation becomes ready and enqueues
      at most one scan admission for the newly ready candidate.
- [ ] A changed observation resets stability, representing copy-in-progress or
      still-mutating objects without early ingest.
- [ ] A committed Media Source with a full/partial Source Fingerprint
      escalation can be traced to safe durable source-hash enqueue behavior.
- [ ] Operator/Admin evidence remains redaction-safe: no Source Locator, local
      path, backend URL, credential, raw fingerprint, raw hash, or job input JSON
      appears in exposed diagnostics.
- [ ] Focused tests cover the changed contracts.

## Technical Approach

First inspect the current intake/watch-folder and source-hash/Admin readiness
flows. Then choose the smallest vertical slice that deepens an existing module
interface rather than adding a parallel path. Likely candidates:

- strengthen `nako-library::intake` state/reason summaries so runtime code does
  less ad hoc counting;
- strengthen watch-folder runtime or acquisition-intake tests around stable
  observation reset and duplicate scan admission;
- add a focused readiness/source-hash regression where scan-originated enqueue
  evidence remains safe and operator-readable.

## Decision (ADR-lite)

**Context**: U3 is broad enough to become a multi-day watcher, scheduler, and
repair productization effort. The current session is constrained to one day and
should produce a reviewable commit.

**Decision**: Build a first slice around stable intake and Source Fingerprint
hash readiness contracts. Do not introduce a new watcher daemon, new storage
backend behavior, automatic duplicate reconciliation, or recurring repair
scheduler in this task.

**Consequences**: The result improves confidence and operability around the
existing pipeline while leaving broader productization follow-ons explicit.

## Out Of Scope

- Automatic Source Duplicate Relationship reconciliation.
- New schema migrations.
- New Public Client API surfaces.
- New raw `tokio::spawn` background work outside ADR 0053 runtime supervision.
- Cache purge/delete/invalidation or backend configuration mutation.
- Full Admin Web UX for U3.

## Definition Of Done

- Relevant Rust code is formatted with `cargo fmt --all`.
- Focused nextest gates pass for changed packages/tests.
- Trellis implement/check context is curated.
- Any durable architecture/spec lesson is either already represented or added to
  the appropriate spec before finish.
- Work is committed with a Conventional Commit.

## Technical Notes

- Roadmap authority:
  `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`.
- Library authority:
  `.trellis/spec/nako-library/backend/index.md`,
  `.trellis/spec/nako-library/backend/directory-structure.md`,
  `.trellis/spec/nako-library/backend/quality-guidelines.md`,
  `.trellis/spec/nako-library/backend/error-handling.md`,
  `.trellis/spec/nako-library/backend/logging-guidelines.md`,
  `docs/architecture/LIBRARY_PIPELINE.md`.
- Server/source-hash authority:
  `.trellis/spec/nako-server/backend/index.md`,
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`,
  `docs/architecture/CONTROL_PLANE.md`,
  `docs/architecture/STORAGE_VFS.md`.
- Read-only subagent audit lanes:
  - Volta: intake/watch-folder stability and diagnostics.
  - Nash: Source Fingerprint hash and Admin readiness.
