# Taru Brand Assets

This directory contains source-level product branding assets shared across Taru
platforms.

## Product Icon

- `taru-app-icon-1024.png` is the current source product/app icon.
- Platform-specific launcher icons should be generated from this source asset
  and stored in each platform's native resource tree.
- Android generated launcher resources currently live under
  `apps/android/app/src/main/res/mipmap-*` and are referenced from
  `apps/android/app/src/main/AndroidManifest.xml`.

Keep this directory for canonical brand assets only. Do not put generated build
outputs or temporary icon experiments here.
