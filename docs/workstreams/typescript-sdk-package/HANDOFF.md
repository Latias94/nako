# TypeScript SDK Package Hardening Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M34 is closed. `sdk/typescript` is now a minimal private TypeScript SDK
package with repeatable generation, local TypeScript tooling, strict
`tsc --noEmit` validation, and a Rust sync test against the `nako-api`
generator.

## Decisions Since Last Update

- The TypeScript SDK package lives under `sdk/typescript`, because it is a
  reusable SDK package rather than a concrete client application.
- TypeScript is a package-local development dependency.
- `node_modules` is ignored and not committed; `package-lock.json` is committed
  once the package exists.
- Generated source is committed as `sdk/typescript/src/index.ts` and refreshed
  by command.

## Blockers

- None.

## Next Recommended Action

- Open M35 for Rust Client SDK Foundation if the next goal continues SDK work.
- Keep npm publishing, Dart/Flutter SDK, OpenAPI runtime serving, and concrete
  client UI split into separate follow-ons.
