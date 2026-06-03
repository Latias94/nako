# Storage Staging Attribution Persistence

## Goal

Persist authoritative staging attribution so storage pressure, diagnostics, and
scan-admission behavior can represent ambiguous same-root or multi-endpoint
library staging honestly.

## Context

Wave 05 shipped scoped staging admission and queued scan fairness, but the
archive evidence still calls out persisted attribution for ambiguous same-root
multi-endpoint library staging records as a follow-on. This task owns that
storage/VFS authority improvement.

## Scope

* Audit current staging manifests, storage pressure policy slices, scan
  admission reads, and Admin/server diagnostics before choosing the exact
  persistence seam.
* Persist only authoritative attribution facts. If a staging record cannot be
  confidently tied to one library/source/backend slice, model that ambiguity
  explicitly.
* Add repository/schema changes only where persistence authority is required.
* Preserve SQLite and PostgreSQL parity for changed repository contracts.
* Keep scan-admission and diagnostics consumers honest: attributed, ambiguous,
  unknown, and multi-owner cases must not collapse into false ownership.
* Preserve redaction of raw source locators, local paths, source fingerprints,
  backend credentials, etags, headers, and host filesystem details.

## Non-Goals

* No watcher runtime, debounce, or filesystem event productization. That belongs
  to `06-03-06a-library-watcher-runtime-productization`.
* No Jellyfin reference research. That belongs to
  `06-03-06c-targeted-jellyfin-watcher-reference`.
* No broad PostgreSQL runtime suite expansion beyond gates required by changed
  attribution persistence.
* No cache repair operator workflow.
* No source fingerprint escalation policy.
* No new staging-pressure policy unless it is the minimal consumer needed to
  prove persisted attribution correctness.

## Acceptance Criteria

* [ ] Staging attribution has a persisted authority or a documented reason why
      persistence is deferred after audit.
* [ ] Ambiguous same-root and multi-endpoint cases remain explicit and do not
      create false per-library attribution.
* [ ] SQLite/PostgreSQL repository contract coverage exists for changed
      persistence behavior.
* [ ] Any server/Admin diagnostics remain redaction-safe.
* [ ] Follow-ons are recorded for repair workflows, broader PostgreSQL runtime
      suites, or source fingerprint escalation if discovered.

## Suggested Gates

* `cargo check -p nako-db -p nako-server -p nako-library --tests`
* Focused `cargo nextest run -p nako-db <staging-or-storage-filter> --no-fail-fast`
* Focused `cargo nextest run -p nako-server <staging-or-scan-filter> --no-fail-fast`
* PostgreSQL contract harness only if changed contracts require it
* `cargo fmt --all -- --check`
* `git diff --check`

## Coordination Notes

* Coordinate with 06a only if watcher runtime implementation needs the same
  attribution model or DTO in its first slice.
* Do not absorb the deferred broader PostgreSQL runtime suite lane into this
  task.
