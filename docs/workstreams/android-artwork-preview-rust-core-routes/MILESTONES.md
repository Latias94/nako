# Android Artwork And Preview Rust Core Routes — Milestones

Status: Closed
Last updated: 2026-05-22

## M0 — Scope Frozen

Exit criteria:

- Workstream docs exist.
- Scope is limited to artwork route construction and preview/test route fixture
  cleanup.
- ADR 0032 boundary is preserved.

Status: Complete.

## M1 — Rust Core Artwork Builder

Exit criteria:

- `taru-client-core` exposes selected artwork image request construction.
- `taru-client-uniffi` exposes a thin FFI-safe binding.
- Rust tests cover encoded image ids, auth, safe preview redaction, and optional
  width/height query parameters.

Status: Complete.

## M2 — Android Artwork Runtime Migration

Exit criteria:

- `PublicArtworkSource` no longer imports or calls generated SDK route
  descriptors.
- Android selected artwork request validation still rejects unsafe or mismatched
  DTO URLs.
- Android artwork/resolver tests pass.

Status: Complete.

## M3 — Preview Fixture Cleanup

Exit criteria:

- `TaruBrowseShellPreview` uses local preview route helpers instead of generated
  SDK descriptors.
- Dead generated descriptor adapter code is removed if no callers remain.
- App `src/main` contains no generated route descriptor imports.

Status: Complete.

## M4 — Integration Verified

Exit criteria:

- Rust core and UniFFI gates pass.
- UniFFI boundary guard passes.
- Targeted Android artwork/resolver tests pass.
- Android compile gate passes.
- Route-owner scans prove no generated route descriptor use in Android
  `src/main` runtime/preview code.

Status: Complete.

## M5 — Closed

Exit criteria:

- TODO ledger is complete.
- Evidence and gates are current.
- Workstream JSON is valid.
- Closeout notes document residual risks and follow-ons.

Status: Complete.
