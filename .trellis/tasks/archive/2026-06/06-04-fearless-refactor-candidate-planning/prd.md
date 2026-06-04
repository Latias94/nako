# Fearless refactor candidate planning

## Goal

Prepare a feature-backed fearless refactor lane that removes accidental
complexity before the next playback/resource/runtime features add more pressure
to `nako-server`.

## What I Already Know

* The working tree is clean after the bounded HLS start admission slice.
* The long-horizon architecture queue is active but currently has no child
  task.
* `docs/architecture/LANES.md` marks playback-transcode, storage-vfs,
  web-product, and control-plane lanes as idle with focused follow-ons.
* `crates/nako-server/src/app/playback/mod.rs` still holds broad playback app
  root responsibilities: session entry points, Direct Play, Remux orchestration,
  HLS delegation, cancellation, source lookup, policy helpers, and tests.
* HLS lifecycle has already been deepened into `hls_flow.rs` and `hls.rs`,
  making Remux lifecycle extraction the most symmetric next refactor candidate.

## Recommended Refactor Candidate

**Playback Remux lifecycle extraction**: move Remux source context construction,
startup admission, background start, input staging, session wait, and output
waiting from `app/playback/mod.rs` into a focused `app/playback/remux_flow.rs`
module.

This is not a pure file split. The intent is to make the playback app root a
thin entrypoint/delegator, matching the HLS lifecycle boundary and reducing
future resource-admission, remote-staging, and playback-session changes that
would otherwise keep growing the root module.

## Alternative Candidates

* **Playback session/direct transport split**: extract session start/link,
  Direct Play stream/preflight, and playback-session lookup helpers from the
  root. Lower risk, but less directly connected to upcoming resource/runtime
  follow-ons.
* **Storage VFS repair execution planning**: continue the VFS cache repair
  preview work into an executable repair boundary. Valuable, but this is a new
  feature slice more than a refactor lane.
* **Provider governance/public client split**: open a metadata/API/Admin
  follow-on. Valuable, but it is broader cross-layer product work and should
  not be the first fearless refactor candidate.

## Requirements (Evolving)

* Keep the refactor feature-backed: it must lower future Remux/playback resource
  and staging work, not only reduce line count.
* Preserve public API, DTO, schema, and runtime behavior.
* Prefer moving cohesive lifecycle code into a focused module over introducing
  new traits or crates.
* Delete or shrink pass-through helpers only when behavior and tests show they
  no longer earn their keep.
* Keep HLS lifecycle and Direct Play remote stream admission unchanged.
* Add or preserve focused tests around Remux resource admission, remote input
  staging release, active session reuse, and playback-session linkage.

## Acceptance Criteria (Evolving)

* [ ] A concrete refactor brief identifies intent, scope, deletion plan,
  boundary plan, tests, risks, and validation commands.
* [ ] If Remux extraction is selected, `PlaybackAppService` Remux public
  methods delegate to a focused module instead of owning the whole lifecycle.
* [ ] Existing Remux, Direct Play, HLS, and playback-session tests continue to
  pass.
* [ ] No public API, DTO, schema, or generated client changes are introduced
  unless a later approved PRD explicitly expands scope.
* [ ] Architecture/spec notes are updated only if a durable boundary or coding
  convention changes.

## Definition Of Done

* Focused PRD and research/context files exist.
* Implementation and check context JSONL are curated before Phase 2.
* `cargo fmt --all -- --check` passes.
* `cargo check -p nako-server --tests` passes.
* Focused `cargo nextest run -p nako-server remux --no-fail-fast` or narrower
  equivalent passes, plus any affected Direct Play/HLS filters.
* Trellis task validation passes.

## Out Of Scope

* New playback API, Admin API, DTO, schema, or generated SDK changes.
* New durable queue/waitlist behavior.
* New HLS features, Direct Play waiting semantics, or storage repair execution.
* New crates or broad trait abstractions without multiple real callers.
* Deleting compatibility behavior without regression tests.

## Technical Notes

* Local scan recorded in `research/refactor-candidate-scan.md`.
* Relevant specs:
  * `.trellis/spec/nako-server/backend/index.md`
  * `.trellis/spec/nako-server/backend/directory-structure.md`
  * `.trellis/spec/nako-server/backend/quality-guidelines.md`
* Relevant architecture maps:
  * `docs/architecture/LANES.md`
  * `docs/architecture/PLAYBACK.md`

## Decision (ADR-lite)

**Context**: The user delegated the next fearless refactor direction to the
agent, with permission to use threads when useful. The candidate scan found
that Remux lifecycle extraction is the most feature-backed refactor because it
mirrors the already deepened HLS lifecycle boundary and reduces future
playback-resource/runtime/staging change cost.

**Decision**: Execute the first task as **Playback Remux lifecycle extraction**.
Do not run multiple implementation workers against the playback root because
the write scope is concentrated in `app/playback/mod.rs` and the new
`remux_flow` boundary. Use a single `trellis-implement` worker, then an
independent `trellis-check` review/fix lane.

**Consequences**: This should shrink the playback root and improve lifecycle
locality without changing public API/schema behavior. Storage VFS repair
execution and Provider/Web governance remain future tasks, not part of this
refactor.
