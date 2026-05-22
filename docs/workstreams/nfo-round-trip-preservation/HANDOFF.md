# NFO Round Trip Preservation Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M47 is complete. `nako-nfo` now has preservation-aware movie NFO rendering and
forced export uses it when updating an existing sidecar.

## Implemented

- `NfoCodec::render_preserving` is the update path for existing XML.
- `MovieNfoCodec` preserves unknown top-level movie XML elements, comments,
  and processing instructions using parser source ranges.
- Nako-owned movie fields are rendered from `NfoDocument`.
- Duplicate owned fields and release-date aliases are reported through
  `NfoPreservationReport`.
- Forced export reads existing sidecar XML and uses preservation-aware output.
- New sidecar export still uses deterministic fresh rendering.

## Validation

Passed:

```powershell
cargo fmt --all -- --check
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
```

## Follow-ons Outside M47

- Nested unknown XML preservation if real-world NFO samples require it.
- Public export conflict diagnostics if UI/API needs them.
- VFS atomic write, backup, soft-link, or hard-link policy.
- Broader Jellyfin/Kodi/Plex compatibility profiles.
