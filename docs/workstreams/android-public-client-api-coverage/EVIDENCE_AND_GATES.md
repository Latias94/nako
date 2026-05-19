# Android Public Client API Coverage Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Evidence Anchors

- Public route authority: `docs/api/HTTP_API.md`
- Android connection client: `apps/android/app/src/main/java/dev/taru/android/connection/TaruConnectionClient.kt`
- Android browse client: `apps/android/app/src/main/java/dev/taru/android/browse/TaruBrowseClient.kt`
- Android playback client: `apps/android/app/src/main/java/dev/taru/android/playback/TaruPlaybackClient.kt`
- Android DTO mirrors: `apps/android/app/src/main/java/dev/taru/android/browse/BrowseModels.kt`
- Public SDK route evidence: `crates/taru-client/src/lib.rs`, `sdk/typescript/src/index.ts`

## APIC-010 Evidence

Date: 2026-05-19

Commands and reads:

```powershell
git merge main
git status --short --branch
rg -n "^##|^###|GET /|POST /|PUT /|DELETE /|PATCH /|/health|/libraries|/items|/sources|/playback|/people|/tags|/genres|/admin|artwork|artifact" docs/api apps/android/app/src/main/java/dev/taru/android -g '*.md' -g '*.kt'
rg -n "fun .*\\(|suspend fun|/images|playback|libraries|items|people|tags|genres|search|source probe|probe" crates/taru-client sdk/typescript/src/index.ts crates/taru-api -g '*.rs' -g '*.ts'
```

Findings:

- `main` merged into `android-client-foundation` without conflicts.
- Android already consumes the core connection, browse, facet, playback
  decision, stream, and playback session routes through real HTTP clients.
- Public image byte routes `GET /images/{image_id}` and `HEAD /images/{image_id}`
  are present in server/API/SDK evidence but not yet consumed by Android.
- Android DTOs already decode image metadata from item detail, so selected
  artwork can be added without admin artwork routes.

Validation:

- APIC-010 is docs-only. Run `git diff --check` before commit.

## APIC-020 Evidence

Date: 2026-05-19

Implementation:

- Added Android `PublicImageRefDto` and `ImagesResponse` mirrors for the current
  Public Client API selected artwork contract.
- Added `TaruBrowseClient.itemImages` for `GET /items/{item_id}/images`.
- Added `PublicArtworkSource` and `PublicArtworkRequest` to build authenticated
  `/images/{image_id}` requests scoped to the active server profile.
- Added Coil 3.3.0 Compose/OkHttp dependencies and `TaruArtworkImage` for
  authenticated Compose image loading with fallback.
- Home and Libraries enrich the visible item page with best-effort public image
  refs. Detail renders selected backdrop/poster artwork from item detail image
  refs. Search/Facet keep deterministic fallback until their routes provide or
  enrich image refs.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
git diff --check
```

Result: PASS on 2026-05-19.

Notes:

- `HEAD /images/{image_id}` remains deferred because Coil handles normal image
  fetch/cache behavior for the first product slice.
- Android does not consume admin artwork routes.
- Public artwork request `toString` and safe previews redact bearer tokens.

## APIC-030 Evidence

Date: 2026-05-19

Implementation:

- Added shared Android artwork slot components for posters, backdrops, and the
  player preparing/error backdrop.
- Home, Libraries, Detail, and Player now use the same quiet deterministic
  missing-artwork presentation: title initial, normalized Media Item kind, and
  muted seed color. The UI does not invent fake posters or show provider/source
  storage hints.
- Player remains video-first. It uses a local title-seeded backdrop behind the
  Media3 surface for preparing/error states and disables Media3 embedded
  artwork display so playback launch requests do not need authenticated image
  tokens.
- Added presentation tests for fallback label normalization and player fallback
  title behavior.

Validation:

```powershell
apps\android\gradlew.bat -p apps\android --stop
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media
```

Result: PASS on 2026-05-19.

Smoke report:

- `apps/android/build/smoke-regression/20260519-131218/report.md`
- Surface evidence includes `home.png`, `detail.png`, and `player.png` under
  `apps/android/build/smoke-regression/20260519-131218/states/profile-with-media/20260519-131345-profile-with-media-emulator-5554`.

Notes:

- A first parallel validation attempt ran Gradle unit tests and smoke at the
  same time and hit a Kotlin incremental compilation cache race. The smoke
  command still completed with PASS. Gradle daemons were stopped and the unit
  test gate was rerun serially with PASS.
- Visual review was checked against
  `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`.

## Standard Gates

Docs-only changes:

```powershell
git diff --check
```

Android client code changes:

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
git diff --check
```

Android smoke changes, when emulator/server fixture impact is possible:

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media
git diff --check
```

## Gate Policy

- Do not mark an Android route covered unless production app code can call it
  against the active server profile.
- Do not mark preview/fake transport behavior as production coverage.
- Do not mark selected artwork complete until authenticated image loading works
  without exposing bearer tokens in UI, logs, or diagnostics.
- Do not claim cross-device Continue Watching until a public User Playback State
  contract exists and Android uses it.
