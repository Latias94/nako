# Library Watcher Runtime Productization

## Goal

Turn the shipped `nako-library::intake` stable-candidate evidence seam into a
real, supervised watcher runtime path that can observe storage changes, debounce
unsafe events, and hand safe candidates to existing scan/intake authorities.

## Context

Wave 05d shipped the stable-candidate intake evidence foundation only. It did
not add an OS watcher daemon, storage-pressure admission, scan scheduler
behavior, or product/runtime integration. This task is the implementation lane
that starts that productization.

## Scope

* Audit the current library scan, intake, durable job runtime, and server
  composition points before selecting the exact integration seam.
* Add the smallest supervised watcher runtime path that fits ADR 0053 and the
  current server runtime model.
* Normalize filesystem events into redaction-safe intake observations and reuse
  the existing stable-candidate evidence primitive.
* Coalesce duplicate or noisy events before scan/probe work starts.
* Hand stable candidates to existing scan/intake scheduling boundaries instead
  of adding a second scan executor.
* Preserve scheduled reconciliation as the correction path for missed or
  unreliable watcher events.
* Add focused tests for duplicate events, copy-in-progress behavior, runtime
  lifecycle, or the selected equivalent productization slice.

## Non-Goals

* No staging attribution persistence or storage schema changes. That belongs to
  `06-03-06b-storage-staging-attribution-persistence`.
* No Jellyfin research deliverable. That belongs to
  `06-03-06c-targeted-jellyfin-watcher-reference`.
* No broad scan scheduler fairness rewrite.
* No raw `tokio::spawn` background runtime that bypasses ADR 0053 supervision.
* No promise that remote backends have trustworthy watch events unless the
  backend capability already proves it.
* No raw source locator, host path, fingerprint, token, or backend credential in
  diagnostics.

## Acceptance Criteria

* [ ] A watcher runtime or clearly bounded first runtime slice is implemented
      through existing server/control-plane boundaries.
* [ ] Stable-candidate evidence gates prevent premature probe/scan on
      copy-in-progress style events.
* [ ] Watcher events hand off to existing scan/intake authorities rather than a
      duplicate executor.
* [ ] Runtime diagnostics are redaction-safe.
* [ ] Follow-ons are recorded for any OS/backend capability not covered by this
      first productization slice.

## Suggested Gates

* `cargo check -p nako-library -p nako-server --tests`
* Focused `cargo nextest run -p nako-library intake --no-fail-fast`
* Focused `cargo nextest run -p nako-server <watcher-or-scan-filter> --no-fail-fast`
* `cargo fmt --all -- --check`
* `git diff --check`

## Coordination Notes

* Coordinate with 06b only if the selected watcher runtime needs persisted
  staging attribution facts in the same schema or DTO shape.
* Treat 06c findings as optional decision input; do not wait for a broad
  Jellyfin audit.
