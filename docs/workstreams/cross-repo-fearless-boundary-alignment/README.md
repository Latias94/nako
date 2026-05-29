# Cross-Repo Fearless Boundary Alignment

Status: Completed
Last updated: 2026-05-29

This workstream is the next architecture-first refactor lane for aligning the
Nako server workspace with `../nako-official-addons`.

It exists because the previous Nako core and official addon refactor lanes
closed successfully, but their follow-on complexity now spans both sides of the
Addon Sidecar boundary. The server has strong crate-level seams, yet some
internal facades are still too wide. The official metadata scraper has working
provider breadth, yet scrape orchestration, protected writes, and large
provider adapters are becoming hard to evolve safely.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

Implementation closed on 2026-05-29. Remaining playback breadth, public addon
SDK/release work, broader provider acceptance coverage, and additional hardware
backend work are follow-on lanes, not hidden work in this lane.
