# Android Clipboard API Cleanup - Milestones

Status: Closed
Last updated: 2026-05-20

## M1 - API Confirmed

Status: Complete

Exit criteria:

- Local Compose clipboard API is confirmed from Gradle cache.

## M2 - Adapter Added

Status: Complete

Exit criteria:

- Internal plain-text clipboard adapter exists.

## M3 - Deprecated Calls Removed

Status: Complete

Exit criteria:

- Settings/player no longer import `LocalClipboardManager` or
  `AnnotatedString` for clipboard copy.

## M4 - Closeout

Status: Complete

Exit criteria:

- Focused/full Android debug unit tests pass.
- Diff hygiene passes.
