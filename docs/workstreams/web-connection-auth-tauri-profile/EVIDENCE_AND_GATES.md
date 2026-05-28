# Web Connection Auth Tauri Profile - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
cargo test --manifest-path web/src-tauri/Cargo.toml
npm --prefix web run tauri -- build
git diff --check
```

Security scans should confirm no bearer/session secrets are placed in URLs,
logs, shared UI props, or persisted profile fields.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WCAT-010 | Queued as lane 4 after route-owned product surfaces. | Queued. |
| 2026-05-28 | WCAT-010 | Activated after WROP closed with route/state tests, check, full test, build, and diff gate passing. | Active. Current task is WCAT-020. |
