# Transcode Interface And Runtime Plan Deepening

Status: Active
Last updated: 2026-05-31

## Why This Lane Exists

The playback/transcode stack has gained audio output requirements, HLS media
renditions, source-aware ladders, seek generations, and hardware pipeline
planning. The next HDR tone-mapping slice would add color pipeline requirements
to the same area. Today, `nako-server` still knows too much about the ordering
needed to turn a playback HLS decision into transcode execution:

- profile identity;
- track selection;
- source facts;
- audio output requirements;
- media rendition plans;
- adaptive request variants;
- artifact layout;
- low-level FFmpeg HLS request fields.

That makes the `nako-transcode` Interface shallow. Callers get too little
leverage for the amount of ordering knowledge they must carry.

## Target State

When this workstream closes:

- `nako-transcode` exposes a higher-leverage HLS runtime planning Interface;
- `nako-server` no longer reconstructs the transcode profile/request identity
  pipeline step-by-step in `hls_source_context`;
- low-level FFmpeg command request details are kept behind internal adapters;
- external callers enter through planned HLS/remux execution paths rather than
  raw `HlsRequest`, `RemuxRequest`, or `FfmpegCommandBuilder` construction;
- pure transcode tests cover the plan shape before server orchestration tests.

## In Scope

- transcode-owned HLS runtime planning values;
- migration of HLS source context assembly out of server orchestration;
- curated `nako-transcode` re-exports;
- hiding FFmpeg command request details behind execution adapters;
- focused transcode HLS/remux and server HLS regression gates.

## Out Of Scope

- HDR tone mapping implementation;
- subtitle burn-in;
- stage-aware hardware matrix expansion beyond what the moved Interface needs;
- HLS session admission, supersede, playlist readiness, segment wait, and
  cleanup consolidation;
- playback runtime resource admission unification;
- copying Jellyfin models or source code.

## Architecture Direction

Keep the existing crate direction: `nako-playback` owns playback decisions and
output requirements, `nako-transcode` owns transcode execution planning, and
`nako-server` remains a composition adapter. Do not make `nako-transcode`
depend directly on `nako-playback` unless a separate planner decision revisits
the crate graph.

The first implementation should deepen the Module around HLS runtime planning,
not merely rename helpers. The deletion test is: if the new Module were deleted,
the profile/request-variant/execution-policy ordering should reappear in one
place, not across multiple server call sites.

## Closeout Condition

This lane can close when:

- HLS source execution uses a transcode-owned runtime plan Interface;
- server playback no longer directly constructs low-level FFmpeg HLS requests;
- `nako-transcode` no longer broadly re-exports FFmpeg command details needed
  only by internal adapters;
- HDR `HTP-030` can start without adding more server-side transcode assembly.
