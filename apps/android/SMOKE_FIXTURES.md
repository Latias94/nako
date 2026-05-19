# Taru Android Smoke Fixtures

This document defines repeatable local state for Android smoke checks.

## State Modes

### current-state

Command:

```powershell
.\scripts\Smoke-Emulator.ps1
```

Use this for a quick launch check against whatever app state already exists on
the emulator. It is useful during development, but it is not a stable evidence
mode for route-specific screenshots because stored profiles, active server
selection, and previous navigation state can vary.

### empty-setup

Command:

```powershell
.\scripts\Smoke-Emulator.ps1 -ResetAppData
.\scripts\Smoke-Emulator.ps1 -FixtureState empty-setup
```

Use this when evidence must start from a deterministic setup state. The script
installs the debug APK, clears `dev.taru.android` app data with `pm clear`, then
launches `dev.taru.android/.MainActivity`.

Expected result:

- no stored Server Profile is present;
- the secure token vault is cleared with app data;
- the app starts at the connection/setup surface;
- generated evidence is written under
  `apps/android/build/smoke/<timestamp>-empty-setup-<serial>/`.

Captured surfaces:

- `setup.png`
- `setup.uiautomator.xml`
- `setup.criteria.txt`

### profile-missing-token

Command:

```powershell
.\scripts\Smoke-Emulator.ps1 -FixtureState profile-missing-token
```

Use this when evidence needs repeatable Home, Settings, and Server Profile
screens without depending on a live Taru server. The script installs the debug
APK, clears app data, seeds a local Server Profile named `Smoke Server`, and
does not seed an access token.

Expected result:

- exactly one local Server Profile is present;
- no access-token value is stored;
- Home opens in the safe `Authentication required` state;
- Settings and Server Profile are reachable from the shell;
- generated evidence is written under
  `apps/android/build/smoke/<timestamp>-profile-missing-token-<serial>/`.

Captured surfaces:

- `home.png`
- `settings.png`
- `server-profile.png`
- matching `*.uiautomator.xml` files
- matching `*.criteria.txt` pass/fail files

### profile-with-media

Command:

```powershell
.\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
```

Use this when evidence must prove the Android media path against real Public
Client API responses. The script installs the debug APK, clears app data,
prepares the `Night Harbor` demo fixture through
`scripts/Start-DemoFixtureServer.ps1`, starts a local `taru-server`, applies
`adb reverse`, then seeds one debug-only Server Profile plus an encrypted token
value through the app's real profile store and token vault. The same debug-only
seed also writes one device-local resume position for the selected `Night
Harbor` Media Item and Media Source.

Expected result:

- exactly one local Server Profile named `Smoke Server` is present;
- the token value is stored only in the Android token vault and is redacted from
  generated reports;
- the local resume position is stored only in Android device-local app storage;
- Home shows `Night Harbor` and visible Media Library facts from the server;
- detail, source picker, and player surfaces are reached through Public Client
  API route shapes;
- detail metadata proves API-backed Genre, Tag, and Person relationships by
  opening facet result routes that return `Night Harbor`;
- source picker and player evidence present resume as local-only state and do
  not claim cross-device **User Playback State** or Continue Watching;
- generated evidence is written under
  `apps/android/build/smoke/<timestamp>-profile-with-media-<serial>/`.

Captured surfaces:

- `home.png`
- `detail.png`
- `detail-metadata.png`
- `facet-genre.png`
- `facet-tag.png`
- `detail-cast-crew.png`
- `facet-person.png`
- `source-picker-local-resume.png`
- `source-picker.png`
- `player.png`
- matching `*.uiautomator.xml` files
- matching `*.criteria.txt` pass/fail files

## Safety Rules

- Do not commit generated screenshots or smoke reports by default.
- Do not put access-token values, token references, server-local paths, FFmpeg
  commands, or provider payloads into fixture files or reports.
- Do not fake server-backed User Playback State, Continue Watching, or
  unsupported browse facets as real client data.
- Use Public Client API responses or Android-local app state only.
- The `profile-with-media` profile seed entry point exists only in the debug
  APK. Release builds must not expose smoke fixture writers.

## Local Regression

Command:

```powershell
.\scripts\Smoke-Regression.ps1
```

Use this as the stable local confidence gate after Android UI, browse,
playback-launch, or smoke-harness changes. The wrapper builds the debug APK
once, runs the stable state set through `Smoke-Emulator.ps1`, and writes a
combined report under `apps/android/build/smoke-regression/<timestamp>/`.

Default states:

- `empty-setup`
- `profile-missing-token`
- `profile-with-media`

Useful variants:

```powershell
.\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token
.\scripts\Smoke-Regression.ps1 -SkipBuild
.\scripts\Smoke-Regression.ps1 -ContinueOnFailure
.\scripts\Smoke-Regression.ps1 -RetriesPerState 0
```

When a regression fails, open the report and then rerun the failed state
directly with `Smoke-Emulator.ps1 -FixtureState <state>` to collect focused
evidence. The wrapper retries each state once by default because ADB
`uiautomator dump` can temporarily return no root node while Android is
transitioning between launched activities. Failed reports include a category,
evidence path, log path, and focused rerun command for the failed state.

## Deferred Fixtures

These states need more work and should not be hand-waved into the smoke script:

- `profile-empty-library`: requires a public, token-safe server/profile fixture
  that can show Home and Settings without private data.
- `playback-ready`: the demo fixture currently prefers direct-play MP4 for a
  player-safe launch target. Full playback quality, HLS/remux, and session
  cancellation smoke remain deferred until they have explicit gates.
