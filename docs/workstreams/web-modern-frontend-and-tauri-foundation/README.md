# Web Modern Frontend And Tauri Foundation

Status: Active
Last updated: 2026-05-28

This lane turns the v0-generated Nako frontend direction into the product
frontend that ships from `web/`, with Tauri packaging as a first-class future
target. The existing `apps/admin-web` remains useful as a validation console and
contract smoke surface, but it is no longer the long-term product frontend.

Authoritative files:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [RUST_CAPABILITY_GAPS.md](RUST_CAPABILITY_GAPS.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [HANDOFF.md](HANDOFF.md)

Current executable task: `WMFT-030`, deepen the route shell and visual system
from the v0 UX direction with Browser/Playwright screenshots. `WMFT-020`
created the `web/` Vite React product app, package scripts, first Media/Admin
routes, Tauri shell skeleton, and Nako-owned Tauri icons.
