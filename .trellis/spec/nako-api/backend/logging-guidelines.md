# Logging Guidelines

`nako-api` is a contract crate and should not emit runtime logs.

## Rules

- Do not add tracing or logging side effects to DTO definitions or generator
  helpers.
- Generator commands may print deterministic command/help output, but should
  not log secrets or machine-local paths beyond explicit output paths.
- Runtime request logging belongs in `nako-server`.

## Evidence

- `crates/nako-api/src/admin_contract.rs`
- `crates/nako-server/src/http.rs`
