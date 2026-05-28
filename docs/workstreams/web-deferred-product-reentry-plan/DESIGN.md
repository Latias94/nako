# Web Deferred Product Reentry Plan - Design

Status: Completed
Last updated: 2026-05-28

## Problem

The new `web/` frontend is now a bounded Vite/TanStack/Tauri product shell with
route contracts, live Admin seams, connection profile handling, and bundle
budgets. WBBP intentionally removed v0-only product surfaces for downloads,
playlists, photos, music, podcasts, AI assistant, and automation because they
had no accepted runtime contracts.

Those gaps still matter. Without an explicit reentry plan, future work can
either forget them or reintroduce fixture-only pages that bloat the app and
drift from Nako's video-first architecture.

## Target State

Each deferred surface has:

- an accepted product owner surface: Media, Admin, Desktop, or future
  non-video client;
- a backend/API contract precondition;
- a relationship to existing workstreams where one already exists;
- a first executable frontend task or a clear "do not implement yet" gate;
- validation gates that prove runtime behavior instead of screenshots alone.

## Scope

In scope:

- map deferred web gaps to existing backend/product workstreams;
- define first frontend reentry slices for accepted video-first and admin
  workflows;
- keep non-video media domains behind explicit domain-baseline gates;
- define validation commands for future implementation lanes.

Out of scope:

- implementing downloads, playlist, AI, automation, music, photo, or podcast UI;
- creating new public API routes in this planning lane;
- restoring deleted v0 prototypes;
- widening the runtime bundle budget.

## Architecture Direction

The next frontend work should prioritize accepted video-first flows before
broader media domains:

1. Live Media browsing/detail/playback against Public Client contracts.
2. New `web/` Admin route parity for already accepted Admin operations such as
   Acquisition Intake and Generated Artifacts.
3. User-owned media state such as playlists only after a backend contract exists.
4. Non-video domains only after ADR-0021's video-first implementation has a
   domain-specific baseline lane.

AI and automation should not return as a free-form chat panel. They should enter
through Generated Artifact proposal/review and Addon/Automation diagnostics,
where Nako can preserve explicit acceptance, capability scopes, and auditability.

Downloads should not return as a Media client "download manager" until Nako has
an accepted acquisition/provider protocol. The near-term web surface is Admin
Acquisition Intake and Managed Import visibility.

## Risk Plan

- Product risk: fake UI can imply unsupported capabilities. Mitigation: require
  backend/API evidence before a surface leaves Deferred.
- Performance risk: reintroducing large optional domains can erase WBBP gains.
  Mitigation: keep bundle budgets mandatory and route-split new surfaces.
- Architecture risk: non-video domains can overload video-first catalog models.
  Mitigation: require a domain-baseline workstream before music/photo/podcast UI.

## Closeout

Completed by WDRP-070. Every deferred surface is now assigned to a follow-on
lane or an explicit deferred trigger; this planning lane does not own runtime
implementation.
