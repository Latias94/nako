# Addon Manager Lifecycle Automation - Handoff

Status: Completed
Last updated: 2026-05-23

## Current State

The alpha manual addon loop is proven. Nako can host the published server
image, the public Addon Protocol crates are published, and the official
metadata scraper can be installed from crates.io and run through the published
smoke script.

This lane is complete. Nako now exposes the first manager-owned registry/plan
slot through `GET /admin/v1/addons/{addon_id}/manager-plan` and
operator-confirmed `POST /admin/v1/addons/{addon_id}/manager-plan` intents for
`install`, `update`, and `remove`. The surface combines registration detail,
Addon Health Check, Addon Token summaries, accepted grants, and Addon Install
Guide output without leaking raw token material or taking ownership of sidecar
processes.

## Next Task

Open a follow-on lane when one of the remaining product areas is ready.

Recommended follow-ons:

- Addon source catalog / marketplace hosting;
- package signing and trust-root policy;
- provider breadth beyond the first official companion addon;
- rollback and update-policy execution beyond the current plan slot;
- process/container supervision, if Nako decides to own sidecar execution.

## Known Risks

- The existing published addon smoke must stay valid while follow-on manager
  lanes evolve.
- Rollback/update-policy execution still needs a separate test fixture if it
  grows beyond the current redaction-safe plan surface.
- Process/container supervision should not be added implicitly through manager
  APIs; it needs its own authority and operator-risk review.
