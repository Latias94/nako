# Evidence: Control Plane Scan Trace Context

## Integrated Commit

* `56c585cc feat(control-plane): propagate scan trace context`

## Changed Scope

* `crates/nako-server/src/app/job_runtime.rs`
* `crates/nako-server/src/app/jobs.rs`
* `crates/nako-server/src/app/tests/startup.rs`
* `docs/architecture/CONTROL_PLANE.md`

## Review

Independent review reported: `No findings; safe to proceed`.

The review specifically checked redaction, request ID normalization, legacy
job compatibility, durable job input handling, outbox payload shape, and
write-scope boundaries.

## Main Merge-Gate Verification

* `cargo check -p nako-server --tests` passed.
* `cargo nextest run -p nako-server trace_context --no-fail-fast` passed: 9
  tests.
* `cargo nextest run -p nako-server jobs --no-fail-fast` passed: 8 tests.
* `cargo fmt --all -- --check` passed.
* `git diff --check` passed.

## Residual Risk

This slice does not connect the HTTP library scan route to `HttpTraceContext`.
That remains a follow-on because the task intentionally avoided HTTP/API scope.

