# Transcode Output Shape, HLS Manifest, And Ladder Runtime

Status: Closed
Last updated: 2026-05-28

Durable fearless refactor lane for deleting Nako's transitional transcode
output-shape assumptions before implementing adaptive HLS ladder runtime.

The lane has three ordered slices:

- replace `output_container: String` plus optional HLS output with a typed
  output-shape model that cannot express remux/HLS invalid combinations;
- introduce explicit HLS artifact manifest/runtime output records instead of
  deriving every artifact from a single playlist path and parent directory;
- implement the first executable adaptive HLS ladder slice on top of that
  manifest boundary.

MPEG-TS and fMP4 single-variant behavior remain covered while adaptive HLS is
added as a separate executable shape.
