# Library Watcher Debounce Intake Stability

## Goal

Start productizing the weak watcher/debounce area in the library pipeline with a
small, durable, testable intake stability slice.

## MVP Scope

- Audit current library scan, source tombstone, and intake flow for where a
  watcher/debounce boundary should attach.
- Define and implement the smallest useful debounce/intake primitive, or produce
  a concrete design note if code proves premature.
- Preserve existing scan behavior and source identity semantics.
- Add tests for duplicate rapid events, stable file candidate handling, or the
  selected equivalent MVP behavior.

## Out of Scope

- No OS-specific filesystem watcher daemon unless the MVP explicitly proves it
  is necessary.
- No storage scheduler fairness or staging budget changes.
- No Public Client API change.
- No unbounded filesystem traversal or raw path exposure.

## Acceptance Criteria

- [x] The watcher/debounce MVP boundary is documented and implemented or
  explicitly deferred with evidence.
- [x] Intake remains bounded and redaction-safe.
- [x] Tests cover rapid duplicate events or stable candidate behavior.
- [x] Follow-ons are recorded for full watcher productization.

## MVP Closeout

- Implemented `crates/nako-library/src/intake.rs` as the smallest useful
  debounce/intake primitive: repeated watch observations are reduced to stable
  intake candidate evidence.
- Stable candidate readiness now requires two consecutive identical
  redaction-safe observation keys. A changed observation key resets the counter
  back to the initial inspecting state.
- The slice intentionally stops before storage pressure admission, queued scan
  scheduling, or any OS-specific watcher runtime integration.

## Follow-Ons

- Wire the stable-candidate seam into a real watcher/runtime path once product
  ownership and control-plane boundaries are chosen.
- Add remote-storage-oriented stable-size evidence beyond a single observation
  key when the productized watcher path exists.
- Reconcile stable candidates with scheduled scans and source tombstone repair
  without moving scheduler fairness or storage pressure policy into
  `nako-library`.

## Suggested Gates

- `cargo check -p nako-library -p nako-server --tests`
- Focused `cargo nextest run -p nako-library <filter> --no-fail-fast` or
  focused `nako-server` app tests if the MVP lives in server orchestration
- `cargo fmt --all -- --check`
- `git diff --check`
