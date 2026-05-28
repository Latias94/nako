# Web Media Live Public Client Parity - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

WMLP-020 verified the generated Public Client SDK and committed route inventory
for Media Web live parity. The SDK has enough contract surface for the first
read slice: `listItems`, `searchItems`, `getItem`, item credits/images,
management context links, playback decision, browser playback ticket, playback
sessions, HLS session segments, and user playback-state reads/writes.

The main readiness gap is library-scoped item browse. `listLibraries`,
`getLibrary`, and `listLibrarySources` exist, but there is no SDK method or
route inventory entry for `/libraries/{library_id}/items`, and `listItems` does
not accept a `library_id` filter. `/media/library` must therefore stay truthful:
library metadata/source readiness is OK, but scoped item browse needs an
explicit missing-contract state or clearly labeled all-library fallback.

## Active Task

- Task ID: WMLP-020
- Owner: Codex
- Status: DONE
- Validation: `npm --prefix web run test -- src/test/data-source-contracts.test.ts` passed with 13 tests; `npm --prefix web run check` passed.

## Next Task

- Task ID: WMLP-030
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build:budget`

## Next Recommended Action

- Run WMLP-030: add `web/src/api/public` read models for `/media`,
  `/media/search`, `/media/detail`, and `/media/library` readiness. Wire routes
  to live contracts where available and show explicit readiness states where
  contracts are missing.
