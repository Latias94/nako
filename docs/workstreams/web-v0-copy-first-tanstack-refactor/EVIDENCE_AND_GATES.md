# Web V0 Copy-First TanStack Refactor - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Policy

This lane intentionally starts with a large copy. Every follow-up task must
reduce ambiguity: runtime assumptions, API boundaries, route ownership,
performance, or desktop packaging.

## Baseline Gates

```bash
git status --short
npm --prefix web run check
npm --prefix web run test
npm --prefix web run build
cargo test --manifest-path web/src-tauri/Cargo.toml
```

## Contract Gates

Run when Admin or Public Client API generated artifacts change:

```bash
npm --prefix web run generate:admin-api
cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output
```

## Browser And Desktop Gates

- Browser/Playwright smoke for `/media`, `/admin`, `/setup`, and mobile view.
- Tauri static packaging smoke once the frontend build path is restored.
- Console output must not contain application errors.

## Performance Gates

- Record production bundle output after copy baseline.
- Record production bundle output after route-level splitting.
- Large poster grids, search results, logs, and admin tables need a
  virtualization decision before closeout.
- Heavy route-only dependencies must not stay in the initial route chunk unless
  there is a measured reason.

## Safety Checks

Release routes must not expose:

- bearer/session secrets;
- provider API keys;
- raw local paths;
- raw Source Locators;
- raw provider payloads;
- raw addon link URLs/passwords;
- Addon tokens or webhook secrets;
- FFmpeg argv/output paths/stderr;
- third-party provider artwork as bundled Nako-owned assets.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WVTR-010 | Workstream opened after user accepted copy-first refactor and autonomous commits. | Active. |
