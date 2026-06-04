# Research: storage/library/control-plane next candidates

- Query: pick the next bounded implementation task for the long-horizon queue across storage/VFS, library scan, and control-plane lanes.
- Scope: internal
- Date: 2026-06-03

## Findings

### Files found

- `docs/architecture/STORAGE_VFS.md`: current storage/VFS follow-ons, risk register, and the proposed cache repair / fingerprint / harness next lanes.
- `docs/architecture/LIBRARY_PIPELINE.md`: watcher/debounce status, intake stability, and the remaining scan/fallback follow-ons.
- `docs/architecture/CONTROL_PLANE.md`: ADR 0053 baseline, durable-job/runtime/diagnostics/API-scale follow-ons, and trace-context work.
- `docs/architecture/LANES.md`: lane ownership map and idle queue status for `storage-vfs`, `library-metadata-control-plane`, and `control-plane`.
- `docs/architecture/WORKSTREAM_LINKS.md`: cross-link registry for storage, library, and control-plane workstreams.
- Recent archive evidence: `06-03-05a` staging budget per-backend policy, `06-03-05b` scan scheduler fairness, `06-03-05c` PostgreSQL parity harness, `06-03-05d` watcher debounce intake stability, `06-03-06a` watcher runtime productization, `06-03-06b` staging attribution persistence, `06-03-06c` Jellyfin watcher reference.
- Core seams: `crates/nako-vfs/src/lib.rs`, `crates/nako-server/src/app/storage.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/storage.rs`, `crates/nako-server/src/app/watch_folder_runtime.rs`, `crates/nako-server/src/app/acquisition_intake.rs`, `crates/nako-server/src/app/watch_folder_suppression.rs`, `crates/nako-server/src/app/jobs.rs`, `crates/nako-core/src/media/source.rs`, `crates/nako-library/src/ingestion/source_commit.rs`.

### Code patterns

- Storage/VFS already has a typed repair-diagnostic seam: `crates/nako-vfs/src/lib.rs:147,188,208,225,254,264`.
- Server storage diagnostics already aggregate policy slices and scan admission: `crates/nako-server/src/app/storage.rs:282,324,358,410,548,599,636,670,792`.
- Admin storage routes already render the storage/staging diagnostics boundary: `crates/nako-server/src/http/admin.rs:1759,1787,1797,1822,1836,1859,1886` and `crates/nako-api/src/admin/storage.rs:21,61,83,101,125,141`.
- Watch-folder productization already exists as a supervised runtime plus intake/suppression/scan handoff: `crates/nako-server/src/app/watch_folder_runtime.rs:18,25,33,57,98,109,121,127,151,164`, `crates/nako-server/src/app/acquisition_intake.rs:338,404,430,433,452,470,482,493,516,754,766,774,816,825,833,849,864,878,930,948,974,991`, `crates/nako-server/src/app/watch_folder_suppression.rs:15,41,43,72,82,120,136,145`, `crates/nako-server/src/app/jobs.rs:316,324,349,361,387,542,585,658,671`.
- Source identity already has duplicate/fingerprint evidence and preservation decisions: `crates/nako-core/src/media/source.rs:55,62,116,121` and `crates/nako-library/src/ingestion/source_commit.rs:190,192,198,253,255,318,371,396`.
- Control-plane baseline already covers request identity, durable jobs, tracing, diagnostics, and API scale: `docs/architecture/CONTROL_PLANE.md:182,190,191,194,199,210,218,224,227,236,240,245` and `docs/adr/0053-application-control-plane-boundary.md:65,68,70,84,85`.
- Lane ownership confirms storage-vfs, library, and control-plane are the right homes for the next tasks: `docs/architecture/LANES.md:318,319,324,325,326,327,331,335,360`.

### Candidate tasks

| Rank | Candidate | Why it is high leverage | Suggested write scope | Validation commands | Parallel |
| --- | --- | --- | --- | --- | --- |
| 1 | VFS cache repair operator actions | Converts existing redaction-safe repair diagnostics into an operator action seam without reopening scan fairness or watcher runtime work. It is already named in `STORAGE_VFS.md` and has a clear `nako-vfs` + server/Admin boundary. | `crates/nako-vfs/src/lib.rs`, `crates/nako-server/src/app/storage.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/storage.rs`, focused storage/Admin tests. | `cargo check -p nako-vfs -p nako-server -p nako-api --tests`; focused `cargo nextest run -p nako-vfs --no-fail-fast`; focused `cargo nextest run -p nako-server storage --no-fail-fast`; focused `cargo nextest run -p nako-api admin_storage --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`. | 有条件: yes if kept preview-only and DTO changes stay isolated; serialize if another task is editing `nako-api` storage DTOs. |
| 2 | Watch-folder degraded / reconciliation-pending diagnostics and scan handoff | Closes the remaining gap after watcher runtime productization by turning unreliable watcher outcomes into explicit, redaction-safe product states instead of only silent suppression or normal scan enqueue. | `crates/nako-server/src/app/watch_folder_runtime.rs`, `crates/nako-server/src/app/acquisition_intake.rs`, `crates/nako-server/src/app/watch_folder_suppression.rs`, `crates/nako-server/src/app/jobs.rs`, `crates/nako-server/src/http/admin.rs`, `crates/nako-api/src/admin/intake.rs`, watcher tests. | `cargo check -p nako-server -p nako-api --tests`; focused `cargo nextest run -p nako-server watch_folder --no-fail-fast`; focused `cargo nextest run -p nako-api watch_folder --no-fail-fast`; focused queue-handoff scan tests; `cargo fmt --all -- --check`; `git diff --check`. | 有条件: mostly yes, but avoid parallel work that also changes the same Admin intake DTOs or watch-folder runtime modules. |
| 3 | Source fingerprint escalation policy | Improves source-identity precision for ambiguous cases using the existing layered evidence model, but it can become expensive if the policy grows beyond a narrow escalation slice. | `crates/nako-core/src/media/source.rs`, `crates/nako-library/src/ingestion/source_commit.rs`, possibly `crates/nako-server/src/app/library_reconciliation.rs` and `crates/nako-server/src/app/storage.rs`, source-identity tests. | `cargo check -p nako-core -p nako-library -p nako-server --tests`; focused `cargo nextest run -p nako-core --no-fail-fast` for source identity tests; focused `cargo nextest run -p nako-library --no-fail-fast` for reconciliation/ingestion tests; `cargo fmt --all -- --check`; `git diff --check`. | yes, with cache-repair work if the API contract stays separate; no if both tasks try to change the same `nako-core` source-identity types. |
| 4 | Control-plane observability and trace context first slice | Valuable for incident correlation, but it is the broadest and most cross-cutting candidate here, so the bounded slice has to stay very small. | `crates/nako-server/src/http/*`, `crates/nako-server/src/app/job_runtime.rs`, `crates/nako-server/src/app/jobs.rs`, `crates/nako-server/src/app/addons.rs`, possibly `crates/nako-api/src/admin/*`, logging/trace tests. | `cargo check -p nako-server --tests`; focused `cargo nextest run -p nako-server startup --no-fail-fast`; focused `cargo nextest run -p nako-server http --no-fail-fast` if the slice touches routes; `cargo fmt --all -- --check`; `git diff --check`. | No: this wants a clean, isolated server-plumbing pass. |

### Recommendation

Start with `vfs-cache-repair-operator-actions`.

Reason: it is the best balance of user value and boundedness. The seam already exists in `nako-vfs` repair diagnostics and in server/Admin storage diagnostics, so the task can turn telemetry into operator action without reopening the completed watcher/runtime work, without a schema-first rewrite, and without a broad control-plane refactor.

## Spec / ADR / Architecture update points

- `docs/architecture/STORAGE_VFS.md` already names `proposed:vfs-cache-repair-operator-actions`; if the chosen task stays read-only preview + refresh action, the architecture map likely only needs a status update after implementation. If the task adds a real operator mutation or new repair lifecycle state, update this doc before code lands.
- `docs/architecture/CONTROL_PLANE.md` and ADR 0053 already cover durable jobs, runtime supervision, and redacted diagnostics; update them only if the selected task introduces a new durable runtime or trace-context contract, not for a small storage/Admin preview slice.
- `docs/architecture/LIBRARY_PIPELINE.md` only needs an update if the next task is the watcher degraded/reconciliation follow-on and the product boundary becomes a new documented capability.
- `docs/architecture/LANES.md` should be revisited when the chosen task becomes the active lane, so the idle/next-action lines match the actual workstream.
- Likely spec touchpoints for implementation work: `.trellis/spec/nako-vfs/backend/error-handling.md`, `.trellis/spec/nako-vfs/backend/quality-guidelines.md`, `.trellis/spec/nako-server/backend/logging-guidelines.md`, `.trellis/spec/nako-server/backend/quality-guidelines.md`, and `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` if new Admin DTOs are introduced.

## Caveats / Not Found

- `python3 ./.trellis/scripts/task.py current --source` returned no active task, so this research used the task directory explicitly provided in the prompt.
- The archive records show `06a`, `06b`, and `06c` as completed; they are evidence for the next decision, not open work to reopen.
- I did not find a separate implemented `vfs-cache-repair-operator-actions` task or workstream evidence file; the architecture doc still lists it as a proposed next lane.
- The watcher fallback/reconciliation follow-on still has some unresolved policy questions in the 06c research notes, so it is less bounded than the cache-repair operator-action slice.
- `repo-ref/` Jellyfin material was treated as behavior-only reference in the prior research lane; no code, comments, tests, schemas, or control flow were copied, translated, or ported.
