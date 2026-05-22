# Admin Playback Runtime Diagnostics Workstream

M56 adds a read-only Admin API v1 diagnostics surface for Nako's **Playback
Runtime**.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)

## Status

Completed.

## Purpose

M55 gave the admin web console a safe playback session list. The next
operational gap is the runtime context around those sessions: selected
hardware acceleration, FFmpeg capability evidence, transcode budgets, remote
stream/stage budgets, and staging cleanup configuration.

This workstream keeps diagnostics in the **Admin API** boundary and leaves the
**Public Client API** and `nako-client-protocol` unchanged.

## Related Workstreams

- [admin-playback-session-read-model](../admin-playback-session-read-model/README.md)
- [admin-web-console](../admin-web-console/README.md)
- [transcode-runtime](../transcode-runtime/README.md)
- [playback-streaming](../playback-streaming/README.md)
