# 0033: Version Addon Protocol Compatibility Separately From Addon and Crate Releases

## Status

Accepted.

## Context

Nako has three different version concerns around addons:

- the Nako server or workspace release, such as `0.1.0-alpha.1`;
- the Rust crate package versions for `nako-addon-protocol`,
  `nako-addon-client`, and SDK/helper crates;
- the runtime compatibility contract an Addon Sidecar declares in its
  manifest and echoes in request/response envelopes.

The first Addon Protocol slice used a date-shaped protocol version. That was
easy during implementation, but it makes compatibility unclear: addon authors
cannot tell whether a date means a server release, a protocol ABI, a schema
revision, or a historical marker.

Other extension ecosystems separate these concerns. Jellyfin plugin repository
entries carry a plugin `version` and a target server ABI such as `targetAbi`.
VS Code extensions declare their own SemVer `version` and an `engines.vscode`
compatibility range. Obsidian plugins declare their own SemVer `version` and a
minimum app version. Nako should borrow the separation of addon identity,
addon package version, and host/protocol compatibility, but not copy Jellyfin's
in-process binary ABI model because Nako addons are HTTP sidecars.

## Decision

Nako will manage Addon compatibility with three separate version concepts.

1. **Addon Version** is the addon author's implementation or package version.
   It is stored in manifest `version`, displayed to operators, and checked by
   Addon Health Check against the registered manifest snapshot.
2. **Addon Protocol Version** is the wire-compatibility version implemented by
   the Addon Sidecar. It is stored in manifest `protocol_version`, sent in
   Nako-to-addon request envelopes and protocol headers, and echoed in addon
   responses.
3. **Rust crate package version** is the Cargo/package release version of
   `nako-addon-protocol`, `nako-addon-client`, and related SDK crates. These
   crates may follow the Nako workspace release for packaging simplicity, but
   their package version is not the runtime compatibility gate.

The current Addon Protocol Version for `alpha.1` is `0.1.0-alpha.1`.

Nako will expose both:

- `ADDON_PROTOCOL_VERSION`: the protocol version Nako emits for new addon
  registrations, current examples, and reference addons;
- `SUPPORTED_ADDON_PROTOCOL_VERSIONS`: the exact protocol versions this Nako
  build accepts at registration, health-check, and resource-response
  validation time.

Old addons remain usable only when their manifest `protocol_version` is still
listed in `SUPPORTED_ADDON_PROTOCOL_VERSIONS` or when Nako later adds an
explicit adapter for that version. Unsupported protocol versions are rejected
with a protocol error and should surface as compatibility diagnostics in Admin
UI.

When Nako calls a registered Addon Sidecar, it must send the sidecar's
registered manifest `protocol_version`, not blindly send the latest protocol
version. The Addon Sidecar must echo that same protocol version in health and
resource responses. This keeps explicit older-version support possible without
turning every old addon call into an implicit latest-version negotiation.

During pre-`1.0`, Nako does not promise long-term compatibility across every
alpha protocol version. The supported list is the compatibility promise. After
the Addon Protocol reaches `1.0`, Nako should follow SemVer expectations:
compatible optional additions stay within the same major compatibility line,
while breaking manifest, envelope, resource, permission, or side-effect
semantics require a new incompatible protocol version.

Future Addon Manager catalog entries may add install-time compatibility
metadata such as `protocol_range`, `nako_server_range`, runtime image
requirements, signing facts, and platform constraints. That catalog metadata
is separate from the sidecar's manifest `protocol_version`. If an Addon
Sidecar later needs to support multiple protocol versions at once, Nako should
introduce an explicit negotiation or `supported_protocol_versions` contract in
a follow-up ADR.

## Consequences

- Addon authors can release addon updates without changing the Addon Protocol
  Version when the wire contract did not change.
- Nako can release server fixes or SDK crate updates without forcing every
  addon to update its manifest `protocol_version`.
- Runtime compatibility becomes testable through a small supported-version
  list instead of implicit workspace version matching.
- Date-shaped pre-release protocol strings are not carried forward into the
  `alpha.1` public contract.
- Nako may still break addon compatibility before `1.0`, but those breaks must
  be visible as explicit protocol version changes and supported-list updates.
- Addon Manager design can later choose package selection policy without
  changing the runtime Addon Protocol handshake.

## Alternatives Considered

- Use the Nako server version as the addon compatibility version: rejected
  because server releases can change deployment, Admin UI, database, or
  packaging behavior without changing the Addon Protocol.
- Use the Rust crate package version as the only compatibility gate: rejected
  because third-party Addon Sidecars may be written in TypeScript, Python, Go,
  or any HTTP-capable language and may not consume Rust crates at all.
- Keep date-shaped protocol versions: rejected because dates do not express
  SemVer compatibility expectations and are easy to confuse with release dates
  or event envelope dates.
- Copy Jellyfin's `targetAbi` model directly: rejected because Nako does not
  load in-process addon binaries and should version the HTTP wire contract
  rather than a native server ABI.
- Let manifests declare SemVer ranges immediately: deferred because the first
  Addon Protocol runtime only needs one concrete protocol version. Exact
  supported versions are simpler to validate and easier to explain during
  alpha.

## References

- Jellyfin plugin repository manifests distinguish plugin `version` from
  server ABI targeting with `targetAbi`:
  <https://jellyfin.org/posts/plugin-updates/>
- VS Code extension manifests distinguish extension `version` from
  `engines.vscode` compatibility:
  <https://code.visualstudio.com/api/references/extension-manifest>
- Obsidian plugin manifests distinguish plugin `version` from
  `minAppVersion` compatibility:
  <https://docs.obsidian.md/Reference/Manifest>

## Related Workstreams

- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
