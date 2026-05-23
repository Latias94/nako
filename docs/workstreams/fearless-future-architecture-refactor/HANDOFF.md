# Fearless Future Architecture Refactor — Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The lane is open and scoped. The architecture hotspot map is frozen, the
reference policy is explicit, FFR-020 completed the playback runtime split,
FFR-021 completed the first managed import runtime split, FFR-030A completed
the PostgreSQL job backend split, and FFR-030B completed the PostgreSQL
event/webhook backend split. FFR-030C completed the PostgreSQL VFS/staging
backend split. FFR-030D completed the PostgreSQL addon/automation backend
split. FFR-030E completed the PostgreSQL managed artwork backend split.
FFR-030F completed the PostgreSQL import-state backend split. FFR-030G
completed the PostgreSQL metadata/catalog backend split. FFR-030H completed
the PostgreSQL playback/transcode runtime state backend split. FFR-030I
completed the PostgreSQL core library/media/scan/search backend split. FFR-040A
started the API boundary split by moving admin playback and network surfaces
out of `admin.rs`. FFR-040B continued the API boundary split by moving storage
and staging diagnostics out of `admin.rs`. FFR-040C moved generated artifact
and automation review DTOs out of `admin.rs`. FFR-040D moved acquisition
intake and watch-folder discovery DTOs out of `admin.rs`. FFR-040E moved job,
outbox, and ingestion failure operation DTOs out of `admin.rs`. FFR-040F moved
catalog governance and local inference evidence summaries out of `admin.rs`.
FFR-050A started the VFS/inference boundary split by moving local path
authority out of `local.rs`. FFR-050B moved local write transactions out of
`local.rs`. FFR-050C moved local apply/link planning out of `local.rs`.
FFR-050D moved local cleanup/restore lifecycle handling out of `local.rs`.
FFR-050E removed the `nako-naming` dependency on `nako-core` and moved
parsed-name to Nako-domain mapping into `nako-library`.

## Final State

- Status: COMPLETE
- Final task: FFR-060
- Evidence: `docs/workstreams/fearless-future-architecture-refactor/EVIDENCE_AND_GATES.md`
- Follow-up candidate: split `crates/nako-library/src/local_inference.rs`
  internally by planning/mapping if future inference work grows.

## Decisions Since Last Update

- Use `repo-ref/jellyfin` as a behavior and layout reference only.
- Start the next refactor wave with `nako-server` runtime control planes.
- Keep Docker-backed validation part of the normal closeout path.
- FFR-020 split playback staging policy, selection helpers, failure mapping,
  event recording, path helpers, and HLS playlist helpers out of
  `playback/mod.rs` without behavior changes.
- FFR-021 selected `managed_import` as the next broad server module because it
  mixed diagnostics, redaction, promotion outcome JSON, storage apply status,
  and catalog orchestration in one file. It split diagnostics/redaction into
  `diagnostics.rs` and promotion outcome/status helpers into `outcomes.rs`
  without changing public behavior.
- FFR-030A split PostgreSQL job and job lease persistence out of
  `postgres.rs` into `postgres/jobs.rs`. The module now owns job SQL select
  fragments, `JobRepository`, `JobLeaseRepository`, job row mapping, lease
  validation, stale-lease error mapping, and the managed artwork transaction
  helpers `insert_job_tx` / `get_job_tx`.
- FFR-030B split PostgreSQL event outbox and webhook persistence out of
  `postgres.rs` into `postgres/events.rs`. The module owns event/webhook SQL
  select fragments, `EventOutboxRepository`, `WebhookRepository`, event subject
  decoding, row mapping, and delivery-attempt lookup.
- FFR-030C chose VFS/staging over addon/automation because its SQL fragments,
  row mapping, transactional listing upserts, staging reservation budget, and
  lease transitions form a cohesive backend domain with direct focused contract
  coverage. It split that implementation into `postgres/vfs_staging.rs`.
- FFR-030D split PostgreSQL addon and automation persistence out of
  `postgres.rs` into `postgres/addons_automation.rs`. The module owns addon
  registration/token/grant/routing/side-effect SQL, automation provider and
  artifact SQL, generated artifact proposal hydration, row mapping, and the
  side-effect apply-outcome transaction helper used by metadata commit.
- FFR-030E split PostgreSQL managed artwork persistence out of `postgres.rs`
  into `postgres/managed_artwork.rs`. The module owns artwork task, artwork
  candidate, managed ingest, managed artifact, selected artwork, gallery, and
  lifecycle cleanup SQL, row mapping, and transaction-heavy flows. The ignored
  PostgreSQL managed artwork contract harness passed locally.
- FFR-030F split PostgreSQL import-state persistence out of `postgres.rs` into
  `postgres/import_state.rs`. The module owns managed import artifact,
  promotion apply, acquisition intake, and NFO sidecar apply SQL, state
  transitions, row mapping, and source-kind codecs. The full ignored
  PostgreSQL all-contract harness passed locally.
- FFR-030G split PostgreSQL provider mapping, metadata commit, and catalog
  graph persistence out of `postgres.rs` into `postgres/metadata_catalog.rs`.
  The module owns provider subject/raw response/attempt persistence, metadata
  field locks, metadata refresh/NFO import/addon metadata write commits,
  catalog graph replacement, catalog entities, image assets, row mapping, and
  related provider/source/image codecs. Shared low-level media item, library
  item state, search projection, and external-id lookup helpers remain in
  `postgres.rs`.
- FFR-030H selected playback plus transcode runtime state as a cohesive tail
  split because it has local SQL select fragments, row mapping, codecs, and
  direct playback runtime contract coverage. The implementation now lives in
  `postgres/playback_runtime.rs`.
- FFR-030I extracted the core library/media/scan/search persistence family
  into `postgres/core_catalog.rs`. The module owns library, library-item,
  media, media-probe, local inference, ingestion failure, scan, search,
  source duplicate, catalog governance, row mapping, and the shared
  media/source/search transaction helpers. `postgres.rs` is now mostly
  connection, migration, schema validation, numeric conversion, and module
  dispatch.
- FFR-040A split the admin playback API surface into
  `crates/nako-api/src/admin/playback.rs`. That module owns playback session
  list DTOs, runtime diagnostics, support evidence, hardware readiness
  summaries, request-key fingerprinting, source-scheme redaction, and the
  playback redaction tests.
- FFR-040A also split the admin network API surface into
  `crates/nako-api/src/admin/network.rs`. That module owns exposure mode,
  readiness checks, external endpoint, trusted proxy, origin policy, tunnel
  provider diagnostics, and the network readiness/redaction tests.
- FFR-040B split the admin storage API surface into
  `crates/nako-api/src/admin/storage.rs`. That module owns staging
  diagnostics, VFS cache summaries, storage backend diagnostics, backend
  runtime state scope, staging record conversion, and storage redaction tests.
- FFR-040C split the admin automation API surface into
  `crates/nako-api/src/admin/automation.rs`. That module owns generated
  artifact proposal, review, acceptance plan, target, provenance, payload
  summary, readiness DTOs, and raw prompt/payload redaction tests.
- FFR-040D split the admin intake API surface into
  `crates/nako-api/src/admin/intake.rs`. That module owns acquisition intake
  candidate diagnostics, watch-folder discovery DTOs, source reference
  redaction fields, source key fingerprints, and intake redaction tests.
- FFR-040E split the admin operations API surface into
  `crates/nako-api/src/admin/operations.rs`. That module owns job, job
  cancellation, outbox event, ingestion failure, and ignore-ingestion-failure
  DTOs plus payload/error redaction tests.
- FFR-040F split the admin catalog governance API surface into
  `crates/nako-api/src/admin/catalog_governance.rs`. That module owns catalog
  governance item summaries, local inference evidence summaries, governance
  issue derivation, and local inference redaction tests.
- FFR-040 review found no blocking findings. The remaining `admin.rs` width
  is server config diagnostics and overview summaries. Keeping those passive
  aggregates in the root admin module is acceptable because the behavior-rich
  and redaction-sensitive surfaces now live in focused modules with local
  tests.
- FFR-050A split local path authority out of
  `crates/nako-vfs/src/local.rs` into
  `crates/nako-vfs/src/local/path_authority.rs`. The module owns local root
  canonicalization, scheme checks, relative path parsing, read/write/cleanup
  path resolution, local URI construction, backup URI construction, and
  security-violation classification. Existing local backend behavior remains
  covered by focused `nako-vfs` tests.
- FFR-050B split local write transactions out of `local.rs` into
  `crates/nako-vfs/src/local/write_transaction.rs`. The module owns atomic
  replace, backup creation, backup retention pruning, restore temp-file
  handling, fsync helpers, and backup sidecar naming.
- FFR-050C split local apply/link planning out of `local.rs` into
  `crates/nako-vfs/src/local/apply_plan.rs`. The module owns link readiness
  planning, copy apply, hardlink/symlink apply, apply status mapping, and
  create-new file actions, while path authority and write transactions remain
  in their focused modules.
- FFR-050D split local cleanup/restore lifecycle handling out of `local.rs`
  into `crates/nako-vfs/src/local/lifecycle.rs`. The module owns cleanup,
  restore, lifecycle request validation, lifecycle status mapping, and
  cleanup/restore report construction.
- FFR-050E made `nako-naming` a pure naming parser crate by replacing direct
  `nako-core` `MediaKind` / `LocalInferenceEvidenceSource` use with
  parser-local `ParsedMediaKind` / `NameEvidenceSource`. `nako-library`
  now owns the mapping into Nako `MediaKind` and `LocalInferenceEvidence`,
  with a custom-parser boundary test proving the conversion point.
- FFR-050 review found no blocking findings. The remaining
  `local_inference.rs` width is an internal follow-up candidate, not a blocker
  for moving to FFR-060.
- FFR-060 deletion/duplication sweep found no remaining replaced helper paths
  requiring immediate removal.
- FFR-060 closeout gates passed: workspace formatting, workspace test
  compilation, workspace nextest, container release gate, PostgreSQL
  all-contract harness, JSON validation, and `git diff --check`.

## Blockers

- None.

## Next Recommended Action

- The workstream is complete. Next architecture work should open a new
  follow-on lane rather than continuing this one.
