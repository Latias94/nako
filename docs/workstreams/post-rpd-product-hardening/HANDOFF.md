# Post-RPD Product Hardening — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The post-RPD product roadmap is open as an umbrella. It chooses
`metadata-provider-breadth` as the first execution lane and records NFO/link,
playback/transcode, managed import, network, AI, and addon distribution as
ordered follow-ons. `metadata-provider-breadth` is now complete, so the
umbrella has re-scored the next lanes.

## Active Task

- Task ID: PRPH-030
- Owner: planner
- Files: `docs/workstreams/post-rpd-product-hardening`, `docs/workstreams/metadata-provider-breadth`
- Validation: child closeout evidence reviewed; next-lane recommendation recorded
- Status: DONE
- Review: ready for `nfo-link-authority` opening
- Evidence: `docs/workstreams/post-rpd-product-hardening/DESIGN.md`

## Decisions Since Last Update

- Do not implement a generic downloads lane first.
- Treat downloads as `managed-import-staging` after metadata and local file
  authority are stronger.
- Start Wave 1 with metadata provider capability, matching, and conflict
  explanation rather than UI or AI breadth.
- Close Wave 1 before downloads/import because provider identity and ambiguity
  must be explicit first.
- Choose `nfo-link-authority` as the next mainline lane because it is the
  remaining high-risk local data-loss boundary.
- Keep playback/transcode ops hardening as a parallel sidecar candidate only
  if it stays diagnostic/runtime-focused and avoids NFO/import write scope.

## Blockers

- None for opening the next execution lane.

## Next Recommended Action

- Open `nfo-link-authority` as the next execution workstream.
- Keep `post-rpd-product-hardening` active until the next lane is represented
  by durable docs, or close it if the roadmap no longer reduces coordination
  cost.
