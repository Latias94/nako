# Fearless Architecture Deepening — Evidence And Gates

Status: Active
Last updated: 2026-05-20

This file records evidence for M63. Do not claim the fearless refactor lane is
complete without fresh command evidence matching the touched Interfaces.

## Gate Policy

Always-on gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run` for touched crates and behavior families
- `git diff --check`

Closeout gates:

- `cargo nextest run --workspace --no-fail-fast`
- PostgreSQL opt-in contract runs for any touched persistence seam when
  `TARU_TEST_POSTGRES_URL` is available.

PostgreSQL opt-in policy:

- SQLite remains always-on.
- PostgreSQL contracts must fail fast when `TARU_TEST_POSTGRES_URL` is absent
  rather than reporting false green evidence.
- New persistence commit seams must have backend-neutral contracts unless the
  workstream explicitly splits a follow-on and gates runtime exposure.

Safety gates:

- No Addon Side Effect refactor may expose raw Addon Tokens, Source Locators,
  storage URIs, local paths, cache URIs, raw source URLs, content hashes,
  database URLs, credentials, or raw database errors in public/admin/addon DTOs.
- No NFO/Library File Write change may bypass Taru-owned VFS write policy,
  backup policy, permission checks, or audit/apply outcome recording.
- No AI/vector/search change may overwrite Canonical Metadata without the
  Generated Artifact and Acceptance Workflow authority described in
  `CONTEXT.md`.
- Reference repositories under `repo-ref/` remain study material only.

## Evidence

### 2026-05-20 — FAD-010 Workstream Opened

Status: complete.

Evidence:

- Created `docs/workstreams/fearless-architecture-deepening/`.
- Recorded architecture review findings after M62 PostgreSQL Production
  Readiness closeout.
- Selected FAD-020 Addon Side Effect Module depth as the first executable task.
- Documented non-goals to keep provider breadth, network traversal, native
  plugin ABI, adaptive bitrate, Managed Artwork PostgreSQL parity, and AI
  runtime out of this lane unless explicitly split back in.

Validation:

```bash
git diff --check
```

Result:

- `git diff --check` passed later during the FAD-020 verification pass with Git
  CRLF normalization warnings only.

### 2026-05-20 — FAD-020 Addon Side Effect Module Depth

Status: complete.

Implementation evidence:

- Kept `crates/taru-server/src/app/addons.rs` as the root
  `AddonAppService` Module for addon registration, token lifecycle, and grant
  administration.
- Split Addon Principal and grant normalization into
  `crates/taru-server/src/app/addons/principal.rs`.
- Split Addon Side Effect intake, idempotency, safe validation error mapping,
  and authority/target validation into
  `crates/taru-server/src/app/addons/intake.rs`.
- Added an Addon Side Effect apply router in
  `crates/taru-server/src/app/addons/side_effect_apply.rs`.
- Split domain-specific apply Adapters:
  - `metadata_write.rs` for Canonical Metadata patch/merge plus existing
    catalog/search refresh behavior;
  - `library_file_write.rs` for NFO Library File Write export through the
    existing VFS/NFO service and backup policy;
  - `artwork_write.rs` for Addon Artwork Candidate proposal;
  - `target.rs` for shared Media Item resolution from side-effect targets.
- No public/admin/addon DTO shape was changed.
- No persistence schema, repository contract, or behavior semantics were
  changed in this slice.

Validation:

```bash
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo fmt --all
cargo check -p taru-server --tests
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo nextest run -p taru-server addon --no-fail-fast
cargo fmt --all -- --check
cargo check -p taru-server --tests
git diff --check
```

Result:

- Baseline focused Addon Side Effect nextest passed before the refactor:
  10 passed, 165 skipped.
- `cargo fmt --all` passed after the refactor.
- `cargo check -p taru-server --tests` passed after the refactor.
- Focused Addon Side Effect nextest passed after the refactor:
  10 passed, 165 skipped.
- Broader addon HTTP nextest passed after the refactor:
  31 passed, 144 skipped.
- `cargo fmt --all -- --check` passed.
- Final `cargo check -p taru-server --tests` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

Broader gates not run:

- Full workspace nextest was not run for FAD-020 because this task is a
  behavior-preserving server Module split, the touched runtime surface is
  covered by focused Addon Side Effect and broader addon HTTP tests, and no
  public API, repository, or persistence contract changed.
- PostgreSQL opt-in contracts were not applicable for FAD-020 because no
  persistence seam or SQL behavior changed. FAD-030 will require DB contract
  evidence if it introduces a transactional Addon metadata commit seam.

## Evidence To Add During Execution

Each task should add:

- command line used;
- result summary;
- touched Interface or Module;
- whether PostgreSQL opt-in evidence was run, skipped, or not applicable;
- remaining risks and split decisions.
