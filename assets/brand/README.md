# Brand Assets

This directory contains source-level product branding assets shared across Nako
platforms.

## Nako

<img src="./nako-app-icon-1024.png" alt="Nako app icon" width="160">

Tagline:

> Your media home, gently kept.

One-line introduction:

> Nako is an open-source, self-hosted media home for gently organizing,
> keeping, and playing your films, shows, anime, and personal collection.

The Nako icon rationale and final generation prompt are recorded in
[nako-brand-identity](../../docs/workstreams/nako-brand-identity/README.md).

## Product Icon

- `nako-app-icon-1024.png` is the selected Nako source product/app icon.
- Platform-specific launcher icons should be generated from this source asset
  and stored in each platform's native resource tree.
- Android generated launcher resources currently live under
  `apps/android/app/src/main/res/mipmap-*` and are referenced from
  `apps/android/app/src/main/AndroidManifest.xml`.

Keep this directory for canonical brand assets only. Do not put generated build
outputs or temporary icon experiments here.
