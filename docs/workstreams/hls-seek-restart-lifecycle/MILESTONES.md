# HLS Seek Restart Lifecycle - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Lane Opened

The workstream is indexed and has clear non-goals, gates, and task boundaries.

## M1 - Generation Identity

Default HLS playback keeps existing request identity. Non-zero seek generation
changes request identity and staging layout without requiring a public API.

## M2 - Runtime Restart Semantics

The HLS runtime distinguishes same-generation reuse from superseding-generation
restart and has explicit cancellation behavior.

## M3 - Seek Command Planning

FFmpeg HLS command planning receives the selected start position and has tests
for seek flags, timestamp behavior, and segment output.

## M4 - Product Surface

The public/internal seek request surface is decided, verified, and documented.

