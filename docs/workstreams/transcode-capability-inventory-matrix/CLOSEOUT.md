# Transcode Capability Inventory Matrix - Closeout

Date: 2026-05-31
Status: Closed

## Result

`TCIM-010` through `TCIM-040` are complete. `nako-transcode` now exposes
optional capability inventory facts for bitstream filters, broader decoders,
encoders, filters, tone-map filters, and subtitle burn-in filters.

These facts are evidence-only. Listed and missing optional facts are visible in
`HardwareAccelerationReport`, but they do not change HLS pipeline selection,
FFmpeg command planning, server routes, Public/Admin DTOs, or release
packaging.

## Final Gates

```text
cargo nextest run -p nako-transcode hardware --no-fail-fast
cargo nextest run -p nako-transcode probe --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json
git diff --check
```

All final gates passed on 2026-05-31. `git diff --check` reported only Windows
line-ending normalization warnings.

## Follow-ons

- hardware tone-map execution and vendor filter chains;
- HEVC/AV1 output profile and FFmpeg command planning;
- subtitle burn-in execution behavior;
- Admin/Public capability reporting and release hardware matrices;
- HLS lifecycle/resource admission work.

## Residual Risks

The lane improves observability only. It does not prove real GPU smoke,
driver/container readiness, output codec selection, subtitle burn-in quality,
or hardware tone-map correctness.
