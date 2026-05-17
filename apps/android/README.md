# Taru Android

This is the Android client scaffold for the Android Client Foundation
workstream. It is intentionally kept outside the Rust Cargo workspace.

## Scope

Current slice: `ACF-020 Public Client Connection And Auth Slice`.

Included:

- single-module Android app under `apps/android`;
- Kotlin, Compose, and Material 3;
- dark-first local debug shell aligned with Design Language v0;
- local token, spacing, type, shape, poster, backdrop, and touch-target roles;
- Gradle Wrapper for local builds;
- server URL plus access-token setup shell;
- `GET /health` preflight and lightweight authenticated Public Client API
  probe;
- API-version and public error-envelope parsing;
- multiple server profiles with one active profile;
- Android secure token vault with profile records storing token references only.

Not included in this slice:

- Media3 playback;
- UniFFI or shared Rust mobile core;
- downloads or external player handoff.

## Prerequisites

- JDK 21.
- Android SDK with platform `android-36`.
- Android build tools available from the configured SDK.

The Gradle project uses its own wrapper. Do not add `apps/android` to the Rust
Cargo workspace.

## Commands

From `apps/android`:

```powershell
.\gradlew.bat :app:assembleDebug
.\gradlew.bat :app:testDebugUnitTest
```

From the repository root:

```powershell
cargo check --workspace --tests
git diff --check
```
