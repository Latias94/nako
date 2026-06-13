---
title: "Nako Product Positioning"
type: plan
status: proposed
date: 2026-06-13
origin: docs/research/nako-product-competitive-analysis/competitive-analysis-first-pass.md
---

# Nako Product Positioning

## Summary

Nako should not position itself as "Rust Jellyfin." Nako should position
itself as an auditable, self-hosted, extensible media server backend and
control plane for users who care about local authority, data portability,
safe extension boundaries, and operator-grade diagnostics.

The near-term promise is not full Jellyfin/Plex replacement parity. The
near-term promise is a trustworthy path for one self-hosted operator to
configure a Media Library, scan, browse, play, diagnose, repair, and extend the
server without surrendering control to a central account service or arbitrary
in-process plugins.

## Problem Frame

Mature users compare media servers through visible surfaces: clients, playback,
remote access, plugin ecosystem, metadata quality, library migration, and
operational recovery. Nako already has strong backend architecture and
control-plane foundations, but that strength can be misread if the product
message is only "Jellyfin/Plex-class."

The positioning must turn Nako's architectural choices into product language:

- local metadata and NFO are first-class authority, not import/export afterthoughts;
- Addons are out-of-process sidecars with scoped grants, not native plugins;
- playback uses typed planning and explainable reasons before runtime work;
- resource budgets, durable jobs, redaction, and diagnostics are product
  behavior;
- Nako is video-first now, but not permanently video-only.

## Positioning Statement

For self-hosted media operators who want ownership, auditability, and
portability over their personal media library, Nako is a media server backend
and control plane that keeps library state, playback decisions, Addon side
effects, and operational recovery explicit.

Unlike Plex, Nako should not require central accounts or subscription-gated
local/remote playback for the core self-hosted experience.

Unlike Jellyfin, Nako should not make its core process depend on in-process
plugin code or provider-specific object models.

## Target Users

Primary users:

- NAS, home-server, and Docker/Compose operators.
- Existing Jellyfin, Plex, or Emby users who are unhappy with account
  dependency, plugin trust, metadata drift, or migration friction.
- Anime, Asian media, AV, subtitle-heavy, and NFO-heavy collectors who need
  provider breadth and field-level metadata control.
- Power users who want resource search, acquisition, subtitle, notification,
  metadata, and renderer behavior to be auditable workflows.

Secondary users:

- Addon authors who want a stable protocol and conformance harness instead of
  host-internal plugin APIs.
- Client authors who want Public Client API and SDK contracts.
- Operators of family or household servers who need future User, Role, and
  Library Access concepts without making Single-Admin Mode permanent.

Not the first target:

- Users who primarily want a polished TV app with no self-hosting knowledge.
- Users who depend on Plex-style cloud discovery, social sharing, or commercial
  streaming aggregation.
- Users who expect the server to manage Docker, systemd, Kubernetes, NAT relay,
  and package updates in the first release track.

## Product Principles

1. **Local authority first.** NFO, sidecars, user edits, field locks, local
   inference evidence, and provider IDs should be durable and portable.
2. **Direct Play first.** Transcode is a fallback selected by typed facts, not
   an opaque runtime attempt.
3. **Planner before runtime.** Metadata, playback, transcode, Addon, and
   acquisition decisions should become typed plans before they mutate state,
   spawn processes, or expose URLs.
4. **Host-owned side effects.** Addons may suggest, discover, and execute
   bounded tasks, but Nako owns permissions, audit, storage writes, catalog
   mutation, and playback runtime.
5. **Diagnostics are product UX.** A feature is not mature until operators can
   understand failures without reading raw logs, local paths, tokens, provider
   payloads, or FFmpeg command lines.
6. **Ecosystem integration beats overbuilding.** Servarr, Bazarr, Seerr,
   Kometa, Tdarr, Tautulli, Maintainerr, WatchState, and linear-TV tools should
   be treated as ecosystem integration points before Nako tries to absorb every
   workflow.

## Alternatives Considered

### Option A: Rust Jellyfin Replacement

How it works:

- Position Nako as a Jellyfin-compatible or Jellyfin-equivalent server.
- Chase broad parity across clients, plugins, Live TV, metadata, playback, and
  community features.

Pros:

- Easy for users to understand.
- Clear feature checklist.

Cons:

- Sets expectations Nako cannot meet during alpha/beta.
- Pushes the project toward copying product shape instead of preserving Nako's
  domain model.
- Undervalues Addon Sidecar, local authority, and control-plane work.

Decision: rejected.

### Option B: Plex Alternative For Everyone

How it works:

- Emphasize polished clients, remote access, family sharing, discovery, and
  consumer-friendly onboarding first.

Pros:

- Strong mainstream appeal.
- Targets visible user pain after Plex pricing/remote-access changes.

Cons:

- Requires broad client coverage and remote-access support before the backend
  is product-ready.
- Risks creating a first-party relay or central account commitment too early.
- Competes directly with Plex where Plex is strongest.

Decision: rejected for the next 12-18 months.

### Option C: Auditable Self-Hosted Media Backend And Control Plane

How it works:

- Position Nako around local authority, explicit media workflows, Addon
  sidecar safety, playback explainability, and operator diagnostics.
- Build toward Jellyfin/Plex-class capability without copying either product's
  trust boundary or business model.

Pros:

- Matches current architecture and research findings.
- Gives Nako a defensible reason to exist before client parity is complete.
- Makes Addon, VFS, metadata, playback, jobs, and diagnostics coherent product
  investments.

Cons:

- Less immediately understandable than "Jellyfin replacement."
- Requires careful messaging so users do not mistake it for a backend-only SDK.

Decision: recommended.

## Success Metrics

| Metric | Current | Target | Measurement |
| --- | --- | --- | --- |
| Product positioning clarity | Research only | README and docs use the same positioning terms | README/docs review |
| First-operator journey | Partial alpha | One documented path from install to scan, browse, play, diagnose | M1 ladder evidence |
| Addon trust clarity | Strong architecture, weak product packaging | Addon docs explain sidecar, grants, side effects, and non-goals in product language | Addon author/operator docs review |
| Migration signal | Research only | NFO/artwork/provider ID/state portability plan exists | Migration/interop plan |
| Remote access clarity | Planned cookbook | Reverse proxy/VPN/tunnel guidance with diagnostics | Deployment docs |
| Ecosystem strategy | Research only | Clear build/integrate/defer decisions for major adjacent tools | Gap matrix and roadmap updates |

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Users read Nako as incomplete Jellyfin | High | High | Use explicit positioning and parity matrix; avoid replacement language before beta |
| Backend-first messaging feels too abstract | Medium | Medium | Tie every architecture point to operator/user workflows |
| Product scope expands into every adjacent tool | High | Medium | Maintain build/integrate/defer decisions in the gap matrix |
| Addon safety is perceived as weaker plugin capability | Medium | Medium | Show concrete Addon workflows, health, grants, and diagnostics |
| Remote access expectations exceed current support | High | Medium | Document supported network shapes and explicitly defer first-party relay |

## Source Documents

- [Competitive analysis first pass](../research/nako-product-competitive-analysis/competitive-analysis-first-pass.md)
- [Nako current state](../research/nako-product-competitive-analysis/nako-current-state.md)
- [External competitive ecosystem supplement](../research/nako-product-competitive-analysis/external-competitive-ecosystem-supplement.md)
- [Architecture map](../ARCHITECTURE.md)
- [Roadmap](../ROADMAP.md)
