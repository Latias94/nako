# Android Browse/Catalog Rust Core Routes — Milestones

Status: Closed
Last updated: 2026-05-21

## M0 — Scope Freeze

Exit criteria:

- Scope targets browse/catalog route construction only.
- Non-goals keep DTO decode, transport, UI, and server API shape out of scope.
- ADR 0032 boundary remains authoritative.

## M1 — Rust Core Browse Builders

Exit criteria:

- `taru-client-core` has explicit builders for `TaruBrowseClient` route family.
- Builders produce `CoreHttpRequest` with auth, safe preview, path encoding, and
  query encoding.
- Core tests cover stable URLs for representative routes.

## M2 — UniFFI Browse Surface

Exit criteria:

- `taru-client-uniffi` exposes thin browse request builder bindings.
- Boundary guard still passes.
- UniFFI tests cover at least search and facet routes.

## M3 — Android Browse Migration

Exit criteria:

- `TaruBrowseClient` runtime route construction uses `BrowseCore`/Rust core.
- Kotlin SDK DTO decode and Android diagnostics remain unchanged.
- Browse JVM tests pass.

## M4 — Integration Verification And Docs

Exit criteria:

- README/workstream docs explain the new route ownership.
- Combined Rust, UniFFI, guard, and Android browse gates pass.

## M5 — Closeout

Exit criteria:

- Closeout records final gates, residual risks, and follow-ons.
- Workstream JSON and markdown agree on closed state.
