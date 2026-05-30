# HDR Tone Mapping Pipeline - Evidence And Gates

Status: Draft
Last updated: 2026-05-30

## HTP-010 Gates

```text
python -m json.tool docs/workstreams/hdr-tone-mapping-pipeline/WORKSTREAM.json
git diff --check -- docs/workstreams/hdr-tone-mapping-pipeline docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md
```

`HTP-010` is docs/research-only. Do not run or modify Rust implementation code
for this task.

## Future Gates To Confirm

These are placeholders until `HTP-010` confirms the executable seam:

```text
cargo nextest run -p nako-playback hdr --no-fail-fast
cargo nextest run -p nako-transcode hdr --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Ledger

No implementation evidence yet. `HTP-010` should record the research result and
either activate `HTP-020` or keep the lane draft with explicit blockers.

## Residual Risks

- HDR color facts may not be complete enough in current media probe output.
- Hardware tone mapping differs across VAAPI, QSV, NVENC, AMF, and CPU paths.
- Tone mapping, audio downmix, and HLS runtime all touch playback/transcode
  seams, so implementation must be serialized unless scopes are narrowed.
