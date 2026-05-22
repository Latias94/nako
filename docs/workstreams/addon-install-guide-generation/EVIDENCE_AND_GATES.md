# Addon Install Guide Generation Evidence And Gates

Status: Completed
Last updated: 2026-05-22

## Smallest Current Repro

```bash
cargo nextest run -p nako-server install_guide --no-fail-fast
```

## Gate Set

### Contract Gate

```bash
cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts
cargo nextest run -p nako-api admin_contract --no-fail-fast
```

Proves the Admin API TypeScript contract is generated from `nako-api` and
stays redaction-safe.

### Server Guide Gate

```bash
cargo nextest run -p nako-server install_guide --no-fail-fast
```

Proves the Admin route composes install guide sections without lifecycle
automation or secret leakage.

### Admin Web Gate

```bash
cd apps/admin-web
npm test
npm run build
```

Proves the guide is consumed through the Admin Web seam and can be rendered in
the production bundle.

### Closeout Gate

```bash
cargo fmt --all -- --check
cargo check -p nako-api -p nako-server --tests
git diff --check
```

Use broader gates if touched files expand beyond the planned scope.

## Evidence Log

| Date | Scope | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | AIG-010 workstream opening | Created authoritative workstream docs and active Codex goal. | Pass |
| 2026-05-22 | AIG-020 server guide contract | `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server install_guide --no-fail-fast` | Pass. Generated route/DTO contract includes Addon Install Guide; focused server test proves snippets, Secret Reference placeholders, health/registration verification steps, and no lifecycle-control or sensitive leakage. |
| 2026-05-22 | AIG-030 Admin Web preview | `npm run check`; `npm test -- --run src/adminApi src/App.test.tsx`; `npm run build` from `apps/admin-web` | Pass. Admin Web loads guide through client/data-source seam, renders Docker Compose/systemd/Secret Reference/verification previews, and production build succeeds. |
| 2026-05-22 | AIG-040 closeout preflight | `cargo fmt --all -- --check`; `cargo check -p nako-api -p nako-server --tests`; `npm run check`; `npm test`; `npm run build` | Pass. Formatting, Rust touched package checks, Admin Web typecheck/tests/build all passed before closeout doc finalization. |
| 2026-05-22 | AIG-040 final closeout | `cargo fmt --all -- --check`; `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server install_guide --no-fail-fast`; `cargo check -p nako-api -p nako-server --tests`; `npm run check`; `npm test`; `npm run build`; `git diff --check` | Pass. Final closeout gates prove generated contract parity, server guide behavior, touched Rust package compilation, Admin Web typecheck/tests/build, formatting, and whitespace sanity. |

## Notes

- Fresh verification is required before marking the Codex goal complete.
- The guide must remain a read-only Admin aid. Any process/package lifecycle
  behavior belongs in a future Addon Manager lane.
