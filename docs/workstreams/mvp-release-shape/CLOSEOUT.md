# MVP Release Shape Closeout

Status: Closed
Date: 2026-06-01

## Decision

Close `mvp-release-shape`. The workstream achieved its planning target: the
first self-hosted, video-first, single-admin MVP is defined, active queue risk
is routed, the P0 campaign slices are integrated, and the release-candidate
validation ladder has fresh evidence.

## Gates

- Gate 0 planner/docs preflight: passed.
- Gate 1 core release preflight: passed.
- Gate 2 server MVP journey: passed 341/341 with 169 skipped.
- Gate 3 Web/Public Client validation: passed 98/98 Web tests, TypeScript
  check, and bundle budget.
- Gate 4 playback runtime closeout: covered by the Gate 2 `playback`/`hls`
  filters plus the PTJCH post-merge seek gate.
- Gate 5 package and container shape: package `-WhatIf` passed; container gate
  passed 42/42 config tests and compose config checks.
- Gate 6 official addon alpha smoke: skipped by MVP scope because this
  candidate does not claim an official Addon Sidecar proof.
- Gate 7 PostgreSQL compatibility: passed 6/6 managed-artwork contract tests
  through a temporary local PostgreSQL harness.

## Follow-Ons

- Open an `operations-release` workstream only if the team wants one command
  for the full MVP ladder or actual artifact publication.
- Run `scripts/official-addon-e2e-smoke.ps1` only if a future candidate claims
  an official Addon Sidecar proof.
- Keep `PTJCH-310` as the playback artifact I/O follow-on unless a future
  release gate escalates it.
- Keep `GAMA-060` and `CSAPA-050` outside the MVP path unless the product cut
  changes.

## Residual Risk

- Gate 6 is intentionally skipped, not passed.
- Gate 5 used package `-WhatIf`; actual artifact publication still needs a
  dedicated release execution task.
- The release-gate wrapper remains optional; the documented ladder is the
  current source of truth.
