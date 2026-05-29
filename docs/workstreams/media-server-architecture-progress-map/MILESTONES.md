# Media Server Architecture Progress Map Milestones

Status: Closed
Last updated: 2026-05-29

## Milestone 1 - Architecture Authority Located

Status: Done

- Existing glossary, ADR, roadmap, and workstream authority were reviewed.
- Documentation-only scope was frozen.

## Milestone 2 - Architecture Map Added

Status: Done

- `docs/ARCHITECTURE.md` now records Nako's north star, system map, maturity
  matrix, and next pressure points.
- `docs/architecture/*.md` now owns detailed capability maps.

## Milestone 3 - HLS Runtime Boundary Recorded

Status: Done

- ADR 0052 records FFmpeg CLI-first HLS runtime ownership and
  manifest-backed artifact publication.

## Milestone 4 - Planning Docs Updated

Status: Done

- Roadmap and indexes point to the new architecture authority.
- Workstream is closed with evidence.

## Milestone 5 - Control Plane And Workstream Links Added

Status: Done

- ADR 0053 records the application control-plane boundary.
- `docs/architecture/CONTROL_PLANE.md` maps addon lifecycle, observability,
  durable jobs, remote access, API scale, and cache-contract concerns.
- `docs/architecture/WORKSTREAM_LINKS.md` links architecture capability areas
  to workstream evidence and proposed lanes.
- Future workstreams now have an `architecture_refs` / `capability_tags`
  convention.
