# Android Fearless Client Refactor — Problem And Decision Register

Status: Complete
Last updated: 2026-05-21

This register captures the architecture review findings that motivated the
lane. It is the canonical issue inventory for this workstream.

## Priority Legend

- P0: security or architecture boundary that should be fixed before more
  feature expansion.
- P1: high-leverage refactor that reduces duplicated future work.
- P2: product scalability or production-readiness hardening.
- P3: product polish, a11y, localization, or UX refinement.

## Register

| ID | Priority | Area | Finding | Decision | Owning task |
| --- | --- | --- | --- | --- | --- |
| AFCR-R001 | P0 | Playback security | `PlaybackLaunchRequest` and `TaruRoute.Player` can carry a `TaruHttpRequest` containing raw bearer headers. | Done 2026-05-20: route-level playback state now stores `PlaybackRequestDescriptor`; Authorization is injected only by explicit final-request builders and the Media3 runtime boundary. | AFCR-020 |
| AFCR-R002 | P0 | Diagnostics safety | Current safety relies partly on safe previews and custom `toString` overrides. | Done 2026-05-20: descriptors reject Authorization/bearer values, safe previews are computed from token-free state, and route/player tests cover save payload and diagnostics redaction. | AFCR-020 |
| AFCR-R003 | P1 | Public Client API | Connection, browse, playback, and User Playback State clients repeat request execution, version checks, error parsing, JSON decode, URL helpers, and redaction. | Done 2026-05-20: introduced `PublicClientApiExecutor`; route clients are now thin route/category modules. | AFCR-010 |
| AFCR-R004 | P1 | Browse state | `BrowseSession` owns too many route families and async request counters. | Done 2026-05-21: `BrowseSession` is now a composition shell over navigation, route state policy, route loading, search, detail/source/playback selection, and playback start modules. | AFCR-030 |
| AFCR-R005 | P1 | Test locality | Tests are strong but mirror the current broad modules; route-specific policy remains coupled. | Done 2026-05-21 for browse state: added module-level route-state policy regressions while preserving existing integration-style browse tests. | AFCR-010, AFCR-030 |
| AFCR-R006 | P2 | Network transport | `HttpURLConnection` transport is minimal and lacks richer cancellation, connection pooling, retry hooks, and final cleanup hardening. | Done 2026-05-21 for current foundation: kept `TaruHttpTransport` seam, added explicit security policy and final cleanup guard around `HttpURLConnection`; richer pooling/retry remains a future production transport decision. | AFCR-040 |
| AFCR-R007 | P2 | Network security | Manifest globally allows cleartext traffic. | Done 2026-05-21: main manifest no longer globally permits cleartext; debug manifest and debug `BuildConfig` opt in for local development; release defaults reject HTTP before token/transport; connection flow reports insecure HTTP as token-safe diagnostics. | AFCR-040 |
| AFCR-R008 | P2 | Large libraries | Home, Search, Library Detail, relationship indexes, and facets use fixed first-page loads. | Done 2026-05-21 for the first scalable vertical: Search, relationship indexes, and public-backed facets now share server-backed next-page semantics and visible Load more actions; Home/Library Detail remain a follow-on extension after this shared policy is proven. | AFCR-050 |
| AFCR-R009 | P2 | Settings/runtime | Playback preferences and local position stores are profile-scoped, but broader client settings are still first-version placeholders. | Done 2026-05-21 for visible settings copy: Settings now distinguishes real profile/sign-in state from placeholder playback preferences and keeps sanitized diagnostics copy explicit. | AFCR-060 |
| AFCR-R010 | P3 | UI copy | Several strings expose implementation language such as API gaps, route checks, token references, and User Playback State internals. | Done 2026-05-21: Settings, connection, browse, detail, source picker, player, and safe client error copy now use media-client language such as server compatibility, versions, titles, watch progress, and sign-in keys while preserving advanced sanitized diagnostics. | AFCR-060 |
| AFCR-R011 | P3 | Localization | UI text is hard-coded in Composables. | Done 2026-05-21 for stable common actions and high-reuse labels: added Android string resources plus `TaruStrings` indirection for Back, Retry, Change server, Search, Load more, Copy diagnostics, access-key label, and player session semantics. | AFCR-060 |
| AFCR-R012 | P3 | Accessibility | Custom rows, chips, artwork, source picker, and player overlays need stronger semantics and state descriptions. | Done 2026-05-21 for key paths: custom settings rows, sign-out, active profile card, status chips/pills, pressable media/library cards, relationship rows, source picker radio rows, library source rows, and player session status now expose roles or TalkBack-friendly labels where practical. | AFCR-060 |
| AFCR-R013 | P2 | Artwork/network auth | Artwork requests currently build authenticated image requests directly from UI resolver. | Split 2026-05-21: AFCR-020/AFCR-040 made playback and transport token-safe, but artwork needs its own descriptor module only when image caching, offline, or shared client-core work begins. Keep it out of this lane to avoid a shallow single-adapter seam. | Follow-on |
| AFCR-R014 | P1 | Future SDK/FFI | Direct Kotlin DTO mirrors are now large enough to revisit shared client core timing, but replacing everything now may slow the refactor. | Decided 2026-05-21: do not generate or FFI-replace inside this lane. Keep the Kotlin `PublicClientApiExecutor` seam for closeout; split generated Kotlin SDK after OpenAPI v1 stabilizes, and split shared Rust/UniFFI client core after a target-state document defines the narrow portable interface. | AFCR-070 |
| AFCR-R015 | P2 | Gradle module shape | Single `:app` is still workable, but package seams must become cleaner before feature growth. | Decided 2026-05-21: package seams are deep enough for this foundation, but Gradle modules would be premature without a second adapter, generated SDK, or measurable build/dependency pressure. Keep one app module for closeout and split later with an explicit dependency graph. | AFCR-070 |

## Cross-Cutting Decisions

1. No historical Android implementation detail is protected. Code may be
   deleted or replaced when a deeper module makes it obsolete.
2. Public Client API remains the only server data boundary for Android.
3. Android owns Media3 playback; Rust and server code do not become a player
   abstraction.
4. Debuggability must remain token-safe and locator-safe.
5. Product UI should be media-client language first, protocol language second.
