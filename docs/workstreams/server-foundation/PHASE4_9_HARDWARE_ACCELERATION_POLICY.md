# Phase 4.9: Hardware Acceleration Policy

Status: completed.

## Goal

Design the hardware acceleration policy for HLS/transcode execution before
enabling VAAPI, NVENC, or QuickSync in production command plans.

## Proposed Shape

- Defined a capability model for CPU-only, VAAPI, NVENC, and QuickSync.
- Added detection hooks that can report available accelerators without starting
  a transcode.
- Modeled encode policy separately from HTTP handlers and FFmpeg runner code.
- Defined CPU and GPU resource budgets with conservative defaults.
- Added fallback behavior when requested acceleration is unavailable.
- Kept command planning deterministic and testable without requiring real GPU
  hardware in CI.
- Default server policy is CPU-only with CPU fallback, one CPU transcode slot,
  and one GPU transcode slot.

## Non-Goals

- No multi-variant adaptive bitrate ladder yet.
- No vendor-specific tuning UI.
- No remote GPU worker or distributed scheduler.
- No local ML/GPU model scheduling.

## Validation

Coverage:

- capability records round-trip through domain/config code;
- unsupported accelerators fail with stable errors or fall back by policy;
- HLS command planning can select CPU-only vs declared hardware policy;
- tests do not require real VAAPI/NVENC/QSV hardware;
- resource-budget defaults are documented and bounded.
