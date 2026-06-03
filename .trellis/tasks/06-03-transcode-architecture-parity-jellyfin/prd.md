# brainstorm: transcode architecture parity with jellyfin

## Goal

Assess whether Nako's transcode area needs a fearless refactor to approach
Jellyfin-class capability, then implement the highest-leverage seam deepening
slice if the answer is yes.

## What I already know

* ADR 0038 and ADR 0049 already split playback planning, transcode policy,
  source-aware runtime, and FFmpeg command planning.
* Nako's transcode path is already split across `nako-playback`, `nako-server`,
  and `nako-transcode`.
* Jellyfin uses a much larger central helper (`EncodingHelper`) and a
  supervising transcode manager.
* The current Nako hotspot is `crates/nako-server/src/app/playback/hls.rs`,
  which still orchestrates admission, persistence, and execution in one module.
* The best current refactor target is the server-side HLS orchestration seam,
  not a wholesale rewrite of `nako-transcode`.

## Assumptions (temporary)

* "逼近 Jellyfin" means feature breadth and operational depth, not Jellyfin
  API compatibility or code-shape parity.
* This task starts as an architecture assessment, but it can and should turn
  into code if the assessment finds a load-bearing shallow seam.

## Open Questions

* Which Jellyfin-grade capability gap matters most next: device profile breadth,
  subtitle handling, audio compatibility, HDR, transcode job UX, or remote
  worker support?

## Requirements (evolving)

* Produce a yes/no recommendation on wholesale fearless refactor.
* Compare current Nako seams with Jellyfin patterns.
* Identify 2-3 concrete deepening opportunities.
* Separate already-deep modules from modules that are still shallow.
* Implement one high-leverage refactor slice that deletes shallow orchestration
  complexity without changing behavior.

## Acceptance Criteria (evolving)

* [x] Written recommendation with rationale.
* [x] File-level evidence from both Nako and Jellyfin.
* [x] Candidate list with files, problem, solution, and benefits.
* [x] Clear out-of-scope statement for code changes.
* [x] Refactor slice implemented with tests or updated coverage.
* [x] Review passes with no regression in playback/transcode behavior.

## Definition of Done

* Assessment is written and internally consistent.
* Evidence is persisted to task files.
* The selected refactor slice is implemented and verified.

## Out of Scope (explicit)

* Implementing a wholesale transcode rewrite.
* Copying Jellyfin code, schemas, tests, or assets.
* Adding new crates as part of this assessment.

## Technical Approach

Deepen the server-side HLS orchestration seam first:

* keep playback decision making in `nako-playback`;
* keep pipeline/policy/FFmpeg planning in `nako-transcode`;
* reduce `crates/nako-server/src/app/playback/hls.rs` to a thinner
  coordinator by extracting the repeated orchestration logic into smaller
  helpers/modules;
* preserve behavior with focused tests around admission, reuse, and execution
  planning.

## Decision (ADR-lite)

**Context**: Nako already has a deeper transcode split than Jellyfin's helper-
heavy shape, but server-side HLS orchestration still concentrates too much
coordination in one place.

**Decision**: Do a fearless refactor of the HLS orchestration seam, not a
wholesale rewrite of the transcode stack.

**Consequences**: Locality improves in `nako-server`; the transcode adapter
boundary stays strict; the refactor should remain behavior-preserving and keep
future media capability work easier to place.

## Technical Notes

* Nako files inspected:
  * `crates/nako-server/src/app/playback/hls.rs`
  * `crates/nako-server/src/app/playback/selection.rs`
  * `crates/nako-transcode/src/pipeline.rs`
  * `crates/nako-transcode/src/profile.rs`
  * `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `crates/nako-transcode/src/ffmpeg/remux.rs`
  * `crates/nako-playback/src/lib.rs`
* Jellyfin files inspected:
  * `repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs`
  * `repo-ref/jellyfin/Jellyfin.Api/Models/MediaInfoDtos/PlaybackInfoDto.cs`
  * `repo-ref/jellyfin/MediaBrowser.Controller/MediaEncoding/EncodingHelper.cs`
  * `repo-ref/jellyfin/MediaBrowser.MediaEncoding/Transcoding/TranscodeManager.cs`
* Persisted comparison notes live in
  `research/jellyfin-transcode-parity.md`.

## Verification

* `cargo check -p nako-server`
* `cargo check -p nako-server --tests`
* `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
* `cargo nextest run -p nako-server hls_source --no-fail-fast`
* `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
* `cargo fmt --all -- --check`
