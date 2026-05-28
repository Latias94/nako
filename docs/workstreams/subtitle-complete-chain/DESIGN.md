# Subtitle Complete Chain

Status: Complete
Last updated: 2026-05-28

## Problem

Nako can register a read-only subtitle provider addon, but it does not yet have
a first-class subtitle chain that turns provider candidates into safe,
host-owned subtitle imports. Without a shared protocol contract and host-owned
import boundary, each provider would either duplicate wire types or pressure the
addon model toward unsafe direct media-library writes.

## Target State

- Subtitle search wire types live in `nako-addon-protocol`.
- Official subtitle provider sidecars use those shared protocol types.
- Nako host owns selection, import planning, Library File Write execution,
  refresh, and playback visibility.
- This first lane does not write subtitle files. It records the host boundary
  and leaves actual Library File Write execution for a later slice.

## Scope

- `docs/adr/0051-host-owned-subtitle-import-chain.md`
- `docs/workstreams/subtitle-complete-chain`
- `crates/nako-addon-protocol`
- `crates/nako-official-addon-catalog`
- `crates/nako-server` documentation and tests only where catalog/protocol facts
  need to stay stable
- `F:\SourceCodes\Rust\nako-official-addons\crates\nako-subtitle-provider`

## Non-Goals

- No subtitle sidecar file writes.
- No automatic subtitle import task.
- No OpenSubtitles, ASR, embedded subtitle extraction, or live provider call.
- No playback subtitle burn-in, HLS subtitle rendition, or client subtitle UI.
- No direct addon filesystem path, Source Locator, or remote storage handle
  authority.

## Architecture Direction

Treat subtitle as a chain, not a single plugin method:

1. Addon provider search returns typed, read-only candidates.
2. Nako records selected candidates through a host-owned reference model.
3. Nako derives import plans from media targets and policy.
4. Library File Write owns actual sidecar persistence.
5. Library refresh and playback planning consume Nako-owned subtitle facts.

The first slice should deepen the protocol boundary by moving the current
official provider's private subtitle request/response shapes into
`nako-addon-protocol`. The official provider should become an implementation of
that public contract.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Provider payload becomes a write authority. | High | Keep this lane read-only; record writes as host-owned follow-on. |
| Protocol drift between Nako and official addons. | Medium | Move shared types and schema constants into `nako-addon-protocol`. |
| Future write path leaks filesystem details. | High | Require Library File Write target derivation and redacted reports. |
| Playback expectations outpace import model. | Medium | Split playback subtitle execution into a later lane. |

## Validation Strategy

- `cargo nextest run -p nako-addon-protocol subtitle --no-fail-fast`
- `cargo check -p nako-addon-protocol --tests`
- `cargo nextest run -p nako-subtitle-provider --no-fail-fast`
- `cargo check -p nako-subtitle-provider --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
