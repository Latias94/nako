# Addon Protected Writes Milestones

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Outcome: concrete protected-write work is split from the completed ATGSE
token/grant/intake lane.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- Existing ATGSE closeout points to this lane.
- Workstream index links the new lane.
- Docs-only validation passes.

Primary evidence:

- `docs/workstreams/addon-protected-writes/DESIGN.md`
- `docs/workstreams/addon-protected-writes/TODO.md`
- `docs/workstreams/addon-token-grants-side-effects/HANDOFF.md`

## M1 - Protected Write Seam Audit

Outcome: existing write seams are classified before adding effect-specific
apply behavior.

Result: completed on 2026-05-18. Canonical Metadata was selected as the first
concrete apply target, with explicit side-effect apply outcome state and Addon
metadata source attribution required before APW-030 writes domain state.

Exit criteria:

- Addon Side Effect intake, metadata merge, catalog commit, artwork, subtitle,
  NFO, and storage/VFS boundaries are inventoried.
- The first apply target is selected with file anchors and risk notes.
- ADR amendment need is accepted, rejected, or split.

Primary gates:

- `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`
- `git diff --check`

## M2 - Canonical Metadata Apply Slice

Outcome: one accepted Addon Side Effect can apply a bounded Canonical Metadata
write through Taru-owned domain seams.

Exit criteria:

- The route/service converts a valid accepted side effect into a narrow domain
  command instead of mutating database rows directly.
- Idempotency covers replay after the apply result is known.
- Audit and safe response summaries distinguish intake validation from apply
  outcome.
- Catalog/search projection consistency is proven or the slice is explicitly
  narrowed to avoid claiming it.

Primary gates:

- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests`
- focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- relevant metadata/catalog tests selected by APW-020
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 - Managed Artwork And Artifact Intake

Outcome: addon-submitted artwork or artifact output has a Taru-owned intake and
storage path.

Exit criteria:

- Artwork output is represented as an Artwork Candidate, Managed Artwork, or
  Taru-Managed Artifact rather than a raw provider URL.
- External fetch ownership, storage budget, provenance, and redacted response
  behavior are documented and tested.
- Any image processing/cache policy breadth is completed or split.

Primary gates:

- focused artwork/addon tests selected by APW-020
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
- `git diff --check`

## M4 - Subtitle, NFO, And Library File Write Policy

Outcome: addon-initiated sidecar file changes use Taru Library File Write
policy.

Exit criteria:

- Subtitle, NFO, and sidecar-asset writes do not expose raw paths to Addon
  Sidecars.
- NFO writes preserve NFO Round Trip and backup policy when relevant.
- Storage/VFS write mode, backup, diagnostics, and redaction behavior are
  recorded.
- Oversized subtitle or NFO breadth is split before it hides the protected-write
  apply model.

Primary gates:

- focused NFO/storage/addon tests selected by APW-020
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## M5 - Closeout Or Split

Outcome: the protected-write apply model is proven and remaining breadth is
closed, deferred, or split.

Exit criteria:

- `EVIDENCE_AND_GATES.md` records fresh command evidence.
- HTTP/API docs reflect shipped protected-write behavior.
- Remaining metadata/artwork/subtitle/NFO/Library File Write breadth is
  completed, deferred, or split.
- `WORKSTREAM.json` status and `HANDOFF.md` match reality.
