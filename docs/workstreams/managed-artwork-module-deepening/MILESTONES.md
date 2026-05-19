# Managed Artwork Module Deepening Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Lane Opened

Exit criteria:

- Workstream docs exist and agree on goals, non-goals, and redaction
  invariants.
- Existing Managed Artwork lanes are referenced as the behavior baseline.
- The first implementation task is small enough to validate independently.

## M1 - Variant Module Extracted

Exit criteria:

- Variant request policy, image derivation, media type choice, and presentation
  ETag creation live in a private app Module.
- `ManagedArtworkAppService` no longer carries variant implementation details.
- Public image route behavior remains compatible.
- Redaction tests still prove storage handles, local paths, raw sources, cache
  URIs, and content hashes are not leaked.

## M2 - Artifact Store Module Extracted

Exit criteria:

- Local artifact path layout, inventory, classification, and delete outcomes are
  owned by the artifact store Module.
- App orchestration deals in domain records and store outcomes, not path layout.
- Lifecycle and remediation behavior remains compatible.

## M3 - Ingest Pipeline Module Extracted

Exit criteria:

- Fetch, validation, artifact write, and failure summary mapping are local to an
  ingest pipeline Module.
- Durable job claiming and commit ordering remain explicit at the app seam.
- No new runtime retry, cancellation, repair, or backoff behavior is introduced.

## M4 - Repository Adapter Locality Improved

Exit criteria:

- SQLite adapter code is split by Managed Artwork concern.
- Existing repository traits remain the external interface.
- SQL row mapping and constants are close to the queries they support.

## M5 - API Locality Audited

Exit criteria:

- Admin/Public Client DTO locality has been reviewed after app/db deepening.
- Any split preserves explicit DTO names and redaction tests.
- No generated SDK or OpenAPI contract changes occur unless intentionally
  documented.

## M6 - Lane Closed Or Split

Exit criteria:

- Fresh verification evidence is recorded.
- Residual product follow-ons are explicitly split instead of hidden in this
  architecture lane.
- `WORKSTREAM.json` and `HANDOFF.md` reflect the final continuation state.

Result:

- Completed. Final low-concurrency format, compile, API, DB, server, and
  whitespace gates passed.
- No residual follow-on was split from this architecture lane.
