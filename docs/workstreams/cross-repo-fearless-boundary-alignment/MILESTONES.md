# Cross-Repo Fearless Boundary Alignment - Milestones

Status: Active
Last updated: 2026-05-24

## M0 - Scope And Evidence Freeze

Exit criteria:

- The workstream target state and non-goals are accepted.
- Both repository worktree states are recorded.
- Active or dirty workstreams that can conflict are named.
- No implementation refactor begins before scope is accepted.

Primary evidence:

- `docs/workstreams/cross-repo-fearless-boundary-alignment/DESIGN.md`
- `docs/workstreams/cross-repo-fearless-boundary-alignment/TODO.md`
- `docs/workstreams/cross-repo-fearless-boundary-alignment/HANDOFF.md`

## M1 - Server Workflow Port Deepening

Exit criteria:

- One server workflow depends on a narrower, workflow-shaped port.
- SQLite/PostgreSQL behavior remains covered for touched persistence behavior.
- Local Inference or Metadata Acceptance has a deeper Module proof.
- Public behavior remains unchanged unless an ADR/workstream explicitly records
  the change.

Primary gates:

- Focused `cargo nextest run -p <server-package> <filter>`.
- Focused `nako-db` contract tests when persistence changes.
- `cargo fmt --all -- --check` when practical.

## M2 - Official Addon Runtime Alignment

Exit criteria:

- `MetadataScrapeRuntime` responsibilities are split without changing public
  payload behavior.
- At least one large provider adapter is split around real behavioral seams.
- Addon protected-write client/protocol ownership is clarified.
- Active provider-quality lanes are not overwritten.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper metadata writeback artwork ranking --no-fail-fast`.
- Focused provider tests for the split adapter.
- `cargo fmt --all -- --check` in `../nako-official-addons` when practical.

## M3 - Playback Runtime Ownership

Exit criteria:

- Playback Runtime owns product-level session lifecycle and diagnostics.
- `nako-transcode` remains a lower-level execution API.
- Hardware acceleration and fallback vocabulary are clearer for future VAAPI,
  NVENC, and QuickSync work.
- Adaptive bitrate and optimized-version breadth are split unless they are the
  explicit proof.

Primary gates:

- Focused playback/transcode nextest gates.
- API/diagnostic contract tests if outputs change.

## M4 - Contract And Closeout

Exit criteria:

- API/protocol drift risks introduced by earlier slices are handled or split.
- Fresh verification evidence is recorded.
- Review findings are closed or documented as non-blocking.
- `WORKSTREAM.json` reflects final status.
- Remaining breadth becomes named follow-ons.

Primary gates:

- Relevant focused nextest gates from M1-M3.
- Broader workspace gates where practical and proportional.
- `git diff --check` for touched paths only when unrelated dirty files exist.
