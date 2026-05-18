# Taru Android Client Interface Direction

Status: Proposed
Last updated: 2026-05-18

This document translates Taru's Android Client Foundation UX baseline into a
screen and component direction for a playback-first phone and tablet client.
`CONTEXT.md` remains authoritative for domain language. `UX_CONTEXT.md`
remains authoritative for first-version scope.

## Design Thesis

Taru Android should feel like a personal screening room connected to a
self-hosted library: calm when configuring, cinematic when browsing, precise
when choosing a source, and quiet once playback starts.

The client is not a mobile admin console. It is a repeat-use media client for
finding a playable **Media Item**, understanding the selected **Media Source**
or **Source Variant**, and starting playback with confidence.

## Scene Sentence

A user opens Taru at night on a phone or tablet, usually from a couch or bed,
with the room dim and the goal already clear: browse a familiar self-hosted
library, resume something, or play a known title without seeing server
internals.

This scene makes the first theme dark-first, artwork-led, and touch-comfortable.
Setup and Settings stay restrained because the user is doing connection work,
not watching media.

## Material 3 Expressive Interpretation

Use Material 3 as the interaction contract and lean expressive only where it
improves media feeling or playback clarity.

Current Android guidance to account for:

- Compose Material 3 is the foundation for Material Design 3 components,
  theming, dynamic color, and Android system visual cohesion.
- As of 2026-05-17, the stable Compose Material 3 release is 1.4.0 and the
  alpha train is 1.5.0-alpha19.
- Several expressive APIs have moved between experimental, stable, and alpha
  tracks. Treat expressive APIs as optional enhancement points unless the
  project deliberately accepts alpha API churn.
- Useful expressive concepts for Taru are responsive components, emphasized
  typography in high-emotion surfaces, short stateful transitions, local
  artwork-derived accents, and glanceable information.

Taru-specific rules:

- Artwork provides the emotion. Chrome must not compete with posters,
  backdrops, or video.
- Expressive motion should explain focus, selection, progress, or playback
  state. Avoid decorative choreography.
- Shape changes belong on interactive controls and selected states, not on
  every content surface.
- Local poster/backdrop-derived accents are allowed on detail, source
  selection, and player surfaces when reliable artwork exists and contrast can
  be preserved. They must fall back to Taru's cyan-green accent.
- Global dynamic theme replacement is deferred. Setup and Settings remain
  restrained and should not be recolored by media artwork.

## Information Architecture

### First App Shape

Phone top-level destinations:

- Home
- Libraries
- Search
- Settings

Tablet uses the same destinations. Wider layouts may show list plus detail
context, but must not add tablet-only product concepts.

### Route Families

- Setup: Server Connect, Token Login, Connection Result, Server Switcher.
- Browse: Home, Libraries, Library Detail, Search Results, Browse Facet
  Results.
- Content: Media Item Detail, Series Detail, Season Detail, Episode Detail.
- Playback: Player, Track / Subtitle Sheet, Source / Version Picker,
  Playback Error Sheet.
- Settings: active server, re-authentication, playback preferences, subtitle
  preferences, diagnostics, About.

### Navigation Pattern

Use a bottom navigation bar on phones for the four top-level destinations. Use
a navigation rail on tablets once the width is large enough to avoid cramped
content. Setup disappears from the normal back stack after successful
connection.

Details push over browse on phones. On tablets, Libraries and Search may use a
supporting pane: list or grid on the left, detail preview on the right. The
player is always immersive and takes over the screen.

## Visual System

### Color Strategy

Use a restrained dark base with one cyan-green brand accent. Browse and detail
may temporarily adopt artwork-derived muted accents, but the global UI should
still be recognizable as Taru.

Recommended base roles:

- App background: near-black blue-green, slightly warmer than pure black.
- Surface: raised blue-charcoal for setup, settings, sheets, and rows.
- Surface muted: low-contrast structural dividers and skeletons.
- Primary accent: cyan-green for Play, selected tabs, focused controls, and
  source-confirming states.
- Warning: amber for likely transcode, remote source latency, or mobile-data
  warnings.
- Error: warm red-orange for playback failure and auth failure.

Avoid:

- Decorative neon gradients.
- Full-screen abstract color fields without artwork.
- Purple-blue default SaaS palettes.
- White or pure black extremes.

### Typography

Use Android/system sans typography. Emotion comes from size, weight, and
artwork composition, not from a display font.

Suggested type roles:

- Display title: Media Item title on detail, 28-34sp, semibold.
- Section title: rail and group headings, 18-22sp, semibold.
- Item title: poster/row title, 14-16sp, medium.
- Metadata: year, runtime, hierarchy, source facts, 12-14sp.
- Player time: tabular or visually steady 12-14sp.

Keep browse titles to two lines and rows to one line where possible. Do not
scale type with viewport width.

### Shape And Elevation

Keep content cards restrained. Rounded cards should stay near the existing
8dp token. Use larger pill or circular shapes for controls, chips, and player
buttons where Material affordances expect them.

Surface depth should come mostly from color separation and artwork layering.
Use elevation sparingly in bottom sheets, navigation chrome, and the player
control tray.

### Imagery

Taru's primary visual asset is media artwork.

- Posters use 2:3.
- Backdrops use 16:9.
- Detail pages should prefer backdrops when available, then poster-led layout.
- Missing artwork should use a quiet generated placeholder based on item kind
  and title initials, not a loud illustration.
- Do not show provider logos, raw source paths, or server storage hints in
  artwork slots.

## Key Screens

### Setup Flow

Goal: make a technical access-token connection feel safe and finite.

Layout:

- Top: Taru wordmark or title, small server context copy.
- Main: one form surface with Display name, Server URL, Access Token.
- Actions: Test Connection as primary until successful, Save after success.
- Saved profiles: compact list below the form with active profile marker.

Visual tone:

- Restrained dark surface.
- No hero artwork.
- Use state-specific inline feedback, not modal-first errors.

Interaction details:

- Hide token by default.
- Show paste affordance when clipboard contains text.
- On success, animate the result row into a compact server profile.
- On failure, show one next action: edit URL, edit access token, retry, or copy
  sanitized diagnostics.

### Home

Goal: launch playback quickly even before personalization APIs exist.

Layout:

- Header: active server name, connection state, search icon, small server switch
  affordance.
- Primary rail: Continue Watching only when backed by authoritative
  **User Playback State**.
- Stable anchors: Media Libraries and Search always visible.
- Optional rails: Recently Added and Next Up only when supported by the Public
  Client API.

First-version empty-home behavior:

- Show Media Libraries as structural entry points.
- Show a compact search call to action.
- Do not fake recommendations or cross-device resume.

Expressive treatment:

- Use a single image-led featured resume card only when real resume data exists.
- Use horizontal poster rails with comfortable snap points.
- Use subtle poster scale on press and selection; no decorative entrance
  sequence.

### Libraries

Goal: reliable structural browsing when personalization is absent.

Layout:

- Library list/grid with library name, Media Domain or Library Preset, visible
  item count when available, and last updated when public API supports it.
- Library Detail opens into poster grid plus a compact filter surface only for
  public API backed facets.
- Tapping a supported facet opens Browse Facet Result instead of creating
  separate one-off pages for genres, tags, people, studios, or collections.

Visual treatment:

- Libraries can be text-forward because they are structure, not media posters.
- Library Detail becomes artwork-led once Media Items are visible.

### Search

Goal: fastest route to a known title.

Layout:

- Collapsed search bar in main shell.
- Expanded full-screen search on phone, docked search on tablet when practical.
- Results grouped by item kind or hierarchy: Movies, Series, Episodes,
  Unknown.

Constraints:

- No advanced filters until the Public Client API exposes explicit facets.
- No search history in first version unless designed as client-local state.

### Browse Facet Result

Goal: turn supported metadata relationships into reliable browsing without
creating separate pages for every relationship type.

Entry points:

- genre, tag, person, studio, and collection chips from Media Item Detail;
- public API backed facet controls from Library Detail;
- optional facet entries from Search when the search API exposes them.

Layout:

- Header: facet value, facet family, current library scope, and result count
  when available.
- Result body: the same poster grid or compact result rows used by Library
  Detail and Search.
- Scope control: current Media Library vs all accessible libraries only when
  the Public Client API supports both scopes.
- Sort control: use only explicit **Sort Keys** exposed by the Public Client
  API. Do not infer sort from local fields.

First-version facet families:

- Genre.
- Tag.
- Person, role-aware where available: Actor, Director, Writer.
- Studio.
- Franchise Collection or supported collection-like grouping.
- Year or Release Year.
- Media Item kind.

Design rule:

Browse Facet Result is not a biography page, collection editor, or advanced
query builder. It is a reusable result page for supported **Browse Facets**.

### Media Item Detail

Goal: decide what to play.

Phone layout:

- Top: image-led backdrop area with a poster anchor and readable scrim.
- Title block: title, year, runtime, content rating if available.
- Action cluster: Play or Resume as the dominant action, Version beside or
  below it, More only for secondary safe actions.
- Source summary: selected source label, playback mode, key warnings.
- Overview: collapsible after a readable first segment.
- Metadata chips: genre, tag, rating, language, hierarchy.
- Cast & Crew preview: compact actor, director, and writer entries when
  public data supports them.
- Relationship rows: franchise collection, series, seasons, episodes, extras,
  or parent series where relevant.

Tablet layout:

- Left: poster/backdrop and action cluster.
- Right: title, overview, source summary, metadata, hierarchy.
- Keep the same route and actions as phone.

Design rule:

The detail page should not become a MediaInfo dump. It shows only enough
**Canonical Metadata** and client-safe source facts to choose playback.
Metadata relationships are entry points into Browse Facet Result, not separate
first-version people, genre, tag, or collection products.

### Source / Version Picker

Goal: answer what will play, why, and whether another source is better.

Presentation:

- Use a modal bottom sheet on phone.
- Use a side sheet or supporting pane on tablet when opened from detail.

Rows:

- Primary label: `4K HDR HEVC`, `1080p H.264 AAC`, or user-facing
  **Source Variant Label**.
- Secondary: Media Library name or safe source context, never local path.
- Facts: container, video codec, audio codec, resolution, HDR, bitrate, track
  counts.
- Playback mode chip: Direct, Remux, HLS, or Transcode.
- Warning chip only when meaningful: likely transcode, unavailable source,
  remote latency, unsupported media.

Action:

- Selecting a row previews selection.
- Primary button confirms "Play this source" or "Use this version".
- If Taru selected a source automatically, show "Selected because..." using
  client-safe explanation text when the API provides it.

### Player

Goal: reliable playback first, cinematic chrome second.

Layout:

- Full-screen video/backdrop.
- Top overlay: back, title, source label, track/subtitle entry.
- Center overlay: play/pause and buffering state only.
- Bottom overlay: seek bar, elapsed/remaining time, selected source, subtitle
  state.
- Error sheet replaces vague failure toast.

Behavior:

- Controls fade but remain quickly recoverable with tap.
- Back exits playback in first version and reports/caches progress when
  supported.
- When a playback session requires cancellation, exit should make that state
  explicit but not expose internal route details.

Expressive treatment:

- Use shape-morphing icon buttons for play/pause and selected controls if the
  Material version supports it.
- Use an expressive but short buffering indicator.
- Use haptic feedback only for large state changes, such as seek commit or
  source switch, when implemented.

### Track / Subtitle Sheet

Goal: choose tracks without exposing a diagnostics panel.

Layout:

- Audio section: language, channel layout, selected marker.
- Subtitle section: off, embedded, external, forced/default when known.
- Explain limited states: "No subtitles available" or "Track data unavailable".

### Playback Error Sheet

Goal: make recovery obvious and diagnostics safe.

Structure:

- Plain-language title.
- One sentence explaining the state.
- Primary recovery action.
- Secondary action where useful.
- "Copy diagnostics" with sanitized public diagnostics only.

Examples:

- Network interrupted: Retry playback, Back to detail.
- Unauthorized: Re-authenticate server.
- No playable Media Source: Choose another item, Back to library.
- Transcode failed: Try another source, Copy diagnostics.

### Settings

Goal: client identity and playback preferences, not server administration.

Groups:

- Active server: name, URL, API version, last successful connection.
- Account access: re-authenticate, switch server, sign out.
- Playback: Auto, prefer Direct, data-saving, compatibility-first.
- Subtitles: default, off, preferred language when supported.
- Network: mobile data warning or restriction.
- Appearance: system, dark, light after light theme is implemented.
- About: app version, server API version, licenses.

Keep Settings visually calmer than Home and Detail.

## Component Semantics

### Poster Card

Purpose: identify a **Media Item** fast.

States:

- Default: poster, title, minimal metadata.
- Progress: bottom progress bar only when authoritative or explicitly local
  transient state is labeled as local.
- Pressed: slight scale and surface tint.
- Loading: skeleton in poster ratio, not spinner.
- Missing artwork: quiet title-initial placeholder.

### Media Row

Purpose: dense scan in Search, Source Picker, and hierarchy lists.

Fields:

- Leading artwork or source icon.
- Title or source label.
- One secondary line.
- Optional trailing chip for progress, playback mode, or warning.

### Source Chip

Purpose: show one safe source fact.

Kinds:

- Neutral: resolution, codec, audio, subtitles.
- Positive: Direct.
- Caution: Transcode, remote latency, unsupported.
- Selected: active source or version.

### Play Action

Purpose: primary route into playback.

Rules:

- Play when no resume state exists.
- Resume when authoritative **User Playback State** exists.
- Continue locally only if the state is clearly active-server-scoped and not
  represented as cross-device.

### Section Header

Purpose: name a rail or structural group.

Rules:

- No marketing copy.
- Optional count or "View all" only when actionable.

### Bottom Sheet

Purpose: source, track, subtitle, and error decisions.

Rules:

- Use sheets for scoped decisions.
- Avoid using sheets for ordinary page navigation.
- Keep actions reachable above navigation bars and gesture areas.

## Motion

Use short, stateful motion:

- Press feedback: 80-140 ms.
- Sheet entrance/exit: 180-240 ms.
- Poster to detail shared transition: 220-320 ms when artwork is loaded.
- Player overlay fade: 120-180 ms.
- Skeleton shimmer: optional and low-contrast; static skeleton is acceptable.

Avoid:

- Page-load choreography.
- Bouncy motion for serious errors.
- Layout-shifting text or buttons.
- Long rail animations that delay browsing.

## Accessibility And Ergonomics

- Minimum touch target: 48dp.
- Do not rely on color alone for source warnings or playback modes.
- Keep player controls reachable one-handed on phone.
- Respect system font scaling and test long titles.
- Keep TalkBack order aligned with visual order in detail, source picker, and
  player controls.
- Avoid tiny metadata chips as the only path to source selection.

## Empty, Loading, And Failure States

Loading:

- Use skeletons in the final layout shape for Home, Libraries, and Detail.
- Use a clear buffering state in Player.

Empty:

- Empty Media Library: explain no visible content and offer Back to libraries.
- No Search results: suggest changing the query.
- No playable source: explain that no playable **Media Source** is available
  and offer Back to detail or Choose another source when possible.

Failure:

- Keep user-facing copy actionable.
- Keep raw public diagnostics behind "Copy diagnostics".
- Never include token values, secret references, filesystem paths, FFmpeg
  commands, provider payloads, or server-local output paths.

## Implementation Slices

### Slice 1: Shell And Visual Tokens

- Keep current dark-first theme.
- Add bottom navigation and Search destination.
- Add skeleton components for browse loading.
- Refine Poster Card, Media Row, Source Chip, and Section Header.

### Slice 2: Home And Library Browsing

- Make Home useful with Libraries and Search even without resume/latest API.
- Move current browse list into Library Detail.
- Add phone poster grid and tablet grid/list-plus-detail variant.

### Slice 3: Detail As Playback Decision

- Replace read-only fact card with image-led detail surface.
- Add Play/Resume action cluster.
- Add selected source summary and Source / Version Picker entry.
- Keep internal diagnostics out of the page.

### Slice 4: Source / Version Picker

- Add bottom sheet rows for safe source facts.
- Add playback-mode chips and warnings.
- Confirm source before playback when multiple choices exist.

### Slice 5: Player Foundation

- Add Media3 playback surface.
- Add overlay controls, buffering, seek, track/subtitle sheet entry, and
  Playback Error Sheet.
- Wire exit progress reporting only to supported Public Client API behavior.

## Mockup Generation Prompts

Use these with the image generation workflow for visual exploration. Generated
mockups are references only; implementation should use Compose and Taru tokens.

### Prompt 1: Home And Browse

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design the Taru Android media client Home screen for a
self-hosted media server, playback-first, immersive, dark-first, artwork-led,
using Material 3 Expressive principles.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Home screen with active server header, Search entry, Media Libraries
anchor, optional Continue Watching rail with real artwork placeholders, poster
rails, bottom navigation for Home, Libraries, Search, Settings.
Style/medium: Native Android Compose Material 3 app UI, high-fidelity product
mockup, restrained chrome, cinematic media artwork.
Composition/framing: 9:19.5 portrait screen, edge-to-edge dark background,
clear hierarchy, comfortable touch targets.
Lighting/mood: Dim room media browsing, calm, personal, confident.
Color palette: Tinted near-black blue-green surfaces, cyan-green accent,
artwork-derived muted accents, amber only for warnings.
Text (verbatim): "Taru", "Home", "Libraries", "Search", "Settings",
"Continue Watching", "Media Libraries".
Constraints: Do not show admin dashboards, server jobs, provider diagnostics,
filesystem paths, token values, or fake recommendation claims.
Avoid: Decorative gradients, glassmorphism, neon glow, marketing hero section,
oversized empty cards, illegible poster text, watermark.

### Prompt 2: Media Item Detail

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design a Taru Media Item Detail screen as a playback decision
surface, not a metadata admin page.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Backdrop-led detail page with poster anchor, title, year, runtime,
overview, genre chips, Play or Resume primary action, Version action, selected
source summary, source facts chips, safe warning state.
Style/medium: Native Android Compose Material 3 app UI, expressive but
practical, cinematic media-client detail surface.
Composition/framing: Artwork top area with readable scrim, action cluster
below title, scannable metadata and source summary.
Lighting/mood: Immersive and emotionally engaging, playback-confident.
Color palette: Dark blue-green shell, cyan-green primary action, muted
artwork-derived accent, amber caution chip.
Text (verbatim): "Resume", "Version", "1080p H.264 AAC", "Direct", "Sources".
Constraints: Use Taru language: Media Item, Media Source, Source Variant,
Playback Source Selection. Show client-safe facts only.
Avoid: Full MediaInfo dump, server-local path, FFmpeg command, provider payload,
admin actions, decorative gradient overlay unrelated to artwork, watermark.

### Prompt 3: Source / Version Picker

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design a Taru Source / Version Picker bottom sheet for a Media
Item with multiple playable Media Sources or Source Variants.
Scene/backdrop: Phone screen with detail page dimmed behind a modal bottom
sheet.
Subject: Rows for "4K HDR HEVC", "1080p H.264 AAC", and "720p H.264", each
with resolution, codec, audio, bitrate, subtitle count, playback mode chips
like Direct, HLS, Transcode, plus one warning chip.
Style/medium: Native Android Compose Material 3 bottom sheet, precise and
glanceable.
Composition/framing: Bottom sheet occupies lower two thirds, selected row
clearly marked, primary action at bottom.
Lighting/mood: Confident technical choice without exposing internals.
Color palette: Dark raised surface, cyan selected state, amber warning, subtle
surface dividers.
Text (verbatim): "Choose version", "Selected by Taru", "Play this source",
"Direct", "HLS", "Transcode".
Constraints: Never show filesystem paths, storage credentials, secret
references, server-local transcode paths, raw diagnostics, or token values.
Avoid: Dense admin table, tiny unreadable chips, bright neon styling, watermark.

### Prompt 4: Player

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design the Taru Android player screen for reliable first
version playback using Material 3 Expressive principles.
Scene/backdrop: Full-screen video playback UI in landscape orientation, no
device frame.
Subject: Video surface with minimal overlays: back button, title, source label,
play/pause, buffering state, seek bar, elapsed and remaining time,
track/subtitle button, playback error sheet variant visible in one state.
Style/medium: Native Android Media3 player UI, cinematic, restrained,
touch-friendly.
Composition/framing: 16:9 landscape, controls placed for reachability, dark
scrims only where needed for readability.
Lighting/mood: Focused, immersive, reliable.
Color palette: Near-black translucent overlays, cyan-green active control,
amber caution for playback issue.
Text (verbatim): "Direct", "Subtitles", "Audio", "Retry playback",
"Back to detail".
Constraints: Player is a reliability surface. Make buffering, seeking, exit,
and errors understandable.
Avoid: Busy chrome, decorative gradients, admin diagnostics, FFmpeg command,
token values, watermark.

### Prompt 5: Detail Relationships

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design the Taru Android Media Item Detail lower section for
the first implementation baseline.
Scene/backdrop: Phone screen in portrait orientation, no device frame. The
screen is scrolled below the Play and Source Summary area.
Subject: Overview, metadata chips, compact Cast & Crew preview, Franchise
Collection row, series or related hierarchy row, Extras row, and a small More
from this library poster strip.
Style/medium: Native Android Compose Material 3 product UI, high-fidelity,
regular v2 baseline layout, dark cinematic media client.
Composition/framing: 9:19.5 portrait screen, regular vertical sections,
compact chips, tappable relationship rows, no heavy overlap.
Lighting/mood: Calm playback-adjacent browsing, precise and readable.
Color palette: Dark blue-green shell, restrained tonal surfaces, cyan-green
only for selected or navigational emphasis, amber only for warnings.
Text (verbatim): "Overview", "Metadata", "Cast & Crew",
"Franchise Collection", "Harbor Cycle", "Extras",
"More from this library".
Constraints: Relationships are navigation entry points into Browse Facet
Result, not editable metadata. Use fictional names only.
Avoid: Metadata editor controls, full MediaInfo dump, biography pages,
collection editor, filesystem paths, provider payloads, raw diagnostics,
FFmpeg commands, token values, real actors, real film names, watermark.

### Prompt 6: Browse Facet Result

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design the reusable Taru Android Browse Facet Result screen
for a supported metadata relationship, such as Director Mara Vale or Genre
Neo-noir.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Header with back button, facet value, facet family, library scope,
result count, lightweight grouping chips, sort chip, and poster grid results.
Style/medium: Native Android Compose Material 3 product UI, high-fidelity,
regular v2 baseline layout, reusable result screen.
Composition/framing: 9:19.5 portrait screen, compact top app bar, facet
header, simple chips, dense poster grid, optional compact result row.
Lighting/mood: Dark media library browsing, calm, task-focused, touch-friendly.
Color palette: Dark blue-green shell, restrained tonal surfaces, cyan-green
for selected chips and focus, no decorative neon.
Text (verbatim): "Mara Vale", "Director", "All accessible libraries",
"12 items", "From supported Browse Facets", "All", "Movies", "Series",
"Episodes", "Title", "Date added".
Constraints: Must be generic enough for Actor, Director, Genre, Tag, Studio,
Franchise Collection, Year, and Media Item kind. Use only public API backed
facets and sort keys.
Avoid: Actor biography page, filmography encyclopedia, collection editor,
advanced query builder, database column names, filesystem paths, provider
payloads, raw diagnostics, real media names, real actors, watermark.

### Prompt 7: Library Detail Facets

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design Taru Android Library Detail with lightweight facets
for the first implementation baseline.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Media Library header, in-library search field, lightweight facet chip
strip, one active filter chip, sort chip, poster grid, empty or loading hint,
and bottom navigation with Libraries selected.
Style/medium: Native Android Compose Material 3 product UI, high-fidelity,
regular v2 baseline layout, structured media browsing.
Composition/framing: 9:19.5 portrait screen, top library header, compact
search, horizontal chips, poster grid, bottom navigation.
Lighting/mood: Dim media browsing, structured and reliable, high density
without admin feel.
Color palette: Dark blue-green shell, restrained tonal surfaces, cyan-green
for selected chips and current destination, amber only for warnings.
Text (verbatim): "Movies", "Local Library", "328 items", "Search in Movies",
"Genre", "Tag", "Actor", "Director", "Year", "Collection", "Sort: Title",
"Genre: Mystery", "Clear filters".
Constraints: Use only public API backed facets. The page is a browse surface,
not Home and not server administration.
Avoid: Advanced multi-condition filters, admin job status, metadata refresh
controls, provider diagnostics, filesystem paths, token values, real film
names, trademarks, watermark, irregular hero layout.

### Prompt 8: Settings Home

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design Taru Android Settings Home for the first
implementation baseline.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Active server summary, account access, playback preferences, subtitle
preferences, network preference, appearance, diagnostics, About, and bottom
navigation with Settings selected.
Style/medium: Native Android Compose Material 3 product UI, high-fidelity,
calm settings surface, regular v2 baseline layout.
Composition/framing: 9:19.5 portrait screen, top app bar, grouped settings
sections, list rows with icons and chevrons, active server card.
Lighting/mood: Safe, calm, technical, less immersive than media browsing.
Color palette: Dark blue-green shell, restrained tonal surfaces, cyan-green
for active server and selected controls, amber only for warnings.
Text (verbatim): "Settings", "Home Server", "Connected", "API 0.1",
"Last connected 2 min ago", "Switch server", "Re-authenticate", "Sign out",
"Playback mode", "Auto", "Prefer Direct", "Data saving", "Subtitles",
"Mobile data warning", "Theme", "Diagnostics", "Copy diagnostics",
"Version", "Licenses".
Constraints: Hide access-token values. Diagnostics must be sanitized. Settings
is client identity and preferences, not server administration.
Avoid: Server jobs, metadata refresh controls, provider settings, addon or
webhook automation, advanced transcode parameters, experimental flags,
filesystem paths, token values, secret references, raw diagnostics, watermark.

### Prompt 9: Server Profile Settings

Use case: ui-mockup
Asset type: Android phone app high-fidelity UI concept
Primary request: Design Taru Android Server Profile and Connection Settings
for the first implementation baseline.
Scene/backdrop: Phone screen in portrait orientation, no device frame.
Subject: Server summary, base URL row, secure access-token card, connection
test section, sanitized diagnostics action, server profile list, and compact
danger zone.
Style/medium: Native Android Compose Material 3 product UI, high-fidelity,
secure connection settings, regular v2 baseline layout.
Composition/framing: 9:19.5 portrait screen, top app bar, summary card,
credential safety card, connection test card, profile list, danger zone.
Lighting/mood: Calm technical settings, safe, clear, and restrained.
Color palette: Dark blue-green shell, restrained tonal surfaces, cyan-green for
connected and primary actions, amber only for warning states.
Text (verbatim): "Server profile", "Home Server", "Connected",
"Public Client API 0.1", "Last successful connection 2 min ago",
"Base URL", "Server access token", "Stored securely on this device",
"Re-authenticate", "Replace access token", "Test connection",
"Copy diagnostics", "Travel Server", "Add server", "Sign out from this
server".
Constraints: Never show token text. Never show secret reference values,
filesystem paths, raw diagnostics, bearer-token strings, or server-local
paths.
Avoid: Server config editing, provider settings, NFO settings, addon or
webhook automation, local filesystem paths, FFmpeg commands, raw error dumps,
watermark.

## Initial Implementation Baseline

Generated mockups are visual references only. They are not implementation
specifications and should be translated into Compose components, tokens, and
responsive behavior before shipping.

Reference screenshots live with this workstream so implementation discussions
do not depend on temporary image generation output:

- `docs/workstreams/android-client-foundation/reference-screenshots/initial-reference-contact-sheet.jpg`
- `docs/workstreams/android-client-foundation/reference-screenshots/initial-home-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/initial-detail-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/initial-source-picker-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/facet-reference-contact-sheet.jpg`
- `docs/workstreams/android-client-foundation/reference-screenshots/facet-detail-relationships-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/facet-browse-result-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/facet-library-detail-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/settings-home-reference.png`
- `docs/workstreams/android-client-foundation/reference-screenshots/settings-server-profile-reference.png`

The first Compose pass should use these screenshots as the baseline direction.
They keep Home, Detail, and Source Picker recognizable as a native media
client, avoid exposing server internals, and use regular Compose-friendly
layout primitives. Their geometry maps directly to `Surface`, `Card`, `Button`,
`FilterChip`, `NavigationBar`, and `ModalBottomSheet`, which makes the design
easier to validate across phones, tablets, missing artwork, long labels, and
changing API data.

Carry forward:

- regular poster rails and section structure as the Home implementation base;
- Detail's simple image-led composition and stable action cluster placement;
- Source Picker's clear bottom-sheet structure and compact row rhythm;
- restrained dark-first tone, cyan-green primary action, and client-safe
  source facts;
- Material 3 Expressive only where it clarifies focus, selection, playback, or
  confidence.

Refine during implementation:

- make Home feel less like a generic streaming catalog through Taru-specific
  hierarchy, source confidence, and active-server context;
- keep Detail as a playback decision surface, not a dense metadata page;
- ensure the Source Picker remains a choice surface, not an admin table;
- keep media titles in native Android UI typography, not poster lettering;
- use cyan-green only for primary action, selected state, and playback
  confidence.

Defer until after the first version:

- irregular or asymmetric layout systems;
- heavy overlapping poster and control stacks;
- art-directed spacing that depends on a specific generated backdrop;
- complex shape morphing, global dynamic theme replacement, or required alpha
  Material APIs;
- complex transition choreography beyond state feedback.

Allowed in the first version:

- lightweight poster or row press feedback, such as subtle scale or tonal
  change;
- bottom navigation selected-state transitions;
- sheet enter/exit and selected-source state transitions;
- inline success/error reveal and loading state feedback;
- local artwork-derived muted accent on detail, source summary, and player
  overlays when contrast and fallback behavior are explicit.

### Facet Reference Notes

The facet reference screenshots are supplemental. They clarify how metadata
relationships should fit into the first implementation baseline without
changing the overall visual direction.

Carry forward:

- Detail lower sections may expose Cast & Crew, Franchise Collection, related
  hierarchy, extras, and library-related rows after the playback decision area.
- Browse Facet Result should use one reusable header and result grid for
  person, genre, tag, studio, collection, year, and item-kind facets.
- Library Detail may expose lightweight facet chips and one active filter chip
  when those facets are explicitly backed by the Public Client API.

Refine during implementation:

- keep Detail relationship rows compact, but do not make text as dense as the
  generated screenshot if labels become hard to scan;
- keep Browse Facet Result generic enough that it does not become a person
  biography page or collection editor;
- keep Library Detail filters lightweight. A selected chip plus sort control is
  acceptable; an advanced multi-condition builder is not part of the first
  version.

### Settings Reference Notes

The Settings reference is intentionally calmer than browse and detail screens.
It should validate client trust, connection state, and preference grouping
rather than create a cinematic surface.

Carry forward:

- grouped list sections for active server, account access, playback,
  subtitles, network, appearance, diagnostics, and About;
- an active server summary card with connection state, API version, and last
  successful connection time;
- explicit rows for switch server, re-authenticate, sign out, copy diagnostics,
  licenses, and app/server version information;
- a Server Profile detail screen with secure access-token management,
  connection testing, sanitized diagnostics, server profile switching, and a
  compact danger zone;
- no token value, secret reference value, filesystem path, raw diagnostics, or
  server-local path in the visible UI.

Refine during implementation:

- keep Settings below Home and Detail in visual intensity;
- make destructive actions such as sign out require confirmation;
- keep diagnostics client-safe and one action away from the main list;
- do not add server administration, provider settings, addon configuration, or
  advanced transcode controls to the first version.

## Implementation Tracking

ACF-030C translated the v2 baseline into Compose code on 2026-05-18. The
implementation now has a Material 3 bottom-navigation shell, split screen and
component files, API-backed Home and Libraries, Media Item Detail playback
decision skeleton, Settings Home, Server Profile, Search placeholder, and
Browse Facet placeholder.

The first implementation intentionally keeps Source / Version Picker behavior,
Search API integration, Browse Facet API integration, Library Detail, playback
decision construction, and Media3 playback as follow-on work.

## References

- `CONTEXT.md`
- `docs/workstreams/android-client-foundation/UX_CONTEXT.md`
- `docs/workstreams/android-client-foundation/DESIGN.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Android Developers, Compose Material 3 release notes:
  https://developer.android.com/jetpack/androidx/releases/compose-material3
- Google Keyword, Material 3 Expressive Android and Wear OS launch:
  https://blog.google/products-and-platforms/platforms/android/material-3-expressive-android-wearos-launch/
- Plex Support, Using the Library View:
  https://support.plex.tv/articles/200392126-using-the-library-view/
- Plex Support, Collections:
  https://support.plex.tv/articles/201273953-collections/
- Jellyfin, Local NFO metadata:
  https://jellyfin.org/docs/general/server/metadata/nfo/
