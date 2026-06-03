# Library Watcher Debounce Intake Stability Evidence

Date: 2026-06-03

## Selected Slice

- Implemented the smallest useful watcher/debounce intake foundation in
  `crates/nako-library/src/intake.rs`.
- Added a stable-candidate evidence seam that reduces repeated watch
  observations to either `Inspecting` or `Stable`.
- A candidate becomes `Stable` only after the same redaction-safe observation
  key is seen twice consecutively; a changed key resets stability back to the
  first-observation state.
- No storage pressure admission, queued scan scheduler, Public Client API, or
  OS-specific watcher daemon work was added.

## Verification

- `cargo check -p nako-library -p nako-server --tests` passed.
- `cargo test -p nako-library intake -- --nocapture` passed: 3 tests passed.
- `cargo nextest run -p nako-library intake --no-fail-fast` passed: 3 tests
  passed.
- `cargo nextest run -p nako-server scan_library_persists_job_success --no-fail-fast`
  passed: 1 test passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

## Coverage

- The first observation for a watcher candidate remains in `Inspecting` state.
- Two consecutive identical observation keys mark the candidate `Stable`.
- A changed observation key resets the stability counter back to one.

## Deferred Follow-Ons

- Real filesystem watcher/runtime integration in `nako-server` or another
  control-plane-owned runtime boundary.
- Richer stable-size or closed-file evidence that uses more than one
  observation key for remote or slow-copy backends.
- Reconciliation between incremental watcher intake and scheduled scans/source
  tombstone repair without absorbing scheduler fairness or storage-pressure
  policy into this slice.

## Spec Update Judgment

Updated `.trellis/spec/nako-library/backend/directory-structure.md` and
`.trellis/spec/nako-library/backend/quality-guidelines.md` because this task
introduced a new intake evidence module and an executable stable-candidate
contract.
