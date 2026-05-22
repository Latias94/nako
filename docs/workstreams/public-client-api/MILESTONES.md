# Public Client API Contract Milestones

Status: Completed
Last updated: 2026-05-17

## M29.0 Scope And Evidence Freeze

Status: completed.

Outcome: M29 has a dedicated workstream, migration rules, and a first proof
slice.

Exit criteria:

- Problem and target state are explicit.
- Public vs server-admin/internal scope is explicit.
- First proof slice is selected.
- Top-level docs point to the M29 lane.

Primary evidence:

- `docs/workstreams/public-client-api/DESIGN.md`
- `docs/workstreams/public-client-api/TODO.md`

## M29.1 Public Browse Protocol DTO Slice

Status: completed.

Outcome: first stable browse/list/detail/search DTOs live in
`nako-client-protocol`, with `nako-api` mapping server records into protocol
wire types.

Deliverables:

- Protocol DTOs for library list/source list, item list/detail, search, source
  probe, media item, metadata, source, and probe summaries.
- Public wire ID fields represented as strings.
- Adapter mapping functions in `nako-api`.
- Server catalog/library route behavior preserved.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-client-protocol --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`
- focused server route tests for catalog/library/system browse behavior
- `cargo tree -p nako-client-protocol`

## M29.2 Public Playback Decision DTO Slice

Status: completed.

Outcome: playback decision response uses protocol-owned wire DTOs rather than
`nako_streaming::PlaybackDecision`.

Deliverables:

- Protocol DTOs for playback decision, direct-play plan, transcode plan
  summary, playback mode, output container, and hardware acceleration.
- Adapter mapping functions in `nako-api`.
- Playback route behavior preserved.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`
- `cargo tree -p nako-client-protocol`

## M29.3 Contract Docs And Route Evidence

Status: completed.

Outcome: docs and route tests make the shipped public client API contract
auditable.

Exit criteria:

- Public route surfaces are mapped to protocol DTOs in
  `EVIDENCE_AND_GATES.md`.
- Server-admin/internal DTOs intentionally left in `nako-api` are listed as
  non-goals or follow-ons.
- Route-level tests cover browse/search/list/detail/playback JSON.

## M29.4 Closeout

Status: completed.

Outcome: M29 closes only after the prompt-to-artifact audit proves every
explicit requirement is complete or explicitly split as out of scope.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo tree -p nako-client-protocol`
- `git diff --check`
- Workstream status is updated to completed.
