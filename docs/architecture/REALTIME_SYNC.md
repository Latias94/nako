# Realtime And Sync Architecture

Last updated: 2026-05-29

This document maps realtime notifications, playback state synchronization, and
future offline sync/download-to-go work.

## Target Chain

```text
Domain Event / Runtime State Change
  -> durable event or ephemeral realtime update
  -> principal and library access filter
  -> SSE/WebSocket/Push/API polling surface
  -> client state update
```

Offline sync is a separate long-running artifact workflow:

```text
Sync request
  -> policy and quota check
  -> transcode/package job
  -> resumable artifact
  -> client download and expiry
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Domain event outbox | Shipped | `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md` | Bridge selected events to client realtime. |
| Webhook delivery | Shipped | webhook guide/workstreams | Keep external automation separate from client realtime. |
| Addon event delivery | Shipped foundation | addon event lanes | Reuse event vocabulary where safe. |
| Playback heartbeat persistence | Shipped foundation | user playback state lanes | Multi-device conflict policy. |
| Continue watching | Shipped foundation | user playback state lanes | Client realtime invalidation. |
| Admin runtime polling | Partial | admin playback diagnostics lanes | Realtime scan/transcode updates. |
| SSE/WebSocket client gateway | Not started | This document | Open `client-realtime-event-gateway`. |
| Offline sync/download-to-go | Not started | This document | Defer until playback artifact lifecycle is mature. |

## Next Work Lanes

### client-realtime-event-gateway

Goal: Push selected scan, playback, transcode, and catalog updates to clients
without exposing privileged internals.

Scope:

- SSE or WebSocket transport decision;
- principal-scoped event filtering;
- scan progress updates;
- playback session/transcode status updates;
- media-added/catalog invalidation updates;
- reconnect and backfill behavior.

Exit criteria:

- Media Web/Admin Web can replace selected polling paths;
- event payloads are redaction-safe;
- disconnected clients can recover state through normal REST reads.

### offline-sync-and-download-artifacts

Goal: Let a client request long-lived offline playback artifacts without
coupling them to transient HLS session output.

Scope:

- sync job identity;
- quota and expiry policy;
- transcode or remux artifact generation;
- resumable download API;
- revocation when user/library access changes.

This lane should wait until playback resource scheduling and artifact lifecycle
are stronger.

## Risk Register

### Durable Events And Realtime Events Are Different

Webhook/addon events need durable retry. Client UI updates often need
low-latency ephemeral delivery plus REST recovery. Do not force every realtime
state update through a durable outbox row.

### Realtime Payloads Need Access Filtering

Scan and playback events can leak library names, item titles, source IDs, or
session state. Filter by principal and library access before sending.

### Offline Sync Can Become A Second Transcode Runtime

Offline packages need quotas, expiry, resumable transfer, and policy checks.
They should reuse planning and resource scheduling but not reuse temporary HLS
session directories as durable downloads.

## Agent Notes

Do not add WebSocket/SSE payloads that expose raw paths, FFmpeg commands,
stderr, provider payloads, or bearer tokens. Prefer event IDs and safe summaries
with REST fetch for details.
