# Local Workspace Hotspots

## Method

Main-thread lightweight scan only. This note records broad workspace signals so
sub-agent reviews can focus on deeper module-boundary friction.

Commands used:

* `git status --short --branch`
* `Get-Content docs/ARCHITECTURE.md`
* `Get-ChildItem docs/architecture -Filter *.md`
* `rg --files crates | rg '\.rs$' | ... line count sort`
* `rg -n "pub trait .*Repository|struct .*AppService|route\(" ...`

## Observations

* The workspace is clean before review.
* Architecture maps already define active capability areas: playback, storage
  VFS, library pipeline, state/access, realtime/sync, operations/release, and
  control plane.
* `docs/architecture/LANES.md` says no implementation lane is active; review
  should propose follow-on Trellis tasks rather than reopen closed lanes.
* Large-file hotspots:
  * `crates/nako-server/src/http/tests/addons.rs` (~12.8k lines)
  * `crates/nako-server/src/http/tests/system.rs` (~9.6k lines)
  * `crates/nako-db/src/contract_tests.rs` (~8.8k lines)
  * `crates/nako-db/src/tests.rs` (~5.9k lines)
  * `crates/nako-metadata/src/tests.rs` (~4.5k lines)
  * `crates/nako-addon-protocol/src/lib.rs` (~4.2k lines)
  * `crates/nako-transcode/src/lib.rs` (~3.8k lines)
  * `crates/nako-api/src/admin_contract.rs` (~3.5k lines)
  * `crates/nako-server/src/http/admin.rs` (~3.2k lines)
  * `crates/nako-playback/src/lib.rs` (~3.1k lines)
  * `crates/nako-server/src/app/storage.rs` (~2.6k lines)
* `nako-server/src/app` has many `*AppService` modules. That is not
  automatically bad, but it is a strong signal to test whether each app-service
  module is a deep module or a shallow pass-through to repositories/DTO mapping.
* `nako-core/src/repository` has many repository traits. This is aligned with
  ADR 0001 and DB parity goals, but each new method should keep the interface
  smaller than the implementation complexity it hides.
* `nako-server/src/http/admin.rs` has a large centralized route inventory plus
  many mapping helpers. It may be an intentional Admin API boundary, but it is a
  likely place to look for repeated mapping conventions that could become deeper
  generated or grouped modules.

## Initial Hypotheses To Validate

1. Admin API mapping may be too centralized: route/mapping/test growth in one
   file may reduce locality for feature-specific Admin surfaces.
2. Repository contract tests may be too monolithic: keeping parity is valuable,
   but test locality may improve if feature contract suites become grouped
   modules without weakening shared backend coverage.
3. Playback/transcode crates may still carry broad `lib.rs` surfaces that would
   be deeper if planner/runtime/artifact concepts were split into smaller
   public modules with stable interfaces.
4. Addon Protocol may be a large interface surface; the deletion test should
   distinguish useful permissive protocol depth from accumulated manifest/task/
   hosted-surface sprawl.
