# HRLB-030 Follow-On Split - 2026-05-31

## Scope

Task: `HRLB-030`

Goal: decide whether PAIP artifact I/O pressure, resource admission
unification, remote workers, LL-HLS/CMAF, player UX, and HLS test stability
remain follow-ons or become the next bounded workstream.

No Rust behavior was changed.

## Decisions

| Area | Decision | Proposed lane |
| --- | --- | --- |
| HLS test stability | Make this the next bounded playback-transcode workstream after HRLB closeout. | `proposed:hls-progressive-readiness-test-stability` |
| Artifact I/O pressure | Keep as a PAIP playback/storage follow-on. | `proposed:hls-artifact-io-pressure-enforcement` |
| Resource admission unification | Keep separate from HLS lifecycle and PAIP. | `proposed:playback-admission-queueing-and-waitlist` |
| Remote workers | Keep as later runtime/control-plane work. | `proposed:remote-transcode-worker-runtime` |
| LL-HLS/CMAF | Keep as later protocol/runtime work. | `proposed:ll-hls-cmaf-runtime` |
| Player UX | Keep in client/player product lanes. | `proposed:player-hls-session-controls-and-recovery` |

## Rationale

HRLB-020 passed its final HLS gate, but a prior full-suite HLS run exposed a
load-sensitive progressive-readiness timeout. Gate stability should be hardened
before PAIP because PAIP will add disk read/write pressure and more concurrent
segment behavior.

PAIP should start with storage/VFS coordination because enforcing
`HlsArtifactIo` crosses playback resource demand, storage backend health,
segment read/write pressure, and redaction-safe diagnostics.

## Handoff

Status: DONE_WITH_CONCERNS

Next: run `HRLB-040` closeout. Preserve these split decisions, verify final
gates, then open `hls-progressive-readiness-test-stability` as the recommended
next bounded workstream.
