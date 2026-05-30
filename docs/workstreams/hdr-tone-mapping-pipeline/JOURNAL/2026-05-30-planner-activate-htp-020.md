# Planner Activation - HTP-020

Date: 2026-05-30

The planner accepted `ACDN-020` and merged commit `b770cac9` into the HDR
branch with merge commit `372b8499`. This serializes the shared playback
vocabulary files before HDR implementation starts.

`HTP-020` is now ready as a playback-only task. It must not edit
`nako-transcode`, server HLS code, Public Client API DTOs, media probe schemas,
or web player code.