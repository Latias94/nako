# Downloads / Watch-Folder Intake — Milestones

Status: Complete
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Status: completed on 2026-05-22.

Exit criteria:

- Workstream docs exist and agree.
- Scope is acquisition intake and watch-folder discovery only.
- Completed Managed Import, promotion apply, NFO sidecar apply, and playback ops
  boundaries are referenced instead of duplicated.
- Parent `post-rpd-product-hardening` points at this lane.

Primary evidence:

- `docs/workstreams/downloads-watch-folder-intake/DESIGN.md`
- `docs/workstreams/downloads-watch-folder-intake/TODO.md`

## M1 — Durable Intake Candidate Domain

Status: completed on 2026-05-22.

Exit criteria:

- [x] Core intake candidate IDs, source kinds, states, and records exist.
- [x] Repository traits support idempotent upsert/list/filter behavior.
- [x] SQLite and PostgreSQL migrations/adapters have backend-neutral contract tests.
- [x] Candidate records do not create Media Sources or imply promotion acceptance.

Primary evidence:

- `crates/nako-core`
- `crates/nako-db/src/contract_tests.rs`
- SQLite/PostgreSQL repository adapters

## M2 — App Service Intake And Managed Import Handoff

Status: completed on 2026-05-22.

Exit criteria:

- [x] App service can record/list redacted intake candidates.
- [x] Accepting a candidate creates or links a Managed Import artifact.
- [x] Repeated acceptance is idempotent.
- [x] No promotion apply, Media Source creation, or Library File Write occurs.

Primary evidence:

- `crates/nako-server/src/app`
- `crates/nako-server/src/app/tests`

## M3 — Watch-Folder Discovery

Status: completed on 2026-05-22.

Exit criteria:

- [x] Watch-folder scans use storage/VFS list/stat boundaries.
- [x] Ready, incomplete/blocked, and unsupported/blocked candidates are
  classified with stable reason categories.
- [x] Repeated scans are idempotent.
- [x] Raw host paths and credentials are not exposed in diagnostics.

Primary evidence:

- watch-folder discovery tests
- storage/VFS fixture tests

## M4 — Admin Intake Diagnostics

Status: completed on 2026-05-22.

Exit criteria:

- [x] Admin-only routes expose bounded intake diagnostics.
- [x] Admin TypeScript contract and typed client/mocks are synchronized.
- [x] Public Client API and `nako-client-protocol` remain unchanged.
- [x] Redaction tests cover raw paths, credentials, secret query strings, and
  downloader internals.

Primary evidence:

- `crates/nako-api/src/admin.rs`
- `crates/nako-server/src/http/admin.rs`
- `apps/admin-web/src/adminApi`
- `crates/nako-server/src/http/tests`

## M5 — Closeout And Follow-On Split

Status: completed on 2026-05-22.

Exit criteria:

- [x] Final gates pass with fresh evidence.
- [x] Workstream status and completed tasks are updated.
- [x] Parent post-RPD umbrella re-scores network, AI, Addon runtime, and protocol
  downloader follow-ons.
- [x] Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/downloads-watch-folder-intake/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
