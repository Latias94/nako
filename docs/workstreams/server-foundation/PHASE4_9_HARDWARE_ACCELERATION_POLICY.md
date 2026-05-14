# Phase 4.9: Hardware Acceleration Policy

## Goal

Design the hardware acceleration policy for HLS/transcode execution before
enabling VAAPI, NVENC, or QuickSync in production command plans.

## Proposed Shape

- Define a capability model for CPU-only, VAAPI, NVENC, and QuickSync.
- Add detection hooks that can report available accelerators without starting a
  transcode.
- Model encode policy separately from HTTP handlers and FFmpeg runner code.
- Define CPU and GPU resource budgets, queue classes, and conservative defaults.
- Decide fallback behavior when requested acceleration is unavailable.
- Keep command planning deterministic and testable without requiring real GPU
  hardware in CI.

## Non-Goals

- No multi-variant adaptive bitrate ladder yet.
- No vendor-specific tuning UI.
- No remote GPU worker or distributed scheduler.
- No local ML/GPU model scheduling.

## Validation

Expected coverage:

- capability records round-trip through domain/config code;
- unsupported accelerators fail with stable errors or fall back by policy;
- HLS command planning can select CPU-only vs declared hardware policy;
- tests do not require real VAAPI/NVENC/QSV hardware;
- resource-budget defaults are documented and bounded.
