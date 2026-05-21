# NFO Link Authority — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Lane Open

Exit criteria:

- Workstream docs exist and agree.
- First slice is bounded and non-destructive.
- Post-RPD umbrella points to this lane as active.

Primary evidence:

- `docs/workstreams/nfo-link-authority/DESIGN.md`
- `docs/workstreams/nfo-link-authority/TODO.md`

## M1 — Non-Destructive Link Planning

Exit criteria:

- `taru-vfs` has typed link plan request/result vocabulary.
- Local backend can dry-run hard/soft link eligibility.
- Unsupported backends are explicit.
- Tests prove target files are not created by planning.

Primary evidence:

- `crates/taru-vfs/src/lib.rs`
- `crates/taru-vfs/src/local.rs`

## M2 — Link Evidence Without Merge

Exit criteria:

- Filesystem link evidence can become a suggested
  `SourceDuplicateRelationship`.
- Media Sources keep library-scoped identity.
- No automatic item merge or source reassignment occurs.

Primary evidence:

- `crates/taru-server/src/app/catalog.rs`
- `crates/taru-server/src/app/tests/catalog.rs`

## M3 — NFO Authority Preview

Exit criteria:

- NFO workflow can explain sidecar decisions before mutation.
- Preview reflects local metadata policy, force/create/skip behavior, and
  backup requirement.
- Preview does not write, back up, or prune files.

Primary evidence:

- `crates/taru-nfo/src/preview.rs`
- `crates/taru-nfo/src/lib.rs`
- `crates/taru-server/src/app/nfo.rs`
- `crates/taru-server/src/app/tests/nfo.rs`

## M4 — Apply/Follow-On Decision

Exit criteria:

- Actual link creation remains deferred or has a dedicated apply design.
- Rollback, audit, redaction, and managed import dependencies are explicit.
- Follow-on lane is opened if mutation is still too broad.

Decision:

- Actual link creation is deferred.
- Follow-on should be opened after `managed-import-staging` defines promotion,
  rollback, and audit semantics.
- Do not add hardlink/symlink mutation directly to this lane.

## M5 — Closeout

Exit criteria:

- Fresh validation is recorded.
- Workstream index and post-RPD umbrella agree on the next lane.
- Residual risks and follow-ons are documented.

Closeout:

- Complete as of 2026-05-21.
- Remaining mutation work is intentionally split.
