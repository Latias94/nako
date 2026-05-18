# Android Client UX Context

Status: Proposed
Last updated: 2026-05-17

This file records the Android client experience baseline for the first
implementation lane. It is product and UX context for this workstream, not a
domain glossary. `CONTEXT.md` remains authoritative for Taru terms.

## Reference Scope

`repo-ref/findroid` is a reference for feature coverage and information
architecture only.

Taru may study Findroid's:

- native Android media-client shape;
- setup, home, media, detail, download, settings, and player page families;
- playback capability checklist such as PiP, chapters, subtitles, audio
  tracks, offline playback, gestures, and skip controls;
- phone/tablet and later TV split.

Taru must not copy Findroid source, resources, images, branding, layouts,
generated code, or Jellyfin-specific object model. Taru UI and client state
must use Taru language: **Media Library**, **Media Item**, **Media Source**,
**Playback Source Selection**, **User Playback State**, and **Public Client
API**.

## First Experience Principle

The first Android client should be playback-first, not administration-first.

A user should be able to:

1. connect to a Taru server;
2. authenticate;
3. find a playable Media Item;
4. understand which source/version will play when there are multiple choices;
5. start playback;
6. recover from common playback or connection errors.

Server administration, metadata editing, addon management, automation,
webhook configuration, provider diagnostics, and storage diagnostics belong to
later surfaces, likely web or admin-specific clients first.

## Server Profile Baseline

The Android client may store multiple server profiles, but one server is
active at a time. Home, Search, cache, playback, and future Downloads are
scoped to the active server.

A server profile is client-side connection state, not a Taru **User**.

First-version server profile fields:

- display name;
- base URL;
- token reference in secure storage;
- last observed API version;
- last successful connection time;
- last public error, if any.

First-version behavior:

- Settings can switch the active server;
- setup can add or re-authenticate a server profile;
- data and future offline downloads are isolated by server profile;
- no cross-server Home, Search, Continue Watching, cache, playback, or
  Downloads aggregation.

## Authentication Baseline

First-version auth is server URL plus access token, matching Taru's current
inbound bearer-token boundary.

The UI should call this credential an access token or server access token,
not "bearer token" in normal user-facing copy.

First-version connection fields:

- Server URL;
- Access Token;
- Test Connection;
- Save.

First-version connection errors:

- invalid URL;
- server unreachable;
- unauthorized;
- unsupported API version;
- TLS or certificate error when recognizable.

Credential rules:

- store token values in Android secure storage;
- hide token values by default;
- support paste from clipboard;
- never print token values in logs, diagnostics, screenshots, or safe request
  previews.

Deferred auth models:

- username/password login;
- OAuth/OIDC;
- QR login;
- device-code flow;
- user picker;
- client-side permission management.

## Error And Empty State Baseline

First-version errors must be actionable and client-safe. Raw protocol or server
codes may be available as diagnostics, but user-facing copy must explain the
state and the next action.

Setup and auth states:

- invalid URL: explain expected URL shape;
- server unreachable: suggest checking address and network;
- unauthorized: explain that the access token is invalid or expired;
- unsupported API version: explain server/client incompatibility;
- TLS or certificate error: explain HTTPS or certificate issues when
  recognizable.

Browse and Search states:

- empty library: explain that the library has no visible content;
- no search results: suggest changing the search term;
- loading timeout: offer retry;
- permission denied: explain that the current token cannot access the content.

Detail and Playback states:

- no playable source: explain that no playable Media Source is available;
- source unavailable: explain that the source is missing or remote storage is
  unavailable;
- unsupported media: explain device or player incompatibility;
- transcode, remux, or HLS failure: explain that server playback processing
  failed;
- session expired or cancelled: explain that the playback session ended;
- network interrupted: offer retry.

Every first-version error state should provide at least one useful action:

- Retry;
- Change server;
- Edit access token;
- Back to library;
- Choose another source;
- Copy diagnostics.

Diagnostics must not include token values, secret references, local filesystem
paths, FFmpeg commands, raw provider payloads, or server-local output paths.

## First Page Set

### Setup Flow

- Server Connect
- Token Login / Auth Check
- Server Switcher, model reserved but UI may be deferred

### Main Shell

- Home
- Libraries
- Search
- Settings

### Content Flow

- Library Detail
- Media Item Detail
- Browse Facet Result
- Series Detail
- Season Detail
- Episode Detail
- Source / Version Picker when one Media Item has multiple playable Media
  Sources or Source Variants

### Playback Flow

- Player
- Track / Subtitle Sheet
- Playback Error Sheet

## Home Baseline

Home is a playback launchpad, not an admin dashboard.

First-version stable anchors:

- Libraries, as the reliable structural browse entry point;
- Search, as the fastest route to a known title;
- Current server status and server switching as lightweight context only.

Enhanced sections may appear when Public Client API support exists:

- Continue Watching / Resume from **User Playback State**;
- Recently Added / Latest;
- Next Up for series.

Home must not depend on unavailable playback-state APIs or fake local-only
state to feel complete. If resume/latest data is unavailable, Home should still
be useful through Libraries and Search.

First-version non-goals for Home:

- recommendation algorithms;
- personalized ranking;
- favorites rail;
- person or collection discovery;
- admin job status;
- metadata refresh or addon/automation cards.

## Resume And Playback State

Resume UX is part of the intended playback loop, but authoritative **User
Playback State** belongs to the **Public Client API**.

When Public Client API support exists, Android should:

- report player position periodically;
- report final position on exit when possible;
- show Resume on detail pages;
- show Continue Watching on Home.

When Public Client API support is incomplete, Android may keep device-local
transient position only as a temporary convenience scoped to the active server
profile and device.

The client must not:

- invent an authoritative client-only watch-state model;
- show device-local transient position as cross-device Continue Watching;
- mix playback state across server profiles;
- sync offline playback progress without a reconciliation design;
- treat external-player progress as reliable unless a later handoff design
  proves it.

## Detail Page Baseline

Media Item Detail is a playback decision surface.

It should present enough **Canonical Metadata** to identify the item and enough
client-safe source facts to choose the right playable source without exposing
server internals.

First-version priorities:

- Play / Resume / Play Episode as the primary action;
- Source / Version selection when multiple playable choices exist;
- title, year, duration, overview, genres, tags, ratings, and content rating
  when available;
- compact Cast & Crew preview when supported by public data;
- tappable genre, tag, person, and collection chips that lead into supported
  browse results;
- basic technical facts surfaced near playback choice instead of as an
  exhaustive MediaInfo dump;
- clear hierarchy navigation for series, seasons, and episodes;
- explainable empty/error states for no playable source, auth failure,
  incompatible playback, or transcode failure.

First-version non-goals for detail pages:

- metadata editing;
- NFO state or provider mapping diagnostics;
- full people/credits exploration or biography pages;
- review/comment systems;
- administrator actions;
- raw storage or FFmpeg diagnostics.

## Browse Facet Result Baseline

Browse Facet Result is the reusable screen for supported metadata-driven
exploration.

It should be used when a user taps a genre, tag, person, studio, or collection
chip from Media Item Detail, Library Detail, or any other supported browse
surface.

First-version purposes:

- show a supported facet value as the active browse scope;
- list matching Media Items across the active server profile and the allowed
  library scope;
- preserve the user's ability to open the matching Media Item detail or related
  hierarchy page;
- keep the same empty, loading, unauthorized, and unreachable behaviors as
  other browse surfaces.

First-version supported facet families:

- Genre;
- Tag;
- Person, initially limited to role-aware entry points such as Actor,
  Director, and Writer when the backing data supports them;
- Studio;
- Franchise Collection or other supported collection-like grouping;
- Year or Release Year when backed by explicit public data;
- Media Item kind when needed for result grouping.

First-version non-goals:

- arbitrary advanced filter builders;
- saved facet searches;
- editable people or collection pages;
- biography, credits, or filmography products;
- client-invented database-column browsing.

Browse Facet Result must only expose facets that the Public Client API
explicitly supports.

## Source / Version Picker Baseline

Source / Version Picker is part of the first playback loop whenever a
**Media Item** has multiple playable **Media Sources** or **Source Variants**.

It must answer:

1. what is about to play;
2. why this source was selected;
3. whether another source can be chosen.

First-version fields:

- user-facing source or variant label, such as `1080p H.264 AAC` or
  `4K HDR HEVC`;
- Media Library name or source context, without local paths;
- container, video codec, audio codec, resolution, HDR, and bitrate when
  available;
- audio track and subtitle counts when available;
- playback-mode preview: Direct, Remux, HLS, or Transcode;
- warnings for likely transcode, device compatibility, remote source latency,
  or unavailable playback.

The picker must not show:

- filesystem paths;
- storage backend credentials or secret references;
- raw provider payloads;
- FFmpeg commands;
- server-local transcode output paths;
- complete internal diagnostic records.

## Search Baseline

Search first provides global title-oriented discovery and safe result
navigation.

First-version search should include:

- a globally reachable Search entry;
- keyword search over **Media Items**;
- basic result grouping by item kind or hierarchy, such as movie, series,
  episode, and unknown;
- empty, unreachable-server, and unauthorized states;
- result navigation to the matching detail page or a supported browse facet
  result when the result itself is a facet entry.

First-version search should not include:

- advanced filter panels;
- multi-condition filtering by year, rating, tag, person, resolution, codec,
  HDR, or source facts;
- sort-control UI;
- saved searches;
- search history;
- provider-specific search;
- cross-server search.

Advanced facets, filters, and sort controls must follow explicit **Public
Client API** support instead of inventing client-only filtering semantics.

## Player Baseline

The first Player is a reliability surface, not only an embedded `PlayerView`.

It must make playback state, buffering, seeking, exit behavior, cancellation,
and errors understandable before adding advanced media enhancements.

First-version player capabilities:

- play and pause;
- seek bar;
- loading and buffering states;
- elapsed and remaining time;
- full-screen and orientation handling;
- clear back behavior, with first version exiting playback and preserving
  progress when server support exists;
- Playback Error Sheet for network loss, auth failure, missing source,
  transcode failure, and unsupported media;
- Track / Subtitle Sheet entry point, even if the first implementation exposes
  only a limited track set;
- playback session cancellation on exit for HLS, remux, or transcode sessions
  when the public session route supports it.

Deferred player enhancements:

- brightness and volume gestures;
- chapter navigation;
- trickplay thumbnails;
- skip intro or outro;
- PiP;
- Cast or route selection;
- background video/audio behavior;
- playback speed;
- mpv or alternate player fallback;
- external player handoff;
- complex lock-screen controls.

## External Player Handoff

External player handoff is a deferred compatibility feature, not a
first-version playback path.

Taru should eventually support handoff to external Android players such as MX
Player, VLC, mpv-android, or Kodi-style consumers because self-hosted media
users may need codec, subtitle, device, or personal-player fallbacks.

The feature must preserve Taru's access boundary:

- it must be opt-in from Settings;
- it must not expose long-lived bearer tokens to external apps;
- it should prefer a short-lived external playback token or handoff URL;
- it may require a new Public Client API before implementation;
- it should clearly explain that progress sync, subtitle selection, track
  selection, and error diagnostics may be limited outside the built-in player;
- it should not be the default playback path for the first Android client.

## Offline / Downloads

Offline playback and Downloads are deferred from the first playback loop but
treated as a second-phase core client capability, not a casual later add-on.

They require a separate storage, permission, source-selection, and lifecycle
design before implementation.

The first Android client should not expose a Downloads tab by default. It may
reserve UI space for future download actions, but those actions should remain
hidden or disabled until the lifecycle is designed.

Future design must answer:

- whether downloaded media is the original direct source, a remux output, an
  HLS package, or an **Optimized Version**;
- whether remote/WebDAV sources download through Taru as a proxy or from a
  storage backend directly;
- how downloads bind to server identity, user identity, and Library Access;
- how subtitles, audio tracks, artwork, and playback progress are stored;
- how disk budget, expiration, deletion, resume, and corruption recovery work;
- how offline playback reports or later reconciles **User Playback State**.

## Settings Baseline

Settings first serves client identity, connection, theme, and basic playback
preferences. Server administration belongs outside the first mobile client.

First-version Settings should include:

- current server identity;
- switch server, re-authenticate, and sign out;
- connection diagnostics such as base URL, API version, and last public error;
- theme preference: system, dark, and light when implemented;
- basic playback preference:
  - Auto;
  - prefer Direct;
  - data-saving;
  - compatibility-first;
- basic subtitle preference:
  - default;
  - off;
  - preferred language when language selection exists;
- mobile-network warning or restriction;
- About, license, and app/server version information.

First-version Settings must not include:

- server administration;
- advanced transcode parameters;
- hardware acceleration selection;
- provider settings;
- NFO settings;
- Addon, Webhook, or Automation configuration;
- home-section customization;
- detailed gesture tuning;
- mpv or alternate player parameter editing;
- a large experimental flag list.

## Deferred Page Families

These are explicitly out of the first page set unless a later task narrows
their scope:

- Downloads / Offline
- Favorites
- Person Detail
- Collections
- Advanced Settings
- Server Admin
- Metadata Edit
- Addon, Webhook, and Automation management

## Navigation Baseline

The first phone/tablet shell should optimize for fast return to content and
playback:

- setup routes should disappear from the normal back stack after a successful
  connection;
- the main shell should expose a small number of stable destinations rather
  than mirroring every detail page as a tab;
- Home should bias toward continue/resume, recently added, and library entry
  points once the server API supports them;
- Libraries should remain the reliable structural browse fallback;
- Search should be globally reachable from the main shell;
- Settings should cover client preferences and server switching, not server
  administration.

## Phone And Tablet Layout

Phone and tablet share one touch-first navigation model. Tablet layouts may use
extra width for grids and detail context, but must not fork the product model.

First-version phone behavior:

- single-column navigation;
- main shell with stable top-level destinations;
- detail pages push full-screen;
- player is immersive and focused.

First-version tablet behavior:

- same route model as phone;
- wider grids with more columns;
- library/search list plus detail context where useful;
- item detail may split artwork, metadata, and actions across columns;
- no tablet-only feature behavior.

Android TV remains a separate later product surface because ten-foot UI and
remote-control focus navigation need a different interaction model.

## Playback Client Visual Baseline

Taru Android does not have a finished in-house design language yet. In this
workstream, `v0` refers to the frontend prototyping tool, not to the product
design system.

The nearest visual target is a Findroid-like Android media client: immersive,
artwork-led, dark-first, playback-confident, and emotionally engaging. Use
Material 3 as the interaction foundation, but lean expressive on browse,
detail, and player surfaces where it helps the media experience feel alive.

Use restrained chrome for setup and settings. Keep browse, detail, and player
surfaces image-led, gesture-friendly, and comfortable for repeat scanning.

Initial direction:

- expressive-leaning Compose Material 3 foundation;
- dark-first media viewing experience with a light theme later;
- image-led browsing with clear poster/backdrop hierarchy;
- dense enough for repeated browsing, but not admin-dashboard dense;
- tactile motion and transitions that support mood and focus, not decoration;
- local artwork-derived accent on browse, detail, source selection, and player
  surfaces when contrast and fallback behavior are explicit;
- predictable Android gestures and system back behavior;
- tablet layouts should use additional width for grids and detail context,
  not oversized decorative hero sections.
- clear typography for title, year, runtime, hierarchy, source labels, and
  playback warnings;
- action areas should make Play, Resume, Version, and More states easy to scan;
- empty and error states should be practical and specific, not marketing copy;
- avoid decorative gradients, glowing chrome, oversized marketing heroes, or
  visual effects that compete with artwork.

Core component semantics:

- Poster Card: identifies a Media Item through artwork, title, progress, and
  minimal secondary facts.
- Media Row: presents a scannable item in compact contexts such as search or
  source lists.
- Section Header: names a content rail or structural group without marketing
  copy.
- Source Chip: summarizes safe source facts such as resolution, codec, HDR, or
  playback mode.
- Play Action: primary route into playback, with Resume when authoritative
  state exists.
- Error Sheet: explains failure state, next action, and sanitized diagnostics.
- Track / Subtitle Sheet: presents player track choices without becoming a
  full technical diagnostics panel.
- Settings List: groups client preferences and identity controls without
  server administration.

Baseline tokens stay intentionally small:

- dark-first color roles;
- restrained accent color;
- poster and backdrop aspect ratios;
- compact and comfortable spacing scales;
- readable title/body/caption type roles;
- consistent corner radius and touch target rules.

This baseline explicitly does not define:

- final logo or brand identity;
- complete brand color system;
- illustration style;
- marketing screenshot art direction;
- complex choreography, global dynamic theme replacement, or alpha-only
  expressive API dependency;
- TV visual system.

## First-Version Non-Goals

- No custom visual brand system beyond a basic Taru theme.
- No TV ten-foot UI.
- No full offline/download manager.
- No cast route UI.
- No top-level people/collection-first exploration model. Detail-driven
  Browse Facet Result routes are allowed when backed by public facets.
- No admin console inside the mobile client.
