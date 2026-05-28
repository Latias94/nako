# Web Admin Live Wiring - Evidence And Gates

Status: Queued
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
