# User Playback State Contract Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Contract Freeze

Exit criteria:

- Route inventory and DTO names are proposed and reviewed.
- User principal behavior for **Single-Admin Mode** is explicit.
- Progress, watched, favorite, hidden, user rating, and Continue Watching
  semantics are either in scope or intentionally deferred.
- Required ADR decision is identified or ruled unnecessary.

Evidence:

- `TODO.md` UPS-010 complete
- `CONTRACT.md`
- `DESIGN.md`
- `EVIDENCE_AND_GATES.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`

## M1 - Server Authority

Exit criteria:

- Server persists **User Playback State** by explicit user principal.
- Lookup/report/mark-watched behavior is covered by app-service and storage
  tests.
- Writes are idempotent and do not require source locators or local paths.

Evidence:

- `cargo nextest run -p taru-db -p taru-server user_playback --no-fail-fast`

## M2 - Public Contract

Exit criteria:

- Public API docs, protocol DTOs, OpenAPI, Rust SDK, and TypeScript SDK expose
  the same route contract.
- SDK drift checks pass.
- Error behavior follows the public error envelope.

Evidence:

- `cargo nextest run -p taru-api -p taru-client --no-fail-fast`
- `npm run check --prefix sdk/typescript`

## M3 - Android Integration

Exit criteria:

- Android can read authoritative resume state and report progress through the
  Public Client API.
- Continue Watching is visible only with server-backed state.
- Device-local resume remains labeled and scoped as fallback/local cache.

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `TODO.md` UPS-040 complete

## M4 - Closeout

Exit criteria:

- Android smoke evidence proves the server-backed Continue Watching path.
- Workstream docs reflect shipped behavior.
- Follow-ons for accounts, offline sync, recommendations, and richer user
  preferences are split or explicitly deferred.
