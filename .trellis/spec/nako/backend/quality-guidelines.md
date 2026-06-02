# Quality Guidelines

The `nako` crate should stay boring, stable, and intentionally narrow. Its
quality bar is public API clarity rather than implementation depth.

## Required Patterns

- Keep the crate a small facade over public protocol and SDK crates.
- Keep default features empty so consumers opt into optional clients.
- Preserve `addon-client = ["dep:nako-addon-client"]` style feature gating.
- Keep README examples synchronized with actual public imports.
- Keep crate docs explicit that this is not the Nako server implementation.
- Add focused tests for public visibility when exports change.

## Forbidden Patterns

- Do not add server-side logic, runtime setup, database access, HTTP route
  handling, storage access, media probing, or playback planning.
- Do not expose AGPL server internals through the permissive facade.
- Do not add broad workspace re-exports for convenience.
- Do not make optional SDK helpers part of default features without an explicit
  product decision.

## Tests Required

- Re-export smoke tests for always-on public modules.
- Feature-enabled checks for optional modules.
- README example review when import paths or feature flags change.

## Gate Selection

- Focused:
  `cargo nextest run -p nako --no-fail-fast`
- Optional client:
  `cargo check -p nako --features addon-client --tests`

## Review Checklist

- Is the facade still small enough to understand from `src/lib.rs`?
- Did the change avoid pulling server or persistence dependencies?
- Are public examples, crate metadata, and feature flags consistent?
- Is the owning crate a better place for the behavior?
