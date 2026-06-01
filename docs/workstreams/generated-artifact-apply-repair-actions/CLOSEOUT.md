# Generated Artifact Apply Repair Actions — Closeout

Status: Closed
Closed: 2026-06-02
Task: GAARA-050

## Final Status

Closed after `GAARA-020` and `GAARA-050`.

The lane proved that the current recovery repair path should stay
preparation-first: recovery rows route operators to the current Metadata
Authority apply plan, and live mutation continues through the existing
single-artifact apply endpoint with a fresh Web-generated idempotency key.

No backend recovery mutation wrapper is shipped or selected for the current
product shape.

## Shipped

- A seam decision for Generated Artifact apply repair actions.
- A Web route-state contract proving recovery-row navigation into the current
  apply plan.
- Guard evidence that no mutation occurs from the recovery queue before
  explicit apply confirmation.
- Guard evidence that confirmation uses a new Web idempotency key instead of
  recovery-row idempotency data.
- Workstream state that defers one-click wrapper and UX polish work to
  separate, explicit follow-ons.

## Gates

Fresh verification for the shipped decision:

```bash
cargo nextest run -p nako-server generated_artifact_metadata_apply_replays_same_idempotency_key_from_durable_outcome generated_artifact_metadata_apply_rejects_stale_target_before_mutation --no-fail-fast
npm --prefix web run test -- src/test/route-state-contracts.test.tsx
npm --prefix web run check
python -m json.tool docs/workstreams/generated-artifact-apply-repair-actions/WORKSTREAM.json
JSONL validation for TASKS.jsonl, CAMPAIGNS.jsonl, and CONTEXT.jsonl
git diff --check
```

Results:

- server apply idempotency replay and stale-target rejection tests passed;
- Web route-state contracts passed, including the new recovery-to-current-plan
  guard;
- Web TypeScript check passed;
- workstream JSON and JSONL validation passed;
- `git diff --check` passed with Windows line-ending warnings only.

## Review Result

No blocking workstream compliance findings remain:

- `GAARA-020` audited the current single-artifact and bulk apply seams;
- the lane explicitly avoids a second metadata apply executor;
- remaining backend wrapper work is deferred behind a new product requirement;
- Web-only preparation is covered by route-state tests and existing redaction
  checks.

No blocking code-quality findings remain:

- the mutation kernel remains centralized in
  `AutomationService::apply_generated_artifact_metadata`;
- bulk apply continues to delegate per item to the same one-artifact path;
- Web recovery remains read-only and hands only the artifact id to the apply
  route;
- no raw payload, prompt, path, provider response, token, secret, or
  idempotency key is exposed by the tested flow.

## Follow-Ons

- `proposed:generated-artifact-recovery-one-click-wrapper`: optional Admin
  wrapper for product-approved one-click row repair. It must only add
  recovery-context guards and delegate to existing apply/bulk apply behavior.
- `proposed:web-generated-artifact-repair-copy-polish`: optional Web copy,
  tooltip, or browser-smoke polish for the existing preparation-first flow.
- `proposed:metadata-provider-depth-and-precision`: provider identity/depth
  work remains separate from repair actions.
- `proposed:admin-settings-api-backed-restoration`: unrelated Admin settings
  restoration remains separate.

## Residual Risks

- Operators still confirm through the Metadata Authority apply page instead of
  a one-click row action. This is intentional until a product requirement
  justifies extra guard state.
- The current decision relies on the existing apply route as the live mutation
  guard. Any future repair wrapper must not replay old plan snapshots.
- PostgreSQL runtime parity was not re-run in this closeout because no schema
  or repository behavior changed.
