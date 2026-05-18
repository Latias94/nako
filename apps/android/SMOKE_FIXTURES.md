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

## Safety Rules

- Do not commit generated screenshots or smoke reports by default.
- Do not put access-token values, token references, server-local paths, FFmpeg
  commands, or provider payloads into fixture files or reports.
- Do not fake server-backed User Playback State, Continue Watching, or
  unsupported browse facets as real client data.
- Use Public Client API responses or Android-local app state only.

## Deferred Fixtures

These states need more work and should not be hand-waved into the smoke script:

- `profile-empty-library`: requires a public, token-safe server/profile fixture
  that can show Home and Settings without private data.
- `profile-with-media`: requires server-backed demo Media Libraries, Media
  Items, detail responses, and playback decisions through the Public Client
  API.
- `playback-ready`: requires an explicit media source fixture and player-safe
  stream target; split this from Android-only smoke work if it needs server
  changes.
