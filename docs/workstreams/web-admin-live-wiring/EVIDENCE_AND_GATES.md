# Web Admin Live Wiring - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
cargo nextest run -p nako-api admin_web_generated_contract_matches_generator_output
git diff --check
```

Run the Admin contract gate when generated Admin API artifacts change.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WALW-010 | Queued as lane 5 after connection/auth/profile. | Queued. |
| 2026-05-28 | WALW-010 | WCAT-050 closed at commit `1cccdd7a`; WALW status moved to active and current task set to WALW-020. | Passed. |
| 2026-05-28 | WALW-020 | Added Admin read-model data source for libraries/users/tasks/logs/settings; wired pages through React Query; added data-source contract tests. Validation: `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build`, scoped `git diff --check`. | Passed. |
