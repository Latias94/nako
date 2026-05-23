# Nako Licensing Policy

Status: Alpha licensing baseline

Nako uses two licensing bands. The server-side implementation remains
AGPL-3.0-or-later. Protocol, SDK, and example code intended for addon authors
or client authors is permissively licensed as Apache-2.0 OR MIT.

This split keeps the self-hosted server reciprocal while avoiding license
friction for third-party Addon Sidecars, clients, SDKs, and generated bindings.

## AGPL-3.0-or-later

Use AGPL-3.0-or-later for crates that implement Nako server behavior, internal
domain policy, persistence, storage, metadata mutation, playback, search,
automation, or operational surfaces.

Current AGPL crates:

- `nako-api`
- `nako-automation`
- `nako-catalog`
- `nako-core`
- `nako-db`
- `nako-events`
- `nako-library`
- `nako-media-probe`
- `nako-metadata`
- `nako-naming`
- `nako-nfo`
- `nako-search`
- `nako-server`
- `nako-streaming`
- `nako-transcode`
- `nako-vfs`

## Apache-2.0 OR MIT

Use Apache-2.0 OR MIT for crates that are intended to be embedded by addon
authors, client authors, generated SDK consumers, or example sidecar
implementations.

Current permissive crates:

- `nako-addon-client`
- `nako-addon-protocol`
- `nako-client`
- `nako-client-cli`
- `nako-client-core`
- `nako-client-protocol`
- `nako-client-uniffi`
- `nako-reference-addon`
- `nako-uniffi-bindgen`

Generated client SDK outputs should use Apache-2.0 OR MIT unless the generated
file or target repository declares a different license.

## Release Artifacts

Release packages should include:

- `LICENSE` for AGPL-3.0-or-later;
- `LICENSE-APACHE-2.0` for Apache-2.0;
- `LICENSE-MIT` for MIT;
- `docs/LICENSING.md` for the crate-level policy.

## Future Changes

If a crate becomes part of the server runtime, moves behind server persistence,
or starts enforcing canonical metadata, storage, playback, automation, or
security policy, default it to AGPL-3.0-or-later.

If a crate exists primarily to let third parties integrate with Nako without
linking server internals, default it to Apache-2.0 OR MIT.
