# Web Bundle Budget And Product Pruning

Status: Queued
Last updated: 2026-05-28

## Why This Lane Exists

The Vite runtime is clean, but the copied v0 frontend still carries heavy
feature inventory. Some domains are not accepted Nako scope yet, and the first
Vite app chunk remains large enough to deserve a dedicated budget lane.

## Target State

- Bundle budgets are explicit and enforced by scripts or tests.
- Heavy optional domains are route-lazy, feature-flagged, or removed.
- Deferred domains such as AI, music, photos, podcasts, Live TV, and downloads
  do not affect first-run browser/Tauri startup.
- Bundle evidence is recorded after pruning.
- Remaining copied surfaces have product ownership status.

## In Scope

- Add bundle analysis/budget scripts.
- Split heavy route-only dependencies.
- Remove or quarantine deferred domains without backend ownership.
- Update feature gap ledger.

## Out Of Scope

- Backend feature implementation.
- Visual redesign.
- Native playback core.

## Closeout Condition

This lane can close when budgets are enforced, first-run chunks are reduced or
justified, deferred product surfaces are pruned/quarantined, and gates pass.
