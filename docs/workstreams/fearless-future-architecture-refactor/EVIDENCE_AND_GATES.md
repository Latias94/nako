# Fearless Future Architecture Refactor — Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Smallest Current Repro

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

Use these while splitting the next module hot spot:

```bash
cargo check -p nako-server --tests
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server managed_import --no-fail-fast
cargo nextest run -p nako-server addons --no-fail-fast
cargo nextest run -p nako-server metadata --no-fail-fast
```

### Persistence Gate

Use these when touching `nako-db` or any backend contract family:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db job --no-fail-fast
cargo nextest run -p nako-db job_lease --no-fail-fast
cargo nextest run -p nako-db event_outbox_and_webhook_delivery --no-fail-fast
cargo nextest run -p nako-db event_addon_automation --no-fail-fast
cargo nextest run -p nako-db vfs_staging --no-fail-fast
cargo nextest run -p nako-db managed_artwork --no-fail-fast
cargo nextest run -p nako-db addon_artwork_candidate_intake --no-fail-fast
cargo nextest run -p nako-db managed_import --no-fail-fast
cargo nextest run -p nako-db acquisition_intake --no-fail-fast
cargo nextest run -p nako-db nfo_sidecar_apply --no-fail-fast
cargo nextest run -p nako-db metadata_catalog --no-fail-fast
cargo nextest run -p nako-db catalog_governance --no-fail-fast
cargo nextest run -p nako-db provider_mapping --no-fail-fast
cargo nextest run -p nako-db playback_runtime --no-fail-fast
cargo nextest run -p nako-db transcode_session --no-fail-fast
cargo nextest run -p nako-db library --no-fail-fast
cargo nextest run -p nako-db scan_commit --no-fail-fast
cargo nextest run -p nako-db catalog_governance --no-fail-fast
cargo nextest run -p nako-db ingestion_failure --no-fail-fast
cargo nextest run -p nako-db --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
```

### API Gate

Use these when splitting `nako-api` DTO surfaces:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api admin_contract --no-fail-fast
```

### VFS And Inference Gate

Use these when splitting file authority or local inference:

```bash
cargo check -p nako-vfs --tests
cargo check -p nako-library --tests
```

### Docker And Container Gate

Use the container gate when a slice touches startup, deployment, or runtime
packaging:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container
```

The container gate covers the Docker Compose config checks for both SQLite and
PostgreSQL stacks. When a slice needs a full runtime smoke, run the explicit
compose commands from `docs/deployment/SELF_HOSTED.md` in an isolated local
environment.

### Closeout Gate

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Review Gate

Run `review-workstream` before accepting a task or lane completion. Record
blocking findings, missing gates, and residual risks in `HANDOFF.md`.

## Evidence Anchors

- `docs/workstreams/fearless-future-architecture-refactor/DESIGN.md`
- `docs/workstreams/fearless-future-architecture-refactor/TODO.md`
- `docs/workstreams/fearless-future-architecture-refactor/MILESTONES.md`
- `docs/workstreams/fearless-future-architecture-refactor/HANDOFF.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-010.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-020.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-021.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030a.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030b.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030c.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030d.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030e.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030f.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030g.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030h.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-030i.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040a.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040b.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040c.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040d.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040e.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-040f.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050a.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050b.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050c.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050d.md`
- `docs/workstreams/fearless-future-architecture-refactor/JOURNAL/2026-05-23-ffr-050e.md`
- `repo-ref/jellyfin/README.md`
- `crates/nako-db/src/postgres.rs`
- `crates/nako-db/src/postgres/addons_automation.rs`
- `crates/nako-db/src/postgres/jobs.rs`
- `crates/nako-db/src/postgres/events.rs`
- `crates/nako-db/src/postgres/import_state.rs`
- `crates/nako-db/src/postgres/managed_artwork.rs`
- `crates/nako-db/src/postgres/metadata_catalog.rs`
- `crates/nako-db/src/postgres/core_catalog.rs`
- `crates/nako-db/src/postgres/playback_runtime.rs`
- `crates/nako-db/src/postgres/vfs_staging.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/staging_policy.rs`
- `crates/nako-server/src/app/playback/selection.rs`
- `crates/nako-server/src/app/playback/failure.rs`
- `crates/nako-server/src/app/playback/events.rs`
- `crates/nako-server/src/app/playback/paths.rs`
- `crates/nako-server/src/app/playback/playlist.rs`
- `crates/nako-server/src/app/managed_import.rs`
- `crates/nako-server/src/app/managed_import/diagnostics.rs`
- `crates/nako-server/src/app/managed_import/outcomes.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/metadata.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-api/src/admin/catalog_governance.rs`
- `crates/nako-api/src/admin/intake.rs`
- `crates/nako-api/src/admin/network.rs`
- `crates/nako-api/src/admin/operations.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-api/src/admin/storage.rs`
- `crates/nako-vfs/src/local.rs`
- `crates/nako-vfs/src/local/path_authority.rs`
- `crates/nako-vfs/src/local/write_transaction.rs`
- `crates/nako-vfs/src/local/apply_plan.rs`
- `crates/nako-vfs/src/local/lifecycle.rs`
- `crates/nako-naming/src/lib.rs`
- `crates/nako-library/src/local_inference.rs`

## Recorded Evidence

### FFR-020 Playback Runtime Split

Commands:

```bash
cargo check -p nako-server --tests
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --package nako-server
```

Result:

- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server playback --no-fail-fast` passed with 54
  tests run and 216 skipped.
- `cargo fmt --package nako-server` completed.

What this proves:

- The playback runtime split preserved direct play, remux, HLS, playback
  session, diagnostics, failure taxonomy, and support-evidence behavior covered
  by the focused playback test family.

### FFR-021 Managed Import Runtime Split

Commands:

```bash
cargo check -p nako-server --tests
cargo nextest run -p nako-server managed_import --no-fail-fast
cargo fmt --package nako-server
```

Result:

- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-server managed_import --no-fail-fast` passed with
  19 tests run and 251 skipped.
- `cargo fmt --package nako-server` completed.

What this proves:

- The managed import split preserved artifact diagnostics redaction, promotion
  preview blocking, idempotent acceptance, storage apply, catalog commit,
  cleanup, and failure outcome behavior covered by the focused managed import
  test family.

### FFR-030A PostgreSQL Job Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db job_lease --no-fail-fast
cargo nextest run -p nako-db job --no-fail-fast
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db job_lease --no-fail-fast` passed with 4 tests
  run and 125 skipped.
- `cargo nextest run -p nako-db job --no-fail-fast` passed with 9 tests run
  and 120 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `NAKO_TEST_POSTGRES_URL` was unset, so ignored PostgreSQL opt-in contracts
  were not run in this slice.

What this proves:

- The PostgreSQL job and job lease repository implementation, SQL select
  fragments, row mapping, lease validation, stale-lease error mapping, and
  managed artwork job transaction helpers now live in `postgres/jobs.rs`.
- The backend-neutral SQLite job/job-lease contract family still passes,
  proving the shared repository contract remains intact while PostgreSQL
  remains opt-in.

### FFR-030B PostgreSQL Event/Webhook Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db event_outbox_and_webhook_delivery --no-fail-fast
cargo nextest run -p nako-db event_addon_automation --no-fail-fast
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db event_outbox_and_webhook_delivery
  --no-fail-fast` passed with 1 test run and 128 skipped.
- `cargo nextest run -p nako-db event_addon_automation --no-fail-fast`
  passed with 4 tests run and 125 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `NAKO_TEST_POSTGRES_URL` was unset, so ignored PostgreSQL opt-in contracts
  were not run in this slice.

What this proves:

- PostgreSQL event outbox and webhook endpoint/delivery persistence now live in
  `postgres/events.rs` with their SQL select fragments, row mapping, event
  subject decoding, and delivery-attempt lookup helper.
- The backend-neutral event/addon/automation contract family still passes
  after moving the event/webhook slice.

### FFR-030C PostgreSQL VFS/Staging Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db vfs_staging --no-fail-fast
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db vfs_staging --no-fail-fast` passed with 2
  tests run and 127 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `NAKO_TEST_POSTGRES_URL` was unset, so ignored PostgreSQL opt-in contracts
  were not run in this slice.

What this proves:

- PostgreSQL VFS cache and staging manifest persistence now lives in
  `postgres/vfs_staging.rs` with its SQL select fragments, row mapping,
  transactional listing upserts, staging reservation budget accounting, and
  lease state transitions.
- The backend-neutral VFS/staging contract family still passes after moving the
  PostgreSQL implementation out of the broad `postgres.rs` backend.

### FFR-030D PostgreSQL Addon/Automation Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db event_addon_automation --no-fail-fast
cargo nextest run -p nako-db addon_artwork_candidate_intake --no-fail-fast
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db event_addon_automation --no-fail-fast` passed
  with 4 tests run and 125 skipped.
- `cargo nextest run -p nako-db addon_artwork_candidate_intake --no-fail-fast`
  passed with 1 test run and 128 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.
- `NAKO_TEST_POSTGRES_URL` was unset, so ignored PostgreSQL opt-in contracts
  were not run in this slice.

What this proves:

- PostgreSQL addon registration, token, grant, routing plan, side-effect, and
  automation provider/artifact persistence now lives in
  `postgres/addons_automation.rs`.
- The module owns its SQL select fragments, row mapping, generated artifact
  proposal hydration, and the side-effect apply-outcome transaction helper used
  by metadata commit.
- Backend-neutral addon/automation contracts and the managed artwork addon
  candidate intake contract still pass after moving the implementation out of
  the broad `postgres.rs` backend.

### FFR-030E PostgreSQL Managed Artwork Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db managed_artwork --no-fail-fast
cargo nextest run -p nako-db addon_artwork_candidate_intake --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db managed_artwork --no-fail-fast` passed with
  12 tests run and 117 skipped.
- `cargo nextest run -p nako-db addon_artwork_candidate_intake --no-fail-fast`
  passed with 1 test run and 128 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite managed-artwork` passed with 6
  ignored PostgreSQL contract tests run and 123 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- PostgreSQL artwork task, candidate, managed ingest, managed artifact,
  selected artwork, gallery, and lifecycle cleanup persistence now live in
  `postgres/managed_artwork.rs`.
- The module owns its SQL select fragments, transaction-heavy ingest and
  publication flows, row mapping, and direct use of the job transaction helper
  boundary.
- Backend-neutral managed artwork tests and ignored PostgreSQL managed artwork
  contracts still pass after moving the implementation out of the broad
  `postgres.rs` backend.

### FFR-030F PostgreSQL Import-State Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db managed_import --no-fail-fast
cargo nextest run -p nako-db acquisition_intake --no-fail-fast
cargo nextest run -p nako-db nfo_sidecar_apply --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db managed_import --no-fail-fast` passed with 1
  test run and 128 skipped.
- `cargo nextest run -p nako-db acquisition_intake --no-fail-fast` passed with
  1 test run and 128 skipped.
- `cargo nextest run -p nako-db nfo_sidecar_apply --no-fail-fast` passed with
  1 test run and 128 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts` passed with 30
  ignored PostgreSQL contract tests run and 99 skipped.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- PostgreSQL managed import artifact, managed import promotion apply,
  acquisition intake candidate, and NFO sidecar apply persistence now live in
  `postgres/import_state.rs`.
- The module owns its SQL select fragments, state transitions, row mapping,
  and source-kind codecs for the import-state family.
- Backend-neutral focused contracts and the full ignored PostgreSQL contract
  harness still pass after moving the implementation out of the broad
  `postgres.rs` backend.

### FFR-030G PostgreSQL Metadata/Catalog Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db metadata_catalog --no-fail-fast
cargo nextest run -p nako-db catalog_governance --no-fail-fast
cargo nextest run -p nako-db provider_mapping --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
cargo fmt --package nako-db -- --check
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db metadata_catalog --no-fail-fast` passed with
  3 tests run and 126 skipped.
- `cargo nextest run -p nako-db catalog_governance --no-fail-fast` passed with
  1 test run and 128 skipped.
- `cargo nextest run -p nako-db provider_mapping --no-fail-fast` passed with 2
  tests run and 127 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts` passed with 30
  ignored PostgreSQL contract tests run and 99 skipped.
- `cargo fmt --package nako-db -- --check` passed.
- `python -m json.tool .../WORKSTREAM.json > $null` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- PostgreSQL provider mapping, metadata commit, and catalog graph persistence
  now live in `postgres/metadata_catalog.rs`.
- The module owns provider subject/raw response/attempt persistence, metadata
  field locks, metadata refresh commits, NFO import commits, addon metadata
  write commits, catalog graph replacement, catalog entities, image assets,
  row mapping, and related provider/source/image codecs.
- Shared low-level media item, library item state, search projection, and
  external-id lookup helpers remain in `postgres.rs` until FFR-030H decides
  whether they are stable infrastructure or should move into another backend
  family.

### FFR-030H PostgreSQL Playback Runtime Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db playback_runtime --no-fail-fast
cargo nextest run -p nako-db transcode_session --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db playback_runtime --no-fail-fast` passed with 2
  tests run and 127 skipped.
- `cargo nextest run -p nako-db transcode_session --no-fail-fast` passed with
  4 tests run and 125 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts` passed with 30
  ignored PostgreSQL contract tests run and 99 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- PostgreSQL user playback state and transcode session persistence now live in
  `postgres/playback_runtime.rs`.
- The module owns the playback/transcode SQL select fragments, row mapping,
  runtime-state codecs, cancellation/stale-state updates, and focused
  playback runtime behavior.
- The explicit FFR-030H judgment is to continue FFR-030 with one more
  meaningful persistence split: core library/media/scan/search, local
  inference, ingestion failure, source duplicate, and catalog governance.
  After that split, `postgres.rs` should be reassessed for handoff to FFR-040.

### FFR-030I PostgreSQL Core Catalog Backend Split

Commands:

```bash
cargo check -p nako-db --tests
cargo nextest run -p nako-db library --no-fail-fast
cargo nextest run -p nako-db scan_commit --no-fail-fast
cargo nextest run -p nako-db catalog_governance --no-fail-fast
cargo nextest run -p nako-db ingestion_failure --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
cargo fmt --package nako-db
cargo fmt --package nako-db -- --check
git diff --check
```

Result:

- `cargo check -p nako-db --tests` passed.
- `cargo nextest run -p nako-db library --no-fail-fast` passed with 8 tests
  run and 121 skipped.
- `cargo nextest run -p nako-db scan_commit --no-fail-fast` passed with 2
  tests run and 127 skipped.
- `cargo nextest run -p nako-db catalog_governance --no-fail-fast` passed with
  1 test run and 128 skipped.
- `cargo nextest run -p nako-db ingestion_failure --no-fail-fast` passed with
  1 test run and 128 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts` passed with 30
  ignored PostgreSQL contract tests run and 99 skipped.
- `cargo fmt --package nako-db` completed.
- `cargo fmt --package nako-db -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- PostgreSQL library, library-item, media, media-probe, local inference,
  ingestion failure, scan, search, source duplicate, and catalog governance
  persistence now live in `postgres/core_catalog.rs`.
- The module owns the shared media/source/search transaction helpers and the
  row mapping/codec family it needs, while `postgres.rs` now stays focused on
  connection, migration, schema validation, and dispatch.
- The persistence half of FFR-030 is now at its intended handoff point and
  should not be split further unless a new wide backend family emerges.

### FFR-040A API Admin Playback And Network Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api admin_playback --no-fail-fast
cargo nextest run -p nako-api network --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api admin_playback --no-fail-fast` passed with 4
  tests run and 49 skipped.
- `cargo nextest run -p nako-api network --no-fail-fast` passed with 2 tests
  run and 52 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 49 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 45 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 54 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Admin playback DTOs, runtime diagnostics, support evidence, redaction
  helpers, and playback redaction tests now live in
  `crates/nako-api/src/admin/playback.rs`.
- Admin network access diagnostics, readiness aggregation, tunnel provider
  summaries, and network redaction tests now live in
  `crates/nako-api/src/admin/network.rs`.
- The generated admin contract and public client tests still pass, so moving
  these admin surfaces did not leak admin routes into the public API or change
  generated contract behavior.
- `FFR-040` remains active because `admin.rs` still contains several unrelated
  admin API surfaces that should be split by ownership and redaction locality.

### FFR-040B API Admin Storage Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api storage --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api storage --no-fail-fast` passed with 11 tests
  run and 44 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 50 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 46 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 55 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Admin storage staging diagnostics, VFS cache summary DTOs, storage backend
  diagnostics, runtime state scope, and staging record conversion now live in
  `crates/nako-api/src/admin/storage.rs`.
- The moved storage tests still prove staging diagnostics expose source scheme
  and summary booleans without serializing raw source URIs, local paths, etags,
  fingerprints, or validation errors.
- The added storage backend diagnostics test keeps runtime state summarized
  and guards against leaking credential-bearing raw storage references.
- Admin contract and public client tests still pass, proving the API surface
  split did not leak admin internals into public API artifacts.

### FFR-040C API Admin Automation Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api generated_artifact --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api generated_artifact --no-fail-fast` passed
  with 2 tests run and 53 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 50 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 46 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 55 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Generated artifact proposal, review, acceptance plan, target, provenance,
  payload summary, and readiness DTOs now live in
  `crates/nako-api/src/admin/automation.rs`.
- The moved automation tests still prove admin automation responses expose
  summaries and metadata-authority boundary facts without raw prompt JSON, raw
  artifact JSON, raw locators, source fingerprints, secret environment values,
  or generic raw payload fields.
- Admin contract and public client tests still pass, proving the generated
  admin TypeScript contract and public API inventory remain consistent after
  the module split.

### FFR-040D API Admin Intake Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api intake --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api intake --no-fail-fast` passed with 2 tests
  run and 53 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 50 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 46 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 55 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Acquisition intake candidate diagnostics and watch-folder discovery DTOs now
  live in `crates/nako-api/src/admin/intake.rs`.
- The moved intake tests prove the admin intake API exposes redacted source
  references, schemes, candidate state, and safe discovery failures without
  raw source URIs, display names, intended locators, diagnostics JSON, root
  URIs, local paths, or token-bearing values.
- Admin contract and public client tests still pass, proving this split did
  not leak intake diagnostics into public API artifacts.

### FFR-040E API Admin Operations Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api job --no-fail-fast
cargo nextest run -p nako-api outbox --no-fail-fast
cargo nextest run -p nako-api ingestion_failure --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api job --no-fail-fast` passed with 4 tests run
  and 51 skipped.
- `cargo nextest run -p nako-api outbox --no-fail-fast` passed with 1 test run
  and 54 skipped.
- `cargo nextest run -p nako-api ingestion_failure --no-fail-fast` passed with
  1 test run and 54 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 50 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 46 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 55 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Job, job cancellation, outbox event, ingestion failure, and
  ignore-ingestion-failure DTOs now live in
  `crates/nako-api/src/admin/operations.rs`.
- The operations tests prove job input JSON, job summary JSON, raw errors,
  outbox payload JSON, idempotency keys, and last-error values remain hidden
  behind explicit booleans and typed operational fields.
- Admin contract and public client tests still pass, proving the API module
  split preserved generated admin contracts and public route separation.

### FFR-040F API Admin Catalog Governance Surface Split

Commands:

```bash
cargo check -p nako-api --tests
cargo nextest run -p nako-api catalog_governance --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-api public_client --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo fmt --package nako-api
cargo fmt --package nako-api -- --check
git diff --check
```

Result:

- `cargo check -p nako-api --tests` passed.
- `cargo nextest run -p nako-api catalog_governance --no-fail-fast` passed
  with 1 test run and 54 skipped.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed with 5
  tests run and 50 skipped.
- `cargo nextest run -p nako-api public_client --no-fail-fast` passed with 9
  tests run and 46 skipped.
- `cargo nextest run -p nako-api --no-fail-fast` passed with 55 tests run.
- `cargo fmt --package nako-api` completed.
- `cargo fmt --package nako-api -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Catalog governance item list, item summary, local inference summary, and
  governance issue DTOs now live in
  `crates/nako-api/src/admin/catalog_governance.rs`.
- The catalog governance test proves raw **Local Inference Evidence** values
  and local paths stay hidden while the admin API still exposes safe
  confidence, issue, source, and inference-version summaries.
- After FFR-040F, `admin.rs` only owns config diagnostics and overview summary
  aggregates. FFR-040 is ready for review before moving to FFR-050.

### FFR-050A VFS Local Path Authority Split

Commands:

```bash
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs local --no-fail-fast
cargo nextest run -p nako-vfs --no-fail-fast
cargo fmt --package nako-vfs
cargo fmt --package nako-vfs -- --check
git diff --check
```

Result:

- `cargo check -p nako-vfs --tests` passed.
- `cargo nextest run -p nako-vfs local --no-fail-fast` passed with 36 tests
  run and 11 skipped.
- `cargo nextest run -p nako-vfs --no-fail-fast` passed with 47 tests run.
- `cargo fmt --package nako-vfs` completed.
- `cargo fmt --package nako-vfs -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Local root canonicalization, local scheme checks, relative path parsing,
  read/write/cleanup path resolution, local URI construction, backup URI
  construction, and security-violation classification now live in
  `crates/nako-vfs/src/local/path_authority.rs`.
- The existing local backend tests still prove read, write, atomic replace,
  backup pruning, link planning/apply, cleanup, restore, and staging behavior
  after the split.
- The new path-authority tests cover parent-directory traversal rejection and
  local URI separator normalization through the extracted module.

### FFR-050B VFS Local Write Transaction Split

Commands:

```bash
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo fmt --package nako-vfs
cargo fmt --package nako-vfs -- --check
git diff --check
```

Result:

- `cargo check -p nako-vfs --tests` passed.
- `cargo nextest run -p nako-vfs --no-fail-fast` passed with 47 tests run.
- `cargo fmt --package nako-vfs` completed.
- `cargo fmt --package nako-vfs -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Local atomic replace, backup creation, backup retention pruning, restore
  temp-file handling, file sync, directory sync, and backup sidecar naming now
  live in `crates/nako-vfs/src/local/write_transaction.rs`.
- `LocalFsBackend` still resolves safe paths and maps storage reports, while
  the local write transaction details are isolated for future
  library-file-write policy work.
- The existing VFS test suite still passes, proving local read, write,
  atomic replace, backup, restore, link/copy, cleanup, WebDAV, and cache
  behavior stayed intact.

### FFR-050C VFS Local Apply/Link Planning Split

Commands:

```bash
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs local_backend --no-fail-fast
cargo nextest run -p nako-vfs --no-fail-fast
cargo fmt --package nako-vfs
cargo fmt --package nako-vfs -- --check
git diff --check
```

Result:

- `cargo check -p nako-vfs --tests` passed.
- `cargo nextest run -p nako-vfs local_backend --no-fail-fast` passed with 28
  tests run and 19 skipped.
- `cargo nextest run -p nako-vfs --no-fail-fast` passed with 47 tests run.
- `cargo fmt --package nako-vfs` completed.
- `cargo fmt --package nako-vfs -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Local link planning, copy apply, hardlink/symlink apply, apply status
  mapping, and create-new copy/symlink file actions now live in
  `crates/nako-vfs/src/local/apply_plan.rs`.
- `LocalFsBackend` now delegates the `plan_link` and `apply` storage trait
  entrypoints to the apply-plan module, while path resolution still stays in
  `path_authority.rs` and atomic write/backup/restore mechanics stay in
  `write_transaction.rs`.
- The focused local backend and full VFS suites still pass, proving local
  copy, hardlink, symlink, security-violation, cleanup, restore, WebDAV, and
  cache behavior stayed intact after the split.

### FFR-050D VFS Local Lifecycle Split

Commands:

```bash
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs local_backend --no-fail-fast
cargo nextest run -p nako-vfs --no-fail-fast
cargo fmt --package nako-vfs
cargo fmt --package nako-vfs -- --check
git diff --check
```

Result:

- `cargo check -p nako-vfs --tests` passed.
- `cargo nextest run -p nako-vfs local_backend --no-fail-fast` passed with 28
  tests run and 19 skipped.
- `cargo nextest run -p nako-vfs --no-fail-fast` passed with 47 tests run.
- `cargo fmt --package nako-vfs` completed.
- `cargo fmt --package nako-vfs -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- Local cleanup, restore, lifecycle request validation, lifecycle status
  mapping, and cleanup/restore report construction now live in
  `crates/nako-vfs/src/local/lifecycle.rs`.
- `LocalFsBackend` now delegates `cleanup` and `restore` storage trait
  entrypoints to the lifecycle module, while path authority, apply/link
  planning, and write transaction mechanics remain in their focused modules.
- The focused local backend and full VFS suites still pass, proving local
  cleanup, restore, backup, copy/link apply, read/write, staging, WebDAV, and
  cache behavior stayed intact after the split.

### FFR-050E Naming And Local Inference Boundary Split

Commands:

```bash
cargo check -p nako-naming --tests
cargo check -p nako-library --tests
cargo nextest run -p nako-naming --no-fail-fast
cargo nextest run -p nako-library local_inference --no-fail-fast
cargo nextest run -p nako-library --no-fail-fast
cargo tree -p nako-naming --depth 1
cargo fmt --package nako-naming --package nako-library
cargo fmt --package nako-naming --package nako-library -- --check
git diff --check
```

Result:

- `cargo check -p nako-naming --tests` passed.
- `cargo check -p nako-library --tests` passed.
- `cargo nextest run -p nako-naming --no-fail-fast` passed with 6 tests run.
- `cargo nextest run -p nako-library local_inference --no-fail-fast` passed
  with 5 tests run and 14 skipped.
- `cargo nextest run -p nako-library --no-fail-fast` passed with 19 tests run.
- `cargo tree -p nako-naming --depth 1` showed only `serde` as a direct
  dependency.
- `cargo fmt --package nako-naming --package nako-library` completed.
- `cargo fmt --package nako-naming --package nako-library -- --check` passed.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- `nako-naming` no longer depends on `nako-core`. It now exposes parser-local
  `ParsedMediaKind` and `NameEvidenceSource` values instead of catalog-domain
  `MediaKind` and `LocalInferenceEvidenceSource` values.
- `nako-library/src/local_inference.rs` now owns the mapping from parsed-name
  output into Nako `MediaKind` and `LocalInferenceEvidenceSource`, including a
  boundary test with a custom `NameParser`.
- Existing naming, local inference, library indexing, scan, WebDAV, and probe
  tests still pass after the dependency inversion.
- FFR-050 review found no blocking findings. The remaining
  `local_inference.rs` width is an internal follow-up candidate, not a blocker
  for moving to FFR-060.

### FFR-060 Closeout Validation And Deletion Sweep

Commands:

```bash
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts
python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null
git diff --check
```

Result:

- Deletion/duplication sweep found no remaining replaced helper paths requiring
  immediate removal. The old local VFS helper names now exist only in focused
  replacement modules or tests.
- `cargo fmt --all -- --check` passed.
- `cargo check --workspace --tests` passed.
- `cargo nextest run --workspace --no-fail-fast` passed with 696 tests run
  and 30 skipped.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1
  -Mode container` passed. It ran formatting, `git diff --check`, redaction
  inventory scan, `cargo nextest run -p nako-server config --no-fail-fast`,
  and Docker Compose config checks for SQLite and PostgreSQL stacks.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File
  scripts/postgres-contract-harness.ps1 -Suite all-contracts` passed. The
  harness initialized a local PostgreSQL 17 cluster, ran 30 ignored-only
  PostgreSQL contract tests, and stopped/removed the temporary data directory.
- `git diff --check` passed with Git CRLF normalization warnings only.

What this proves:

- The refactor lane's workspace-wide Rust behavior still passes after the
  server, persistence, API, VFS, and naming/inference boundary splits.
- Docker-backed self-hosted config validation is usable for both SQLite and
  PostgreSQL Compose stacks.
- PostgreSQL opt-in contracts pass against a real local PostgreSQL instance.
- No immediate deletion blockers remain. The remaining `local_inference.rs`
  width is a named follow-up candidate rather than a closeout blocker.
