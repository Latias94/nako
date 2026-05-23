# Addon Manager Lifecycle Automation - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The alpha manual addon loop is proven. Nako can host the published server
image, the public Addon Protocol crates are published, and the official
metadata scraper can be installed from crates.io and run through the published
smoke script.

What remains is the first manager-owned lifecycle slice. Nako still requires
manual sidecar management; this lane exists to decide and build the first
manager boundary without collapsing marketplace, package signing, or provider
breadth into the initial implementation.

## Next Task

Start AMG-010.

Goal: freeze the Addon Manager problem, target state, non-goals, and the first
manager-owned lifecycle slice.

Suggested first steps:

1. Re-read ADR 0020 and ADR 0033 for the existing addon boundary.
2. Define the first manager-owned lifecycle slot in plain Nako terms.
3. Keep marketplace, package signing, and provider breadth out of the first
   slice.
4. Record the split/follow-on boundaries before adding implementation tasks.

## Known Risks

- A manager lane can accidentally absorb marketplace, package signing, or
  provider breadth if the first slice is not narrow.
- The existing published addon smoke must stay valid while the manager lane
  evolves.
- Process supervision and update rollback will likely need their own test
  fixtures once code starts.
