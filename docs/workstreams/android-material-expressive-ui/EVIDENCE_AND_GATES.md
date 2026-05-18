# Android Material Expressive UI — Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
git diff --check
```

## Gate Set

### Targeted Android Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

This proves Android DTO/client unit tests and UI-adjacent behavior still pass.

### Android Build Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon
```

This proves Compose, resources, manifest, Media3 dependencies, and debug
packaging still compile.

### Rust Workspace Gate

```powershell
cargo fmt --all -- --check
cargo nextest run --workspace --no-fail-fast
```

Run when server/public API/shared protocol files are touched, and before final
closeout.

### Diff Hygiene Gate

```powershell
git diff --check
```

This catches whitespace errors and unresolved patch artifacts.

### Review Gate

Use `review-workstream` before accepting each AME task and
`verify-rust-workstream` before closeout.

## Evidence Anchors

- `docs/workstreams/android-material-expressive-ui/DESIGN.md`
- `docs/workstreams/android-material-expressive-ui/TODO.md`
- `docs/workstreams/android-material-expressive-ui/MILESTONES.md`
- `docs/workstreams/android-material-expressive-ui/HANDOFF.md`
- `apps/android/app/src/main/java/dev/taru/android/ui/theme/`
- `apps/android/app/src/main/java/dev/taru/android/ui/components/`
- `apps/android/app/src/main/java/dev/taru/android/ui/shell/`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/`

## Evidence Log

- 2026-05-18: Workstream opened after merging `main` into
  `android-client-foundation`.
- 2026-05-18: `AME-020` completed with a new Material 3 theme/tokens layer,
  artwork-accent hook, shared UI surfaces, adaptive shell, browse-shell
  integration, and targeted JVM test coverage.
