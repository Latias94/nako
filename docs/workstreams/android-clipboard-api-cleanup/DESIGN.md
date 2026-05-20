# Android Clipboard API Cleanup

Status: Closed
Last updated: 2026-05-20

## Problem

Settings and player diagnostics copy actions call deprecated
`LocalClipboardManager` directly and build `AnnotatedString` at each call site.
This creates upgrade noise and spreads clipboard API details through UI routes.

## Target State

- Use Compose UI 1.11.1 `LocalClipboard` and `Clipboard.setClipEntry`.
- Add a small internal clipboard adapter for plain text copy actions.
- Replace settings/player direct clipboard calls with the adapter.
- Keep visible UI behavior unchanged.

## Scope

- New UI clipboard adapter under `apps/android/app/src/main/java/dev/taru/android/ui/`.
- Settings/player diagnostics copy call sites.
- Focused unit/build verification.
- Workstream docs under this directory.

## Non-Goals

- Do not add toast/snackbar feedback in this lane.
- Do not change diagnostics content.
- Do not introduce app-wide clipboard history or telemetry.

## Local API Evidence

Local Compose artifact inspected:

- `androidx.compose.ui:ui-android:1.11.1`
- `androidx.compose.ui.platform.LocalClipboard`
- `androidx.compose.ui.platform.Clipboard.setClipEntry`
- `androidx.compose.ui.platform.ClipEntry`

Plain text is represented with Android `ClipData.newPlainText`.

## Closeout Notes

The lane closed with `rememberTaruClipboard()` as the only app-level clipboard
entrypoint. Settings and player diagnostics copy actions now call
`TaruClipboard.copyPlainText`.

No call sites import `LocalClipboardManager` or construct `AnnotatedString` for
clipboard copying.
