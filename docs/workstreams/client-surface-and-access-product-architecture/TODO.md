# Client Surface And Access Product Architecture - TODO

Status: Draft
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

- [x] CSAPA-010 [owner=planner] [deps=none] [scope=docs/workstreams/client-surface-and-access-product-architecture]
  Goal: Freeze the product architecture problem, target state, non-goals, and evidence anchors for Nako client surfaces and access.
  Validation: DESIGN.md, README.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/client-surface-and-access-product-architecture/DESIGN.md
  Handoff: Open narrower execution lanes before implementation work starts.

## M1 - Identity And Access Contract

- [ ] CSAPA-020 [owner=unassigned] [deps=CSAPA-010] [scope=docs/adr,docs/workstreams,crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server]
  Goal: Define the first post-Single-Admin account model: local users, roles, Library Access, login/session shape, registration defaults, and Admin API readiness.
  Validation: New or updated ADR/design docs plus focused Rust contract tests when implementation starts.
  Review: review-workstream before accepting any account persistence or route shape.
  Evidence: Identity/access follow-on DESIGN.md and ADR if route/auth semantics change.
  Handoff: Do not expose Admin Web account CRUD until backend authority exists.

## M2 - Media Web Foundation

- [ ] CSAPA-030 [owner=unassigned] [deps=CSAPA-020] [scope=docs/workstreams,apps/media-web or accepted route namespace]
  Goal: Split the first browser Media Web lane for local media browsing and playback through Public Client API.
  Validation: Product design lists first routes, Public Client API gaps, auth/session expectations, player limitations, and no Admin API dependencies.
  Review: UX/product review before code scaffold.
  Evidence: Media Web foundation workstream.
  Handoff: Keep recommendations, online discovery, and streaming-storefront features out of the first slice.

## M3 - Management Context Links

- [ ] CSAPA-040 [owner=unassigned] [deps=CSAPA-020,CSAPA-030] [scope=apps/admin-web,apps/media-web,docs]
  Goal: Define and implement the first permission-gated links between media consumption and management actions.
  Validation: Route/link matrix, role gating tests, redaction tests, and browser smoke once implemented.
  Review: review-workstream for boundary leakage and UX consistency.
  Evidence: Management Context Links route matrix.
  Handoff: Admin Web owns review/confirmation for broad or destructive actions.

## M4 - Desktop Playback Strategy

- [ ] CSAPA-050 [owner=unassigned] [deps=CSAPA-030] [scope=docs/workstreams,apps/desktop or accepted package]
  Goal: Split a Tauri desktop playback spike comparing WebView playback against Tauri plus native playback core.
  Validation: Spike records codec/subtitle/hardware-acceleration evidence, packaging risks, and recommended player core.
  Review: Architecture review before committing to a desktop runtime.
  Evidence: Desktop playback spike workstream.
  Handoff: Admin Web packaging remains separate from playback-client packaging.

## M5 - Closeout

- [ ] CSAPA-060 [owner=planner] [deps=CSAPA-020,CSAPA-030,CSAPA-040] [scope=docs/workstreams/client-surface-and-access-product-architecture]
  Goal: Close this planning lane once narrower execution lanes own identity, Media Web, context links, and desktop playback.
  Validation: EVIDENCE_AND_GATES.md lists accepted follow-ons and any deferred risks.
  Review: review-workstream for workstream compliance.
  Evidence: WORKSTREAM.json and HANDOFF.md.
  Handoff: Keep this lane as product architecture, not a dumping ground for implementation tasks.
