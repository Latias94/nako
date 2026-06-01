# Douban Subject Kind Precision - Milestones

Status: Active
Last updated: 2026-06-02

## M0 - Lane Opening

Exit criteria:

- workstream state agrees across `TODO.md`, `TASKS.jsonl`, and
  `WORKSTREAM.json`;
- architecture maps route the active lane;
- non-goals exclude schema, Public Client API, Admin/Web, Generated Artifact
  apply, and hierarchy graph preview changes.

## M1 - Endpoint-Backed Capability Claims

Status: Complete after `DSKP-020`.

Exit criteria:

- Douban capabilities no longer claim Series, Season, or Episode support while
  the adapter uses only movie endpoints;
- unsupported search/fetch requests fail explicitly before provider HTTP;
- existing movie search/fetch and refresh behavior remains compatible.

## M2 - Closeout

Exit criteria:

- fresh evidence is recorded;
- any future Douban TV/episode endpoint follow-ons are split;
- architecture maps no longer route active work to a closed lane.
