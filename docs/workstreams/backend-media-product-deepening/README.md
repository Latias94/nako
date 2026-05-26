# Backend Media Product Deepening

Status: Active
Last updated: 2026-05-27

This workstream owns the backend refactor that makes Nako's local-media product
surface coherent across Admin Web, Media Web, desktop shells, and native mobile
clients.

The lane is intentionally backend-first. Frontend implementation is out of
scope while the UI is being redesigned, but the backend contracts must be good
enough for a mature media client to consume later.

## Goals

- Flatten the current no-users database migration history into a clean baseline
  instead of a historical replay script.
- Extend local identity beyond admin-provisioned passwords with controlled
  invitation-based registration.
- Deepen the **Playback Runtime** so a **Playback Session** is distinct from a
  temporary **Playback Transcode** artifact.
- Add permission-gated Management Context Links so media browsing can discover
  safe admin actions such as scan, metadata refresh, playback diagnostics,
  jobs, and runtime settings.

## Non-Goals

- Open public self-registration by default.
- Build or redesign Admin Web or Media Web UI.
- Replace FFmpeg, mpv/libmpv, GStreamer, WebCodecs, or platform-native player
  engines with a Nako-owned decoder.
- Copy code, schemas, assets, comments, or tests from reference repositories.
- Move Admin API DTOs into `nako-client-protocol`.
- Add recommendation systems before local browse/playback/admin workflows are
  stable.

## Reference Research

Reference repositories are used for product and architecture shape only:

- `repo-ref/jellyfin` and prior Plex research: mature admin/media switching and
  operator workflows.
- `repo-ref/kyoo`: modern self-hosted video server with a separate transcoder
  surface and running-stream diagnostics.
- `repo-ref/dim`: Rust media server reference for virtual manifests, stream
  tracking, and FFmpeg-backed session orchestration.
- `repo-ref/oximedia`: broad Rust media framework reference for pipeline and
  hardware acceleration layering, not a dependency target.
- `repo-ref/libmedia`: web-client capability reference for WebCodecs, WASM
  codec fallback, MSE, worker/threading, and soft/hard decode expression.

The selection conclusion is conservative: use proven media engines for media
heavy lifting and keep Nako's backend deep modules focused on authority,
session state, source selection, tickets, jobs, and redacted read models.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
