# Playback Runtime Resource Scheduler — TODO

Status: Active
Last updated: 2026-05-29

## Task Ledger

### PRRS-010 — Open lane and freeze resource scheduler boundary

Status: Completed
Owner: codex
Depends on: none

Scope:

- `docs/workstreams/playback-runtime-resource-scheduler`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

Goal:

- Create the durable workstream.
- Freeze single-node playback runtime admission as the first proof.
- Link the lane from playback architecture indexes.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-runtime-resource-scheduler docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Evidence:

- `docs/workstreams/playback-runtime-resource-scheduler/DESIGN.md`
- `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: This lane is open.
- The first implementation task is PRRS-020.
- Do not add remote workers, LL-HLS, DASH, DRM, or queueing semantics before
  the single-node admission model is proven.

### PRRS-020 — Model playback resource demand and admission decisions

Status: Completed
Owner: codex
Depends on: PRRS-010

Scope:

- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/config.rs`

Goal:

- Introduce a server-owned playback resource demand model for direct/remux/HLS
  runtime work.
- Add an admission decision boundary that can explain accepted, rejected, and
  not-yet-enforced resource classes without changing route behavior.
- Keep `nako-transcode` runner semaphores as low-level execution guards.

Validation:

```text
cargo nextest run -p nako-server playback_resource --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-server/src/app/playback/resource.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Playback resource demand is typed for direct stream, remux, and HLS.
- DONE: Admission decisions can explain accepted, rejected, and not-yet-enforced
  classes without changing route behavior.
- The first enforcement task is PRRS-030.

### PRRS-030 — Acquire permits for HLS and remux start paths

Status: Completed
Owner: codex
Depends on: PRRS-020

Scope:

- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/remux.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`

Goal:

- Make HLS and remux start paths acquire admission permits before launching
  process-backed work.
- Ensure reuse of existing sessions does not double-acquire process permits.
- Preserve cancellation/failure cleanup and browser/renderer route contracts.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-server/src/app/playback/resource.rs`
- `crates/nako-server/src/app/playback/hls.rs`
- `crates/nako-server/src/app/playback/remux.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: HLS and remux start paths acquire admission permits before launching
  process-backed runtime work.
- DONE: Existing active/completed session reuse does not double-acquire
  process permits.
- DONE: Permit lifetime is tied to the consuming runtime task, including
  preflight browser playback routes that return before background work exits.
- The next task is PRRS-040.

### PRRS-040 — Surface runtime pressure in Admin diagnostics

Status: Completed
Owner: codex
Depends on: PRRS-030

Scope:

- `crates/nako-server/src/app/playback/resource.rs`
- `crates/nako-server/src/app/playback/support.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi/generated/contract.ts`
- `web/src/api/admin/generated/contract.ts`

Goal:

- Report configured capacity, available permits, and current playback pressure
  in Admin runtime diagnostics.
- Preserve redaction and avoid exposing raw local paths or command lines.

Validation:

```text
cargo nextest run -p nako-server admin_v1_playback --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `admin_v1_playback_runtime_reports_active_resource_pressure`
- Admin runtime diagnostic tests.
- Admin API contract tests and generated TypeScript contracts.
- `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Admin playback runtime diagnostics report configured capacity,
  available permits, and in-use permits for playback resource classes.
- DONE: Active HLS runtime pressure is observable without exposing raw paths,
  locators, command lines, or filenames.
- DONE: Admin TypeScript contracts were regenerated for both web surfaces.
- The next task is PRRS-050.

### PRRS-050 — Verify, document, and close or split follow-ons

Status: Pending
Owner: planner
Depends on: PRRS-040

Scope:

- `docs/workstreams/playback-runtime-resource-scheduler`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Goal:

- Run fresh focused gates.
- Record final evidence and residual risks.
- Close the lane or split follow-ons for queueing, remote workers, OS resource
  isolation, and per-device tuning.

Validation:

```text
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Review:

- Use `review-workstream` and `verify-rust-workstream` before closeout.

Evidence:

- `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`
- `docs/workstreams/playback-runtime-resource-scheduler/HANDOFF.md`
- `docs/workstreams/playback-runtime-resource-scheduler/WORKSTREAM.json`

Handoff:

- Update `WORKSTREAM.json` status and continue policy.
