# Nako Product Context

register: product

Last updated: 2026-05-26

## Product Purpose

Nako is an open-source, self-hosted media home for organizing, keeping, and
playing films, shows, anime, and personal collections. The product should feel
personally owned and operationally transparent: a private media home that helps
an operator understand what the server knows, what it is doing, and where it
needs a deliberate decision.

The Admin Web surface is an administration and media-governance console. It is
not the flagship playback client, not a streaming storefront, and not a generic
SaaS dashboard.

## Primary Users

- Self-hosted operators who run Nako for themselves, family, or a small trusted
  group.
- Power users who curate libraries, metadata authority, NFO sidecars, artwork,
  playback behavior, and addon permissions.
- Nako contributors who need a faithful visual map of backend capabilities and
  Admin API gaps.

## Product Principles

- Use Nako domain language. Prefer Media Library, Media Source, Media Item,
  Canonical Metadata, Provider Mapping, Local Inference, NFO, Playback Source
  Selection, Addon Sidecar, and Automation Provider.
- Make authority visible. The UI should show whether data is live, mocked,
  planned, local, provider-backed, addon-backed, or NFO-backed.
- Protect sensitive information by default. Secrets, tokens, raw credentials,
  unsafe local paths, raw provider bodies, and addon-hosted pages must not be
  casually exposed.
- Prefer operator confirmation for broad or destructive workflows. Dry-run and
  review states are part of the product, not afterthoughts.
- Keep web administration separate from playback-client design. Light browsing
  is allowed when it supports governance, but watching-first workflows belong
  to client applications.
- Connect media and administration through permission-gated context links. An
  administrator should be able to jump from a media problem to scan, metadata,
  playback, job, session, or settings workflows without exposing Admin API
  data to ordinary viewers.
- Preserve self-hosted clarity. Network access, reverse proxy, tunnel provider,
  and addon lifecycle boundaries should describe what Nako owns and what the
  operator owns.

## Brand And Tone

Nako should feel calm, careful, and technically trustworthy. The accepted brand
direction is a private media home with a restrained, approachable identity,
not a corporate streaming platform. Copy should be concise and practical:
clear nouns, direct verbs, safe diagnostics, and no marketing filler inside the
admin console.

Public brand language may use the tagline "Your media home, gently kept." The
admin console should be more task-focused than the public tagline, with labels
that help operators act quickly.

## Anti-References

- Streaming storefronts that optimize for posters and consumption over
  administration.
- Cloud SaaS dashboards with generic hero metrics and decorative cards.
- File-manager UIs that reduce Nako concepts to folders and paths.
- Provider-centric UIs that make TMDB, Douban, Bangumi, or NFO concepts appear
  more authoritative than Nako's Canonical Metadata model.
- Plugin dashboards that imply in-process trust for Addon Sidecars.

## Strategic Surfaces

- Admin Web: server administration, diagnostics, media governance, addons,
  automation, metadata authority, storage, network, and settings.
- Media Web: browser-based browsing, playback, personal state, and local-media
  interaction through the Public Client API. It may expose admin context links
  only for principals with the required role.
- Desktop Client: a packaged media client that may reuse Media Web UI, but
  should use a native playback core for robust local media playback when
  WebView playback is insufficient.
- Mobile Client Applications: native playback-first clients that consume the
  Public Client API and keep server administration outside the main mobile
  experience.
- Addon surfaces: external Addon Hosted Pages and declared Addon Entry Points
  that Nako can surface without treating addon UI as trusted admin code.
