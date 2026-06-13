# nako-official-addon-catalog

Shared official addon catalog facts for Nako and official addon sidecars.

This crate contains descriptor and manifest builders for official addons whose
facts must not drift between the Nako catalog and the official addon runtime.

It also exposes `official_addon_catalog()` and
`render_official_addon_catalog_markdown()` for the durable operator artifact at
`docs/addons/OFFICIAL_ADDON_CATALOG.md`. The artifact is a catalog and install
reference surface only; it does not install, start, stop, update, remove, log,
or supervise Addon Sidecar processes.
